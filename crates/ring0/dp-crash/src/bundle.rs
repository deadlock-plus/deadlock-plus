use crate::{redact, CrashMarker};

pub const LOG_LINES: usize = 500;

/// The last `count` lines of `text`, joined with `\n`.
pub fn tail_lines(text: &str, count: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(count)..].join("\n")
}

/// `2026-09-30 12:00:00 UTC` for a Unix time in milliseconds.
pub fn format_utc(timestamp_ms: u64) -> String {
    let secs = timestamp_ms / 1000;
    let (days, rem) = ((secs / 86_400) as i64, secs % 86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// The plain-text report a user attaches to an issue. `current_log` is used only when the marker
/// carries no log of its own (a marker written this run). Everything is redacted.
pub fn build_bundle(marker: &CrashMarker, current_log: &str) -> String {
    let log = marker.log.as_deref().unwrap_or(current_log);
    let mut out = format!(
        "Deadlock+ crash report\nVersion: {}\nOS: {}\nKind: {}\nTime: {}\n\nMessage\n-------\n{}\n",
        marker.version,
        marker.os,
        marker.kind.as_str(),
        format_utc(marker.timestamp_ms),
        marker.message.trim_end()
    );
    if let Some(backtrace) = marker.backtrace.as_deref().filter(|b| !b.trim().is_empty()) {
        out.push_str(&format!("\nBacktrace\n---------\n{}\n", backtrace.trim_end()));
    }
    out.push_str(&format!("\nLog (last {LOG_LINES} lines)\n-------------------------\n"));
    out.push_str(&tail_lines(log, LOG_LINES));
    out.push('\n');
    redact(&out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CrashKind;

    fn marker() -> CrashMarker {
        CrashMarker {
            kind: CrashKind::Panic,
            timestamp_ms: 1_790_000_000_000,
            version: "0.5.0".into(),
            os: "Windows 11".into(),
            message: r"panicked at C:\Users\Alice\src\x.rs:1".into(),
            backtrace: Some("0: foo\n1: bar".into()),
            log: None,
        }
    }

    #[test]
    fn formats_unix_time_as_utc() {
        assert_eq!(format_utc(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(format_utc(1_709_210_096_000), "2024-02-29 12:34:56 UTC");
        assert_eq!(format_utc(1_790_000_000_000), "2026-09-21 14:13:20 UTC");
    }

    #[test]
    fn header_names_version_os_kind_and_time() {
        let text = build_bundle(&marker(), "");
        assert!(text.starts_with("Deadlock+ crash report\nVersion: 0.5.0\nOS: Windows 11\nKind: panic\n"));
        assert!(text.contains("Time: 2026-09-21 14:13:20 UTC"));
        assert!(text.contains("Backtrace\n---------\n0: foo\n1: bar"));
    }

    #[test]
    fn user_names_are_redacted_everywhere() {
        let mut m = marker();
        m.log = Some(r"[12:00:00] [main | INFO] [a]: read C:\Users\Alice\file".into());
        let text = build_bundle(&m, "");
        assert!(!text.contains("Alice"));
        assert!(text.contains(r"C:\Users\<user>\src\x.rs:1"));
        assert!(text.contains(r"C:\Users\<user>\file"));
    }

    #[test]
    fn tokens_in_the_log_are_masked() {
        let mut m = marker();
        m.log = Some("sent token=abc123 ok".into());
        let text = build_bundle(&m, "");
        assert!(!text.contains("abc123"));
    }

    #[test]
    fn keeps_only_the_last_lines_of_the_log() {
        let log: String = (1..=700).map(|n| format!("line {n}\n")).collect();
        let text = build_bundle(&marker(), &log);
        assert!(!text.contains("line 200\n"));
        assert!(text.contains("line 201\n"));
        assert!(text.contains("line 700\n"));
        assert_eq!(text.matches("\nline ").count(), LOG_LINES);
    }

    #[test]
    fn a_markers_own_log_wins_over_the_current_one() {
        let mut m = marker();
        m.log = Some("from the crashed run".into());
        let text = build_bundle(&m, "from this run");
        assert!(text.contains("from the crashed run"));
        assert!(!text.contains("from this run"));
    }

    #[test]
    fn a_missing_backtrace_leaves_no_section() {
        let mut m = marker();
        m.backtrace = None;
        assert!(!build_bundle(&m, "").contains("Backtrace"));
    }

    #[test]
    fn tail_of_short_text_is_all_of_it() {
        assert_eq!(tail_lines("a\nb", 5), "a\nb");
        assert_eq!(tail_lines("", 5), "");
    }
}
