use std::time::{Duration, Instant};

use crate::LiveFacts;

/// Map changes leave the game without a rules entity for 3 to 26 seconds each side.
pub const LOAD_GRACE: Duration = Duration::from_secs(30);

/// Holds the last facts through a short gap so a map change does not flash a generic status.
#[derive(Default)]
pub struct LoadGrace {
    last: Option<(LiveFacts, Instant)>,
}

impl LoadGrace {
    pub fn apply(&mut self, read: Option<LiveFacts>, now: Instant) -> Option<LiveFacts> {
        if let Some(facts) = read {
            self.last = Some((facts, now));
            return Some(facts);
        }
        match self.last {
            Some((facts, at)) if now.saturating_duration_since(at) <= LOAD_GRACE => Some(facts),
            _ => {
                self.last = None;
                None
            }
        }
    }

    pub fn reset(&mut self) {
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Context;

    fn facts(context: Context) -> LiveFacts {
        LiveFacts { context, ..Default::default() }
    }

    #[test]
    fn a_gap_keeps_the_last_facts_for_the_grace_period() {
        let t0 = Instant::now();
        let mut g = LoadGrace::default();
        assert_eq!(g.apply(Some(facts(Context::Match)), t0), Some(facts(Context::Match)));
        assert_eq!(g.apply(None, t0 + Duration::from_secs(10)), Some(facts(Context::Match)));
        assert_eq!(g.apply(None, t0 + LOAD_GRACE), Some(facts(Context::Match)));
    }

    #[test]
    fn a_gap_longer_than_the_grace_period_gives_up() {
        let t0 = Instant::now();
        let mut g = LoadGrace::default();
        g.apply(Some(facts(Context::Hideout)), t0);
        assert_eq!(g.apply(None, t0 + LOAD_GRACE + Duration::from_secs(1)), None);
        assert_eq!(g.apply(None, t0 + Duration::from_secs(2)), None, "stale facts are not revived");
    }

    #[test]
    fn fresh_facts_replace_the_held_ones_and_restart_the_clock() {
        let t0 = Instant::now();
        let mut g = LoadGrace::default();
        g.apply(Some(facts(Context::Match)), t0);
        let t1 = t0 + Duration::from_secs(25);
        assert_eq!(g.apply(Some(facts(Context::Hideout)), t1), Some(facts(Context::Hideout)));
        assert_eq!(g.apply(None, t1 + Duration::from_secs(20)), Some(facts(Context::Hideout)));
    }

    #[test]
    fn nothing_seen_yet_stays_empty_and_reset_forgets() {
        let t0 = Instant::now();
        let mut g = LoadGrace::default();
        assert_eq!(g.apply(None, t0), None);
        g.apply(Some(facts(Context::Match)), t0);
        g.reset();
        assert_eq!(g.apply(None, t0), None);
    }
}
