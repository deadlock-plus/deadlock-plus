//! macOS: one pf anchor owned by Deadlock+, replaced as a whole on every change. The stock ruleset already
//! evaluates anchors under `com.apple/`, so nothing in `/etc/pf.conf` needs editing. Loading needs
//! administrator rights, asked for through the system's own password dialog.

use std::net::IpAddr;
use std::process::Command;

use super::ruleset::Backend;
use super::FirewallRuleSpec;

const ANCHOR: &str = "com.apple/deadlock_plus";
const LABEL_PREFIX: &str = "deadlock_plus_";

pub struct Pf;

fn safe_id(id: &str) -> bool {
    !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

/// One rule per group. Naming tcp and udp (not "all") leaves ICMP open, so ping still works.
fn render(groups: &[FirewallRuleSpec]) -> String {
    let mut script = String::new();
    for group in groups.iter().filter(|g| safe_id(&g.group_id)) {
        let ips: Vec<String> =
            group.relay_ips.iter().filter_map(|ip| ip.parse::<IpAddr>().ok()).map(|ip| ip.to_string()).collect();
        if !ips.is_empty() {
            script.push_str(&format!(
                "block drop out quick proto {{ tcp udp }} to {{ {} }} label \"{LABEL_PREFIX}{}\"\n",
                ips.join(" "),
                group.group_id
            ));
        }
    }
    script
}

/// Writing an empty file to the anchor flushes it, which is how "no groups" is applied.
fn shell_command(script_path: &str) -> String {
    format!("/sbin/pfctl -e >/dev/null 2>&1; /sbin/pfctl -a {ANCHOR} -f '{script_path}'")
}

fn applescript(shell: &str) -> String {
    format!("do shell script \"{}\" with administrator privileges", shell.replace('\\', "\\\\").replace('"', "\\\""))
}

fn explain_failure(stderr: &str) -> String {
    if stderr.contains("-128") || stderr.to_lowercase().contains("user canceled") {
        "Authorization was cancelled.".into()
    } else {
        format!("pfctl failed: {}", stderr.trim())
    }
}

impl Backend for Pf {
    fn render(&self, groups: &[FirewallRuleSpec]) -> String {
        render(groups)
    }

    fn apply(&self, script: &str) -> Result<(), String> {
        // A fresh private folder, so another local user cannot swap the file between our write and root's read.
        let dir = std::env::temp_dir().join(format!("deadlock-plus-pf-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        create_private_dir(&dir).map_err(|e| format!("could not prepare the rules: {e}"))?;
        let path = dir.join("rules.conf");
        let result =
            std::fs::write(&path, script).map_err(|e| format!("could not prepare the rules: {e}")).and_then(|()| {
                let path = path.to_str().filter(|p| !p.contains('\'')).ok_or("unsafe temporary path")?;
                let output = Command::new("osascript")
                    .args(["-e", &applescript(&shell_command(path))])
                    .output()
                    .map_err(|e| format!("could not run osascript: {e}"))?;
                if output.status.success() {
                    Ok(())
                } else {
                    Err(explain_failure(&String::from_utf8_lossy(&output.stderr)))
                }
            });
        let _ = std::fs::remove_dir_all(&dir);
        result
    }

    fn boot_id(&self) -> Option<String> {
        let output = Command::new("sysctl").args(["-n", "kern.boottime"]).output().ok()?;
        output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

#[cfg(unix)]
fn create_private_dir(dir: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    std::fs::DirBuilder::new().mode(0o700).create(dir)
}

#[cfg(not(unix))]
fn create_private_dir(dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(id: &str, ips: &[&str]) -> FirewallRuleSpec {
        FirewallRuleSpec {
            group_id: id.into(),
            description: id.into(),
            relay_ips: ips.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn no_groups_is_an_empty_ruleset() {
        assert_eq!(render(&[]), "");
    }

    #[test]
    fn a_group_becomes_one_labelled_outbound_block() {
        assert_eq!(
            render(&[spec("fra", &["155.133.226.1", "2001:db8::1"])]),
            "block drop out quick proto { tcp udp } to { 155.133.226.1 2001:db8::1 } label \"deadlock_plus_fra\"\n"
        );
    }

    #[test]
    fn anything_that_is_not_an_address_or_a_safe_id_is_left_out() {
        let script = render(&[spec("fra", &["1.1.1.1", "1.1.1.1 } pass all #"]), spec("bad\" x", &["2.2.2.2"])]);
        assert_eq!(script.lines().count(), 1);
        assert!(script.contains("{ 1.1.1.1 }"));
        assert!(!script.contains("pass"));
        assert!(!script.contains("2.2.2.2"));
    }

    #[test]
    fn the_admin_command_targets_the_anchor_and_survives_applescript_quoting() {
        let shell = shell_command("/tmp/x/rules.conf");
        assert!(shell.contains("-a com.apple/deadlock_plus -f '/tmp/x/rules.conf'"));
        assert!(applescript(&shell).starts_with("do shell script \"/sbin/pfctl -e"));
        assert_eq!(
            applescript("echo \"hi\" \\ x"),
            "do shell script \"echo \\\"hi\\\" \\\\ x\" with administrator privileges"
        );
    }

    #[test]
    fn a_dismissed_password_dialog_is_explained() {
        assert!(explain_failure("execution error: User canceled. (-128)").contains("cancelled"));
        assert_eq!(explain_failure(" boom "), "pfctl failed: boom");
    }
}
