use std::ffi::OsStr;

/// Deadlock runs as `deadlock.exe` under Wine and Proton too.
pub const PROCESS_NAME: &str = "deadlock.exe";

pub fn is_process(name: &OsStr) -> bool {
    name.to_string_lossy().eq_ignore_ascii_case(PROCESS_NAME)
}

fn any_process<'a>(names: impl IntoIterator<Item = &'a OsStr>) -> bool {
    names.into_iter().any(is_process)
}

pub fn is_running() -> bool {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    any_process(sys.processes().values().map(|p| p.name()))
}

fn earliest_start<'a>(procs: impl IntoIterator<Item = (&'a OsStr, u64)>) -> Option<u64> {
    procs.into_iter().filter(|(name, _)| is_process(name)).map(|(_, start)| start).min()
}

/// Unix seconds at which the oldest running Deadlock process started, or `None` when the game is not running.
pub fn start_time() -> Option<u64> {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    earliest_start(sys.processes().values().map(|p| (p.name(), p.start_time())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_process_finds_the_game_among_names() {
        assert!(any_process(["explorer.exe", "Deadlock.exe"].map(OsStr::new)));
        assert!(!any_process(["explorer.exe", "deadlock-plus.exe"].map(OsStr::new)));
        assert!(!any_process(std::iter::empty::<&OsStr>()));
    }

    #[test]
    fn earliest_start_among_matching_processes_wins() {
        let procs = [("explorer.exe", 5), ("deadlock.exe", 300), ("Deadlock.exe", 200), ("other.exe", 1)];
        assert_eq!(earliest_start(procs.map(|(n, t)| (OsStr::new(n), t))), Some(200));
        assert_eq!(earliest_start([(OsStr::new("explorer.exe"), 5)]), None);
    }

    #[test]
    fn matches_the_game_exe_in_any_case_only() {
        assert!(is_process(OsStr::new("deadlock.exe")));
        assert!(is_process(OsStr::new("Deadlock.EXE")));
        assert!(!is_process(OsStr::new("deadlock")));
        assert!(!is_process(OsStr::new("deadlock-plus.exe")));
    }
}
