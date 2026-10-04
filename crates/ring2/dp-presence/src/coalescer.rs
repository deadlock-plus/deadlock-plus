use crate::Presence;

pub const MIN_SEND_INTERVAL_MS: u64 = 15_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Poll {
    Idle,
    Send(Option<Presence>),
    WaitMs(u64),
}

#[derive(Debug, Clone, Default)]
pub struct Coalescer {
    sent: Option<Presence>,
    last_send_ms: Option<u64>,
}

impl Coalescer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn poll(&mut self, desired: &Option<Presence>, now_ms: u64) -> Poll {
        if *desired == self.sent {
            return Poll::Idle;
        }
        if desired.is_some() {
            if let Some(last) = self.last_send_ms {
                let elapsed = now_ms.saturating_sub(last);
                if elapsed < MIN_SEND_INTERVAL_MS {
                    return Poll::WaitMs(MIN_SEND_INTERVAL_MS - elapsed);
                }
            }
        }
        self.sent = desired.clone();
        self.last_send_ms = Some(now_ms);
        Poll::Send(desired.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Option<Presence> {
        Some(Presence { details: Some(s.into()), ..Presence::default() })
    }

    #[test]
    fn nothing_to_do_when_nothing_sent_and_none_desired() {
        let mut c = Coalescer::new();
        assert_eq!(c.poll(&None, 0), Poll::Idle);
    }

    #[test]
    fn first_value_goes_out_immediately() {
        let mut c = Coalescer::new();
        assert_eq!(c.poll(&p("a"), 5), Poll::Send(p("a")));
    }

    #[test]
    fn unchanged_value_is_idle() {
        let mut c = Coalescer::new();
        c.poll(&p("a"), 0);
        assert_eq!(c.poll(&p("a"), 60_000), Poll::Idle);
    }

    #[test]
    fn change_inside_window_waits_for_remainder() {
        let mut c = Coalescer::new();
        c.poll(&p("a"), 1_000);
        assert_eq!(c.poll(&p("b"), 4_000), Poll::WaitMs(12_000));
    }

    #[test]
    fn trailing_update_sent_once_window_opens() {
        let mut c = Coalescer::new();
        c.poll(&p("a"), 0);
        assert_eq!(c.poll(&p("b"), 1_000), Poll::WaitMs(14_000));
        assert_eq!(c.poll(&p("b"), 15_000), Poll::Send(p("b")));
        assert_eq!(c.poll(&p("b"), 15_001), Poll::Idle);
    }

    #[test]
    fn only_latest_value_goes_out() {
        let mut c = Coalescer::new();
        c.poll(&p("a"), 0);
        c.poll(&p("b"), 1_000);
        c.poll(&p("c"), 2_000);
        assert_eq!(c.poll(&p("c"), 15_000), Poll::Send(p("c")));
    }

    #[test]
    fn reverting_to_sent_value_cancels_pending() {
        let mut c = Coalescer::new();
        c.poll(&p("a"), 0);
        assert_eq!(c.poll(&p("b"), 1_000), Poll::WaitMs(14_000));
        assert_eq!(c.poll(&p("a"), 2_000), Poll::Idle);
    }

    #[test]
    fn clear_goes_out_immediately_inside_window() {
        let mut c = Coalescer::new();
        c.poll(&p("a"), 0);
        assert_eq!(c.poll(&None, 100), Poll::Send(None));
        assert_eq!(c.poll(&None, 200), Poll::Idle);
    }

    #[test]
    fn set_after_clear_still_rate_limited() {
        let mut c = Coalescer::new();
        c.poll(&p("a"), 0);
        c.poll(&None, 100);
        assert_eq!(c.poll(&p("a"), 200), Poll::WaitMs(14_900));
    }

    #[test]
    fn reset_forgets_sent_state_and_window() {
        let mut c = Coalescer::new();
        c.poll(&p("a"), 0);
        c.reset();
        assert_eq!(c.poll(&p("a"), 1), Poll::Send(p("a")));
    }

    #[test]
    fn clock_going_backwards_does_not_panic() {
        let mut c = Coalescer::new();
        c.poll(&p("a"), 10_000);
        assert_eq!(c.poll(&p("b"), 5_000), Poll::WaitMs(15_000));
    }
}
