use crate::{redact, CrashKind, CrashMarker};

pub const ISSUE_REPO: &str = "deadlock-plus/deadlock-plus";
/// GitHub rejects request lines much past 8 KB; stay well under.
pub const MAX_URL_LEN: usize = 6000;
const TITLE_CHARS: usize = 80;
const SUMMARY_CHARS: usize = 800;

fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 2);
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn truncate(text: &str, chars: usize) -> String {
    match text.char_indices().nth(chars) {
        Some((end, _)) => format!("{}...", &text[..end]),
        None => text.to_string(),
    }
}

fn url(title: &str, body: &str) -> String {
    format!("https://github.com/{ISSUE_REPO}/issues/new?title={}&body={}", encode(title), encode(body))
}

/// A pre-filled "new issue" link. The body carries the version, OS, a redacted summary and a line
/// asking for the report file; the file itself is attached by hand. The summary shrinks until the
/// link fits `MAX_URL_LEN`.
pub fn issue_url(marker: &CrashMarker, bundle_file_name: &str) -> String {
    let message = redact(&marker.message);
    let first_line = message.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("no message");
    let prefix = match marker.kind {
        CrashKind::UncleanExit => "Possible crash",
        CrashKind::Panic | CrashKind::Webview => "Crash",
    };
    let title = format!("{prefix}: {}", truncate(first_line, TITLE_CHARS));
    let mut budget = SUMMARY_CHARS;
    loop {
        let body = format!(
            "**Version:** Deadlock+ {}\n**OS:** {}\n**Kind:** {}\n\n**Summary**\n```\n{}\n```\n\nPlease attach the crash report file `{}` to this issue.\n",
            redact(&marker.version),
            redact(&marker.os),
            marker.kind.as_str(),
            truncate(message.trim(), budget),
            bundle_file_name,
        );
        let link = url(&title, &body);
        if link.len() <= MAX_URL_LEN || budget == 0 {
            return link;
        }
        budget /= 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(message: &str) -> CrashMarker {
        CrashMarker {
            kind: CrashKind::Panic,
            timestamp_ms: 0,
            version: "0.5.0".into(),
            os: "Windows 11".into(),
            message: message.into(),
            backtrace: None,
            log: None,
        }
    }

    fn decode(text: &str) -> String {
        let bytes = text.as_bytes();
        let mut out = Vec::new();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' {
                out.push(u8::from_str_radix(&text[i + 1..i + 3], 16).unwrap());
                i += 3;
            } else {
                out.push(bytes[i]);
                i += 1;
            }
        }
        String::from_utf8(out).unwrap()
    }

    fn part<'a>(link: &'a str, key: &str) -> &'a str {
        let query = link.split_once('?').unwrap().1;
        query.split('&').find_map(|p| p.strip_prefix(key)).unwrap()
    }

    #[test]
    fn targets_the_project_repo() {
        let link = issue_url(&marker("boom"), "r.txt");
        assert!(link.starts_with("https://github.com/deadlock-plus/deadlock-plus/issues/new?title="));
    }

    #[test]
    fn title_and_body_are_percent_encoded() {
        let link = issue_url(&marker("a & b = c\nnext"), "r.txt");
        assert!(!link.contains(' ') && !link.contains('\n'));
        assert_eq!(link.matches('&').count(), 1);
        assert_eq!(decode(part(&link, "title=")), "Crash: a & b = c");
        assert!(decode(part(&link, "body=")).contains("a & b = c\nnext"));
    }

    #[test]
    fn non_ascii_text_survives_the_round_trip() {
        let link = issue_url(&marker("fehlgeschlagen: \u{fc}\u{1f4a5}"), "r.txt");
        assert!(decode(part(&link, "body=")).contains("fehlgeschlagen: \u{fc}\u{1f4a5}"));
    }

    #[test]
    fn body_has_version_os_summary_and_the_attach_line() {
        let link = issue_url(&marker("boom"), "report-1.txt");
        let body = decode(part(&link, "body="));
        assert!(body.contains("Deadlock+ 0.5.0"));
        assert!(body.contains("Windows 11"));
        assert!(body.contains("boom"));
        assert!(body.contains("Please attach the crash report file `report-1.txt`"));
    }

    #[test]
    fn a_huge_message_is_capped() {
        let link = issue_url(&marker(&"x".repeat(100_000)), "r.txt");
        assert!(link.len() <= MAX_URL_LEN);
        assert!(decode(part(&link, "body=")).contains("attach the crash report file"));
    }

    #[test]
    fn a_message_of_multibyte_characters_is_capped() {
        let link = issue_url(&marker(&"\u{1f4a5}".repeat(5_000)), "r.txt");
        assert!(link.len() <= MAX_URL_LEN);
    }

    #[test]
    fn the_user_name_never_appears() {
        let link = issue_url(&marker(r"panicked at C:\Users\Alice\src\x.rs"), "r.txt");
        assert!(!decode(&link).contains("Alice"));
    }

    #[test]
    fn an_unclean_exit_is_titled_as_possible() {
        let mut m = marker("did not exit cleanly");
        m.kind = CrashKind::UncleanExit;
        assert!(decode(part(&issue_url(&m, "r.txt"), "title=")).starts_with("Possible crash: "));
    }
}
