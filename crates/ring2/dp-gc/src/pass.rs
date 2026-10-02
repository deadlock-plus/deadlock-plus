use std::collections::HashSet;
use std::future::Future;

use serde::{Deserialize, Serialize};

use crate::client::{GcSession, RecoveredSalts};
use crate::error::GcError;
use crate::quota::QuotaWindow;

/// How long an account whose GC handshake failed is left alone.
pub const GC_BACKOFF_SECS: i64 = 24 * 60 * 60;

/// Persisted per-account state. Keeps the quota across restarts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccountState {
    #[serde(default)]
    pub quota: QuotaWindow,
    #[serde(default)]
    pub backoff_until: Option<i64>,
}

impl AccountState {
    pub fn backed_off(&self, now: i64) -> bool {
        self.backoff_until.is_some_and(|until| now < until)
    }
}

pub trait SaltFetcher {
    fn fetch_match_salts(&self, match_id: u64) -> impl Future<Output = Result<RecoveredSalts, GcError>>;
}

impl SaltFetcher for GcSession {
    fn fetch_match_salts(&self, match_id: u64) -> impl Future<Output = Result<RecoveredSalts, GcError>> {
        GcSession::fetch_match_salts(self, match_id)
    }
}

/// What the caller supplies around the fetch loop: pacing, the stop condition and delivery.
pub trait Host {
    /// Waits out the request spacing. Returns false when the pass must stop, for example because
    /// Deadlock started.
    fn before_request(&mut self) -> impl Future<Output = bool>;
    /// Hands over recovered salts. Returns whether they were accepted.
    fn deliver(&mut self, salts: RecoveredSalts) -> impl Future<Output = bool>;
    fn now(&self) -> i64;
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    pub delivered: u32,
    pub rate_limited: bool,
}

/// Fetches `picks` one by one until they run out, the quota is spent, Steam rate-limits the
/// account or the host asks to stop.
pub async fn fetch_loop(
    fetcher: &impl SaltFetcher,
    host: &mut impl Host,
    picks: Vec<u64>,
    state: &mut AccountState,
    processed: &mut HashSet<u64>,
) -> Outcome {
    let mut out = Outcome::default();
    for match_id in picks {
        if state.quota.remaining(host.now()) == 0 || !host.before_request().await {
            break;
        }
        let salts = match fetcher.fetch_match_salts(match_id).await {
            Ok(salts) => salts,
            // Left unmarked so another account can still fetch it.
            Err(GcError::GcRateLimited) => {
                state.quota.exhaust(host.now());
                out.rate_limited = true;
                break;
            }
            Err(e) => {
                log::warn!("gc: salt fetch failed for {match_id}: {e}");
                processed.insert(match_id);
                continue;
            }
        };
        processed.insert(match_id);
        state.quota.try_consume(host.now());
        if host.deliver(salts).await {
            out.delivered += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    const NOW: i64 = 1_000_000;

    struct Fake(RefCell<HashMap<u64, Result<RecoveredSalts, GcError>>>);

    impl Fake {
        fn new(entries: Vec<(u64, Result<RecoveredSalts, GcError>)>) -> Self {
            Self(RefCell::new(entries.into_iter().collect()))
        }
    }

    impl SaltFetcher for Fake {
        async fn fetch_match_salts(&self, match_id: u64) -> Result<RecoveredSalts, GcError> {
            self.0.borrow_mut().remove(&match_id).expect("unexpected fetch")
        }
    }

    struct TestHost {
        allow_requests: usize,
        accept: bool,
        delivered: Vec<u64>,
    }

    impl TestHost {
        fn new() -> Self {
            Self { allow_requests: usize::MAX, accept: true, delivered: Vec::new() }
        }
    }

    impl Host for TestHost {
        async fn before_request(&mut self) -> bool {
            if self.allow_requests == 0 {
                return false;
            }
            self.allow_requests -= 1;
            true
        }
        async fn deliver(&mut self, salts: RecoveredSalts) -> bool {
            self.delivered.push(salts.match_id);
            self.accept
        }
        fn now(&self) -> i64 {
            NOW
        }
    }

    fn salts(id: u64) -> RecoveredSalts {
        RecoveredSalts { match_id: id, cluster_id: Some(1), metadata_salt: Some(2), replay_salt: None, account_id: 7 }
    }

    fn run(
        fetcher: &Fake,
        host: &mut TestHost,
        picks: Vec<u64>,
        state: &mut AccountState,
        processed: &mut HashSet<u64>,
    ) -> Outcome {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(fetch_loop(fetcher, host, picks, state, processed))
    }

    #[test]
    fn success_delivers_consumes_quota_and_marks_processed() {
        let fetcher = Fake::new(vec![(9, Ok(salts(9)))]);
        let (mut host, mut state, mut processed) = (TestHost::new(), AccountState::default(), HashSet::new());
        let out = run(&fetcher, &mut host, vec![9], &mut state, &mut processed);
        assert_eq!(out, Outcome { delivered: 1, rate_limited: false });
        assert_eq!(host.delivered, vec![9]);
        assert_eq!(state.quota.remaining(NOW), crate::quota::FETCH_QUOTA_LIMIT - 1);
        assert!(processed.contains(&9));
    }

    #[test]
    fn rate_limit_exhausts_quota_and_leaves_the_id_for_other_accounts() {
        let fetcher = Fake::new(vec![(9, Err(GcError::GcRateLimited))]);
        let (mut host, mut state, mut processed) = (TestHost::new(), AccountState::default(), HashSet::new());
        let out = run(&fetcher, &mut host, vec![9, 8], &mut state, &mut processed);
        assert!(out.rate_limited);
        assert_eq!(state.quota.remaining(NOW), 0);
        assert!(!processed.contains(&9));
        assert!(host.delivered.is_empty());
    }

    #[test]
    fn other_failures_skip_the_id_without_spending_quota() {
        let fetcher = Fake::new(vec![(9, Err(GcError::GcUnavailable("x".into()))), (8, Ok(salts(8)))]);
        let (mut host, mut state, mut processed) = (TestHost::new(), AccountState::default(), HashSet::new());
        let out = run(&fetcher, &mut host, vec![9, 8], &mut state, &mut processed);
        assert_eq!(out.delivered, 1);
        assert!(processed.contains(&9));
        assert_eq!(state.quota.remaining(NOW), crate::quota::FETCH_QUOTA_LIMIT - 1);
    }

    #[test]
    fn host_stop_ends_the_loop_before_fetching() {
        let fetcher = Fake::new(vec![]);
        let mut host = TestHost { allow_requests: 0, ..TestHost::new() };
        let (mut state, mut processed) = (AccountState::default(), HashSet::new());
        let out = run(&fetcher, &mut host, vec![9], &mut state, &mut processed);
        assert_eq!(out, Outcome::default());
    }

    #[test]
    fn a_rejected_delivery_still_costs_quota_but_is_not_counted() {
        let fetcher = Fake::new(vec![(9, Ok(salts(9)))]);
        let mut host = TestHost { accept: false, ..TestHost::new() };
        let (mut state, mut processed) = (AccountState::default(), HashSet::new());
        let out = run(&fetcher, &mut host, vec![9], &mut state, &mut processed);
        assert_eq!(out.delivered, 0);
        assert_eq!(state.quota.remaining(NOW), crate::quota::FETCH_QUOTA_LIMIT - 1);
    }

    #[test]
    fn stops_when_the_quota_runs_out_mid_loop() {
        let mut state = AccountState::default();
        for _ in 0..crate::quota::FETCH_QUOTA_LIMIT - 1 {
            state.quota.try_consume(NOW);
        }
        let fetcher = Fake::new(vec![(9, Ok(salts(9))), (8, Ok(salts(8)))]);
        let (mut host, mut processed) = (TestHost::new(), HashSet::new());
        let out = run(&fetcher, &mut host, vec![9, 8], &mut state, &mut processed);
        assert_eq!(out.delivered, 1);
        assert!(!processed.contains(&8));
    }

    #[test]
    fn backoff_applies_until_its_deadline() {
        let state = AccountState { backoff_until: Some(NOW + 10), ..Default::default() };
        assert!(state.backed_off(NOW));
        assert!(!state.backed_off(NOW + 10));
        assert!(!AccountState::default().backed_off(NOW));
    }
}
