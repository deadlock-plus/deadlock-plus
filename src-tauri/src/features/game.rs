use std::ffi::OsStr;

/// Deadlock runs as `deadlock.exe` under Wine and Proton too.
pub const PROCESS_NAME: &str = "deadlock.exe";

pub fn is_process(name: &OsStr) -> bool {
    name.to_string_lossy().eq_ignore_ascii_case(PROCESS_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_game_exe_in_any_case_only() {
        assert!(is_process(OsStr::new("deadlock.exe")));
        assert!(is_process(OsStr::new("Deadlock.EXE")));
        assert!(!is_process(OsStr::new("deadlock")));
        assert!(!is_process(OsStr::new("deadlock-plus.exe")));
    }
}
