use std::ffi::OsString;
use std::path::{Path, PathBuf};

pub const FILE_NAME: &str = "deadlock-plus.desktop";

/// Per the XDG base directory spec, a relative `$XDG_CONFIG_HOME` is invalid and must be ignored.
pub fn entry_path(xdg_config_home: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    let config = xdg_config_home
        .filter(|v| Path::new(v).has_root())
        .map(PathBuf::from)
        .or_else(|| home.filter(|h| !h.is_empty()).map(|h| PathBuf::from(h).join(".config")))?;
    Some(config.join("autostart").join(FILE_NAME))
}

/// The Exec value is double-escaped: reserved characters get a backslash inside the quotes, then the
/// desktop-file string layer doubles every backslash.
fn exec_arg(arg: &str) -> String {
    let mut quoted = String::from("\"");
    for c in arg.chars() {
        match c {
            '"' | '`' | '$' | '\\' => {
                quoted.push('\\');
                quoted.push(c);
            }
            '%' => quoted.push_str("%%"),
            _ => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

fn escape_value(v: &str) -> String {
    v.replace('\\', "\\\\").replace('\n', "\\n").replace('\r', "\\r")
}

fn unescape_value(v: &str) -> String {
    let mut out = String::with_capacity(v.len());
    let mut chars = v.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('s') => out.push(' '),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

pub fn render(exe: &str, launch_arg: &str) -> String {
    let exec = escape_value(&format!("{} {launch_arg}", exec_arg(exe)));
    format!(
        "[Desktop Entry]\nType=Application\nName=Deadlock+\nComment=Starts Deadlock+ when you sign in\nExec={exec}\nTerminal=false\nX-GNOME-Autostart-enabled=true\n"
    )
}

fn first_exec_arg(exec: &str) -> Option<String> {
    let mut out = String::new();
    let mut chars = exec.trim_start().chars().peekable();
    let quoted = chars.peek() == Some(&'"');
    if quoted {
        chars.next();
    }
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted => return (!out.is_empty()).then_some(out),
            ' ' if !quoted => break,
            '\\' if quoted => out.extend(chars.next()),
            '%' if chars.peek() == Some(&'%') => {
                chars.next();
                out.push('%');
            }
            _ => out.push(c),
        }
    }
    (!quoted && !out.is_empty()).then_some(out)
}

pub fn parse_exe(contents: &str) -> Option<String> {
    let mut in_entry = false;
    for line in contents.lines() {
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
        } else if in_entry {
            if let Some(value) = line.strip_prefix("Exec=") {
                return first_exec_arg(&unescape_value(value));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_home_wins_when_absolute() {
        assert_eq!(
            entry_path(Some("/cfg".into()), Some("/home/u".into())),
            Some(PathBuf::from("/cfg/autostart/deadlock-plus.desktop"))
        );
    }

    #[test]
    fn relative_or_empty_config_home_falls_back_to_dot_config() {
        let want = Some(PathBuf::from("/home/u/.config/autostart/deadlock-plus.desktop"));
        assert_eq!(entry_path(Some("rel/path".into()), Some("/home/u".into())), want);
        assert_eq!(entry_path(Some("".into()), Some("/home/u".into())), want);
        assert_eq!(entry_path(None, Some("/home/u".into())), want);
    }

    #[test]
    fn no_home_and_no_config_home_means_no_path() {
        assert_eq!(entry_path(None, None), None);
        assert_eq!(entry_path(None, Some("".into())), None);
    }

    #[test]
    fn entry_is_a_visible_application_that_starts_with_the_autostart_flag() {
        let text = render("/opt/dp/deadlock-plus", "--autostart");
        assert!(text.starts_with("[Desktop Entry]\n"));
        assert!(text.contains("\nType=Application\n"));
        assert!(text.contains("\nName=Deadlock+\n"));
        assert!(text.contains("\nExec=\"/opt/dp/deadlock-plus\" --autostart\n"));
        assert!(text.contains("\nTerminal=false\n"));
        assert!(text.contains("\nX-GNOME-Autostart-enabled=true\n"));
    }

    #[test]
    fn exe_round_trips_through_quoting() {
        for exe in [
            "/opt/dp/deadlock-plus",
            "/home/u/My Apps/Deadlock+.AppImage",
            "/tmp/a \"b\" $c `d` %e \\f/dp",
            "/weird/\u{e9}\u{4e2d}/dp",
        ] {
            assert_eq!(parse_exe(&render(exe, "--autostart")).as_deref(), Some(exe), "{exe}");
        }
    }

    #[test]
    fn quoting_follows_the_desktop_entry_spec() {
        let text = render("/a b/$x\\y%z", "--autostart");
        // Exec-level escapes for `$` and `\`, then the file-level string escape doubles each backslash.
        assert!(text.contains("Exec=\"/a b/\\\\$x\\\\\\\\y%%z\" --autostart\n"), "{text}");
    }

    #[test]
    fn unreadable_entries_have_no_exe() {
        assert_eq!(parse_exe(""), None);
        assert_eq!(parse_exe("[Desktop Entry]\nName=x\n"), None);
        assert_eq!(parse_exe("[Desktop Entry]\nExec=\n"), None);
    }

    #[test]
    fn unquoted_exec_is_read_up_to_the_first_space() {
        assert_eq!(parse_exe("[Desktop Entry]\nExec=/usr/bin/dp --autostart\n").as_deref(), Some("/usr/bin/dp"));
    }

    #[test]
    fn crlf_line_endings_are_tolerated() {
        assert_eq!(parse_exe("[Desktop Entry]\r\nExec=\"/x/dp\" --autostart\r\n").as_deref(), Some("/x/dp"));
    }
}
