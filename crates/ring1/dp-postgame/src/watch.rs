#[derive(Debug, PartialEq, Eq)]
pub enum Plan {
    Start(u32),
    Restart(u32),
    Keep,
    Wait,
}

pub fn plan(running: Option<u32>, signed_in: Option<u32>) -> Plan {
    match (running, signed_in) {
        (None, Some(id)) => Plan::Start(id),
        (None, None) => Plan::Wait,
        (Some(current), Some(id)) if current != id => Plan::Restart(id),
        _ => Plan::Keep,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_once_an_account_is_signed_in() {
        assert_eq!(plan(None, Some(7)), Plan::Start(7));
    }

    #[test]
    fn waits_while_nothing_runs_and_nobody_is_signed_in() {
        assert_eq!(plan(None, None), Plan::Wait);
    }

    #[test]
    fn keeps_a_capture_for_the_same_account() {
        assert_eq!(plan(Some(7), Some(7)), Plan::Keep);
    }

    #[test]
    fn restarts_for_a_different_account() {
        assert_eq!(plan(Some(7), Some(8)), Plan::Restart(8));
    }

    #[test]
    fn keeps_a_running_capture_when_the_account_is_briefly_unreadable() {
        assert_eq!(plan(Some(7), None), Plan::Keep);
    }
}
