use std::ffi::OsString;
use std::path::PathBuf;

pub const LABEL: &str = "app.deadlockplus.autostart";

pub fn plist_path(home: Option<OsString>) -> Option<PathBuf> {
    let home = home.filter(|h| !h.is_empty())?;
    Some(PathBuf::from(home).join("Library").join("LaunchAgents").join(format!("{LABEL}.plist")))
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&amp;", "&")
}

/// The agent is only written to disk, never bootstrapped with `launchctl`, so it first runs at the next login.
pub fn render(exe: &str, launch_arg: &str) -> String {
    let exe = xml_escape(exe);
    let launch_arg = xml_escape(launch_arg);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{exe}</string>
    <string>{launch_arg}</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>ProcessType</key>
  <string>Interactive</string>
</dict>
</plist>
"#
    )
}

pub fn parse_exe(contents: &str) -> Option<String> {
    let after_key = &contents[contents.find("<key>ProgramArguments</key>")?..];
    let start = after_key.find("<string>")? + "<string>".len();
    let end = after_key[start..].find("</string>")? + start;
    let exe = xml_unescape(after_key[start..end].trim());
    (!exe.is_empty()).then_some(exe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_lives_in_the_user_launch_agents_folder() {
        assert_eq!(
            plist_path(Some("/Users/u".into())),
            Some(PathBuf::from("/Users/u/Library/LaunchAgents/app.deadlockplus.autostart.plist"))
        );
        assert_eq!(plist_path(None), None);
        assert_eq!(plist_path(Some("".into())), None);
    }

    #[test]
    fn agent_runs_at_login_with_the_autostart_flag() {
        let text = render("/Applications/Deadlock+.app/Contents/MacOS/deadlock-plus", "--autostart");
        assert!(text.contains("<key>Label</key>\n  <string>app.deadlockplus.autostart</string>"));
        assert!(text.contains("<key>RunAtLoad</key>\n  <true/>"));
        assert!(text.contains(
            "<string>/Applications/Deadlock+.app/Contents/MacOS/deadlock-plus</string>\n    <string>--autostart</string>"
        ));
    }

    #[test]
    fn exe_is_xml_escaped_and_round_trips() {
        let exe = "/Users/a&b/<x> \"y\"/dp";
        let text = render(exe, "--autostart");
        assert!(text.contains("/Users/a&amp;b/&lt;x&gt; &quot;y&quot;/dp"));
        assert_eq!(parse_exe(&text).as_deref(), Some(exe));
    }

    #[test]
    fn unreadable_plists_have_no_exe() {
        assert_eq!(parse_exe(""), None);
        assert_eq!(parse_exe("<plist><dict><key>Label</key><string>x</string></dict></plist>"), None);
    }

    #[test]
    fn program_arguments_are_read_not_the_label() {
        let text = render("/x/dp", "--autostart");
        assert_eq!(parse_exe(&text).as_deref(), Some("/x/dp"));
    }
}
