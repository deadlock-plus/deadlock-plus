use std::panic::{self, AssertUnwindSafe};

use dp_crash::{format_utc, redact, tail_lines, CrashMarker, LOG_LINES};

pub const MAX_REPORT_BYTES: usize = 64 * 1024;
pub const STRIPPED: &str = "<stripped>";

pub type Fields = Vec<(String, String)>;

pub trait Probe {
    fn collect(&self) -> Result<Fields, String>;
}

impl<F: Fn() -> Result<Fields, String>> Probe for F {
    fn collect(&self) -> Result<Fields, String> {
        self()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section {
    App,
    Game,
    Features,
    Platform,
    Jobs,
    Settings,
}

const SECRET_KEY_PARTS: [&str; 12] = [
    "token",
    "secret",
    "password",
    "passwd",
    "apikey",
    "authorization",
    "credential",
    "cookie",
    "webhook",
    "privatekey",
    "installid",
    "sessionid",
];

const LOG_TRUNCATED: &str = "[log truncated: older lines omitted]\n";
const REPORT_TRUNCATED: &str = "\n[report truncated]\n";

/// Matches regardless of case and of `_`, `-`, `.` and space separators, so `API-Key`, `api_key` and
/// `apiKey` are all the same key.
pub fn is_secret_key(key: &str) -> bool {
    let normalized: String =
        key.chars().filter(|c| !matches!(c, '_' | '-' | '.' | ' ')).flat_map(char::to_lowercase).collect();
    SECRET_KEY_PARTS.iter().any(|part| normalized.contains(part))
}

/// Builds the plain-text support report. Sections not supplied are left out, and they always render in
/// the order of [`Section`], then Crash, then Log. The result is redacted and at most
/// [`MAX_REPORT_BYTES`] long; the log tail is what gives way first.
pub fn build_report(sections: &[(Section, &dyn Probe)], log_tail: &str, crash_marker: Option<&CrashMarker>) -> String {
    let mut ordered: Vec<&(Section, &dyn Probe)> = sections.iter().collect();
    ordered.sort_by_key(|(section, _)| *section);

    let mut head = String::from("Deadlock+ support report\n");
    for (section, probe) in ordered {
        head.push_str(&format!("\n== {} ==\n", section.title()));
        match run_probe(*probe) {
            Ok(fields) if fields.is_empty() => head.push_str("(none)\n"),
            Ok(fields) => {
                for (key, value) in fields {
                    let value = if *section == Section::Settings && is_secret_key(&key) { STRIPPED } else { &value };
                    push_field(&mut head, &key, value);
                }
            }
            Err(error) => head.push_str(&format!("unavailable: {}\n", error.trim_end())),
        }
    }
    head.push_str("\n== Crash ==\n");
    match crash_marker {
        None => head.push_str("none\n"),
        Some(marker) => {
            push_field(&mut head, "kind", marker.kind.as_str());
            push_field(&mut head, "time", &format_utc(marker.timestamp_ms));
            push_field(&mut head, "version", &marker.version);
            push_field(&mut head, "os", &marker.os);
            push_field(&mut head, "message", &marker.message);
        }
    }
    head.push_str("\n== Log ==\n");

    let head = redact(&head);
    let log = redact(&tail_lines(log_tail, LOG_LINES));
    fit(head, &log)
}

impl Section {
    fn title(self) -> &'static str {
        match self {
            Section::App => "App",
            Section::Game => "Game",
            Section::Features => "Features",
            Section::Platform => "Platform",
            Section::Jobs => "Jobs",
            Section::Settings => "Settings",
        }
    }
}

fn run_probe(probe: &dyn Probe) -> Result<Fields, String> {
    panic::catch_unwind(AssertUnwindSafe(|| probe.collect())).unwrap_or_else(|_| Err("probe panicked".into()))
}

fn push_field(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(": ");
    out.push_str(&value.trim_end().replace('\n', "\n  "));
    out.push('\n');
}

fn fit(head: String, log: &str) -> String {
    if head.len() + log.len() < MAX_REPORT_BYTES {
        return format!("{head}{log}\n");
    }
    if head.len() + LOG_TRUNCATED.len() + REPORT_TRUNCATED.len() > MAX_REPORT_BYTES {
        let keep = floor_boundary(&head, MAX_REPORT_BYTES - REPORT_TRUNCATED.len());
        return format!("{}{REPORT_TRUNCATED}", &head[..keep]);
    }
    let budget = MAX_REPORT_BYTES - head.len() - LOG_TRUNCATED.len() - 1;
    let mut kept = &log[ceil_boundary(log, log.len().saturating_sub(budget))..];
    // Start on a whole line; a single line longer than the budget keeps its mid-line cut.
    if let Some(newline) = kept.find('\n') {
        if newline + 1 < kept.len() {
            kept = &kept[newline + 1..];
        }
    }
    format!("{head}{LOG_TRUNCATED}{kept}\n")
}

fn floor_boundary(text: &str, mut at: usize) -> usize {
    while !text.is_char_boundary(at) {
        at -= 1;
    }
    at
}

fn ceil_boundary(text: &str, mut at: usize) -> usize {
    while !text.is_char_boundary(at) {
        at += 1;
    }
    at
}

#[cfg(test)]
mod tests {
    use super::*;
    use dp_crash::CrashKind;

    fn ok(fields: &[(&str, &str)]) -> impl Fn() -> Result<Fields, String> {
        let fields: Fields = fields.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        move || Ok(fields.clone())
    }

    fn marker() -> CrashMarker {
        CrashMarker {
            kind: CrashKind::Panic,
            timestamp_ms: 1_790_000_000_000,
            version: "0.10.0".into(),
            os: "Windows 11".into(),
            message: "boom".into(),
            backtrace: Some("0: secret_frame".into()),
            log: None,
        }
    }

    #[test]
    fn redacts_user_paths_and_tokens_everywhere() {
        let game = ok(&[("Install path", r"C:\Users\Alice\Steam\deadlock"), ("note", "token=abc123")]);
        let report = build_report(&[(Section::Game, &game)], r"read C:\Users\Bob\x api_key=k9", None);
        assert!(!report.contains("Alice") && !report.contains("Bob"));
        assert!(!report.contains("abc123") && !report.contains("k9"));
        assert!(report.contains(r"C:\Users\<user>\Steam\deadlock"));
    }

    #[test]
    fn a_failing_probe_does_not_abort_the_report() {
        let bad = || -> Result<Fields, String> { Err("reader detached".into()) };
        let good = ok(&[("version", "0.10.0")]);
        let report =
            build_report(&[(Section::App, &good), (Section::Features, &bad), (Section::Jobs, &good)], "", None);
        assert!(report.contains("unavailable: reader detached"));
        assert_eq!(report.matches("version: 0.10.0").count(), 2);
        assert!(report.contains("== Log =="));
    }

    #[test]
    fn a_panicking_probe_is_isolated_too() {
        let bad = || -> Result<Fields, String> { panic!("kaput") };
        let report = build_report(&[(Section::Features, &bad)], "", None);
        assert!(report.contains("unavailable: probe panicked"));
    }

    #[test]
    fn the_size_cap_trims_the_log_first() {
        let app = ok(&[("version", "0.10.0")]);
        let pad = "padding ".repeat(20);
        let log: String = (0..5000).map(|i| format!("line {i:05} {pad}\n")).collect();
        let report = build_report(&[(Section::App, &app)], &log, None);
        assert!(report.len() <= MAX_REPORT_BYTES);
        assert!(report.contains("version: 0.10.0"));
        assert!(report.contains("line 04999"));
        assert!(!report.contains("line 04500 "));
        assert!(report.contains("[log truncated"));
    }

    #[test]
    fn trimming_never_cuts_inside_a_character() {
        let app = ok(&[("version", "0.10.0")]);
        let log = "é".repeat(MAX_REPORT_BYTES);
        let report = build_report(&[(Section::App, &app)], &log, None);
        assert!(report.len() <= MAX_REPORT_BYTES);
        assert!(report.contains("[log truncated"));
    }

    #[test]
    fn an_oversized_section_is_cut_and_marked() {
        let big = ok(&[("blob", &"x".repeat(MAX_REPORT_BYTES * 2))]);
        let report = build_report(&[(Section::Settings, &big)], "tail", None);
        assert!(report.len() <= MAX_REPORT_BYTES);
        assert!(report.contains("[report truncated]"));
    }

    #[test]
    fn sections_come_in_a_fixed_order_whatever_the_input_order() {
        let p = ok(&[("k", "v")]);
        let report = build_report(
            &[
                (Section::Settings, &p),
                (Section::Jobs, &p),
                (Section::Platform, &p),
                (Section::Features, &p),
                (Section::Game, &p),
                (Section::App, &p),
            ],
            "one\ntwo",
            Some(&marker()),
        );
        let headings: Vec<&str> = report.lines().filter(|l| l.starts_with("== ")).collect();
        assert_eq!(
            headings,
            [
                "== App ==",
                "== Game ==",
                "== Features ==",
                "== Platform ==",
                "== Jobs ==",
                "== Settings ==",
                "== Crash ==",
                "== Log =="
            ]
        );
    }

    #[test]
    fn unsupplied_sections_are_omitted_and_the_crash_section_says_none() {
        let p = ok(&[("k", "v")]);
        let report = build_report(&[(Section::App, &p)], "", None);
        assert!(!report.contains("== Platform =="));
        assert!(report.contains("== Crash ==\nnone"));
    }

    #[test]
    fn the_crash_section_shows_the_marker_without_its_backtrace() {
        let report = build_report(&[], "", Some(&marker()));
        assert!(report.contains("kind: panic"));
        assert!(report.contains("message: boom"));
        assert!(report.contains("2026-09-21") || report.contains("UTC"));
        assert!(!report.contains("secret_frame"));
    }

    #[test]
    fn secret_looking_settings_are_stripped() {
        let settings = ok(&[
            ("language", "en"),
            ("ingest_token", "tok-123"),
            ("discordWebhook", "https://discord.com/api/webhooks/1/abc"),
            ("API-Key", "k-77"),
            ("password", "hunter2"),
            ("install_id", "7f3e"),
        ]);
        let report = build_report(&[(Section::Settings, &settings)], "", None);
        assert!(report.contains("language: en"));
        for leaked in ["tok-123", "webhooks/1", "k-77", "hunter2", "7f3e"] {
            assert!(!report.contains(leaked), "{leaked} leaked");
        }
        // The final redact pass rewrites the value after a `token`/`password` key to its own marker.
        assert_eq!(report.matches(STRIPPED).count() + report.matches("<redacted>").count(), 5);
    }

    #[test]
    fn only_the_settings_section_strips_by_key() {
        let game = ok(&[("token_count", "3")]);
        let report = build_report(&[(Section::Game, &game)], "", None);
        assert!(report.contains("token_count: 3"));
    }

    #[test]
    fn multi_line_values_stay_inside_their_field() {
        let p = ok(&[("jobs", "a\nb")]);
        let report = build_report(&[(Section::Jobs, &p)], "", None);
        assert!(report.contains("jobs: a\n  b\n"));
    }
}
