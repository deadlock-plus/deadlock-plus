#[derive(Debug, PartialEq, Eq)]
pub enum Toggle {
    Start,
    Stop,
    Keep,
}

/// The frontend re-sends each background setting on every reload. Restarting a running worker
/// would redo its start-up work each time, which for GC recovery means signing in to every Steam
/// account again.
pub fn toggle(enabled: bool, running: bool) -> Toggle {
    match (enabled, running) {
        (true, false) => Toggle::Start,
        (false, true) => Toggle::Stop,
        _ => Toggle::Keep,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_a_running_worker_and_ignores_a_repeated_off() {
        assert_eq!(toggle(true, true), Toggle::Keep);
        assert_eq!(toggle(false, false), Toggle::Keep);
    }

    #[test]
    fn starts_and_stops_on_a_real_change() {
        assert_eq!(toggle(true, false), Toggle::Start);
        assert_eq!(toggle(false, true), Toggle::Stop);
    }
}
