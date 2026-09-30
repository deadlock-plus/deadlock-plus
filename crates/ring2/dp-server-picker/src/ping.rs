use std::sync::OnceLock;

use tokio::sync::Semaphore;

/// First try plus three retries, each with a longer timeout. Only groups that got no
/// reply are retried, so the common case stays fast.
const TIMEOUTS_MS: [u32; 4] = [1000, 2000, 3000, 4000];
const IPS_PER_GROUP: usize = 3;
const MAX_CONCURRENT_GROUPS: usize = 24;

/// One native ICMP echo. Spawning `ping.exe` per relay made dozens of concurrent
/// child processes, and many of them were dropped under that load.
fn ping_ip(ip: &str, timeout_ms: u32) -> Option<u32> {
    let addr: std::net::Ipv4Addr = ip.parse().ok()?;
    dp_icmp::ping(addr, timeout_ms).map(|ms| ms.ceil() as u32)
}

/// Lowest reply across `ips`, retrying the whole sample with the next (longer) timeout
/// while nothing has answered.
fn best_latency(ips: &[String], timeouts_ms: &[u32], ping: impl Fn(&str, u32) -> Option<u32>) -> Option<u32> {
    timeouts_ms.iter().find_map(|&timeout| ips.iter().filter_map(|ip| ping(ip, timeout)).min())
}

fn permits() -> &'static Semaphore {
    static PERMITS: OnceLock<Semaphore> = OnceLock::new();
    PERMITS.get_or_init(|| Semaphore::new(MAX_CONCURRENT_GROUPS))
}

/// Best (lowest) latency over a sample of the group's relays, since SDR routes through whichever is closest.
pub async fn ping_group(ips: &[String]) -> Option<u32> {
    let sample: Vec<String> = ips.iter().take(IPS_PER_GROUP).cloned().collect();
    let _permit = permits().acquire().await.ok()?;
    tokio::task::spawn_blocking(move || best_latency(&sample, &TIMEOUTS_MS, ping_ip)).await.ok()?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn a_failing_group_is_retried_with_increasing_timeouts() {
        let seen = RefCell::new(Vec::new());
        let ping = |_: &str, timeout: u32| {
            seen.borrow_mut().push(timeout);
            (timeout >= 2000).then_some(40)
        };
        assert_eq!(best_latency(&["a".into()], &[1000, 2000, 4000], ping), Some(40));
        assert_eq!(*seen.borrow(), vec![1000, 2000]);
    }

    #[test]
    fn a_group_that_answers_first_time_is_not_retried() {
        let calls = RefCell::new(0);
        let ping = |_: &str, _: u32| {
            *calls.borrow_mut() += 1;
            Some(25)
        };
        assert_eq!(best_latency(&["a".into()], &[1000, 2000], ping), Some(25));
        assert_eq!(*calls.borrow(), 1);
    }

    #[test]
    fn group_reports_the_lowest_reply() {
        let by_ip = |ip: &str, _: u32| match ip {
            "a" => Some(90),
            "b" => Some(30),
            _ => None,
        };
        assert_eq!(best_latency(&["a".into(), "b".into(), "c".into()], &[1000], by_ip), Some(30));
    }

    #[test]
    fn group_gives_up_after_the_last_timeout() {
        assert_eq!(best_latency(&["a".into()], &[500, 1000, 2000, 3000], |_, _| None), None);
    }
}
