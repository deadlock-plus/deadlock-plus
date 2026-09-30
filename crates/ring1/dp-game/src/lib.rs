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
    fn matches_the_game_exe_in_any_case_only() {
        assert!(is_process(OsStr::new("deadlock.exe")));
        assert!(is_process(OsStr::new("Deadlock.EXE")));
        assert!(!is_process(OsStr::new("deadlock")));
        assert!(!is_process(OsStr::new("deadlock-plus.exe")));
    }
}
