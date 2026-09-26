//! Blocks/unblocks Steam Datagram Relay IPs via the Windows Filtering Platform
//! (Windows Firewall) COM API, mirroring what server-picker-x does through .NET's
//! `NetFwTypeLib`. Requires the process to be running elevated (see `build.rs`).

use windows::core::BSTR;
use windows::Win32::Foundation::VARIANT_TRUE;
use windows::Win32::NetworkManagement::WindowsFirewall::{
    INetFwPolicy2, INetFwRule, INetFwRules, NetFwPolicy2, NetFwRule, NET_FW_ACTION_BLOCK, NET_FW_RULE_DIR_OUT,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
};

const RULE_NAME_PREFIX: &str = "deadlock_plus_";
const PROFILES_ALL: i32 = i32::MAX;
const PROTOCOL_TCP: i32 = 6;
const PROTOCOL_UDP: i32 = 17;

pub struct FirewallRuleSpec {
    pub group_id: String,
    pub description: String,
    pub relay_ips: Vec<String>,
}

struct ComGuard {
    initialized: bool,
}

impl ComGuard {
    fn new() -> Self {
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        Self { initialized: hr.is_ok() }
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { CoUninitialize() };
        }
    }
}

/// Blocking TCP and UDP separately (instead of protocol Any) leaves ICMP open, so
/// ping still works against a blocked region.
fn block_rules(group_id: &str) -> Vec<(String, i32)> {
    vec![
        (format!("{RULE_NAME_PREFIX}{group_id}_tcp"), PROTOCOL_TCP),
        (format!("{RULE_NAME_PREFIX}{group_id}_udp"), PROTOCOL_UDP),
    ]
}

/// Earlier builds created one protocol-Any rule per group; it must still be
/// recognised and cleaned up so it doesn't silently block ICMP forever.
fn owned_rule_names(group_id: &str) -> Vec<String> {
    let mut names: Vec<String> = block_rules(group_id).into_iter().map(|(n, _)| n).collect();
    names.push(format!("{RULE_NAME_PREFIX}{group_id}"));
    names
}

fn open_rules() -> windows::core::Result<INetFwRules> {
    let result = unsafe {
        let policy: INetFwPolicy2 = CoCreateInstance(&NetFwPolicy2, None, CLSCTX_ALL)?;
        policy.Rules()
    };
    if let Err(e) = &result {
        log::error!("could not open the Windows Firewall rules: {e}");
    }
    result
}

fn remove_rule_if_present(rules: &INetFwRules, name: &BSTR) {
    unsafe {
        // Item() errors when the rule doesn't exist; that's an expected, not exceptional, outcome here.
        while rules.Item(name).is_ok() {
            if let Err(e) = rules.Remove(name) {
                log::warn!("could not remove firewall rule {name}: {e}");
                break;
            }
        }
    }
}

pub fn block_groups(specs: &[FirewallRuleSpec]) -> Result<(), String> {
    let _com = ComGuard::new();
    let rules = open_rules().map_err(|e| e.to_string())?;

    for spec in specs {
        for name in owned_rule_names(&spec.group_id) {
            remove_rule_if_present(&rules, &BSTR::from(name));
        }

        for (name, protocol) in block_rules(&spec.group_id) {
            unsafe {
                let rule: INetFwRule = CoCreateInstance(&NetFwRule, None, CLSCTX_ALL).map_err(|e| e.to_string())?;
                rule.SetName(&BSTR::from(name)).map_err(|e| e.to_string())?;
                rule.SetDescription(&BSTR::from(spec.description.as_str())).map_err(|e| e.to_string())?;
                rule.SetDirection(NET_FW_RULE_DIR_OUT).map_err(|e| e.to_string())?;
                rule.SetAction(NET_FW_ACTION_BLOCK).map_err(|e| e.to_string())?;
                rule.SetProtocol(protocol).map_err(|e| e.to_string())?;
                rule.SetRemoteAddresses(&BSTR::from(spec.relay_ips.join(",").as_str())).map_err(|e| e.to_string())?;
                rule.SetProfiles(PROFILES_ALL).map_err(|e| e.to_string())?;
                rule.SetEnabled(VARIANT_TRUE).map_err(|e| e.to_string())?;

                rules.Add(&rule).map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}

pub fn unblock_groups(group_ids: &[String]) -> Result<(), String> {
    let _com = ComGuard::new();
    let rules = open_rules().map_err(|e| e.to_string())?;

    for group_id in group_ids {
        for name in owned_rule_names(group_id) {
            remove_rule_if_present(&rules, &BSTR::from(name));
        }
    }

    Ok(())
}

/// Returns the subset of `group_ids` that currently have an active block rule.
/// Used to reconcile UI state with the real firewall on startup (e.g. if the
/// app's settings file was deleted but rules from a previous run still exist).
pub fn list_blocked(group_ids: &[String]) -> Result<Vec<String>, String> {
    let _com = ComGuard::new();
    let rules = open_rules().map_err(|e| e.to_string())?;

    let blocked = group_ids
        .iter()
        .filter(|id| owned_rule_names(id).iter().any(|n| unsafe { rules.Item(&BSTR::from(n.as_str())).is_ok() }))
        .cloned()
        .collect();

    Ok(blocked)
}

pub struct ExistingBlockRule {
    pub name: String,
    pub remote_ips: Vec<String>,
}

/// Looks up enabled outbound block rules by exact name and returns their remote IPs.
/// Used to detect rules created by other tools; missing rules are skipped.
pub fn read_block_rules(names: &[String]) -> Result<Vec<ExistingBlockRule>, String> {
    let _com = ComGuard::new();
    let rules = open_rules().map_err(|e| e.to_string())?;
    let mut found = Vec::new();

    for name in names {
        let Ok(rule) = (unsafe { rules.Item(&BSTR::from(name.as_str())) }) else {
            continue;
        };
        let is_block = unsafe { rule.Action().map(|a| a == NET_FW_ACTION_BLOCK).unwrap_or(false) };
        let enabled = unsafe { rule.Enabled().map(|e| e.as_bool()).unwrap_or(false) };
        if !is_block || !enabled {
            continue;
        }

        let Ok(addresses) = (unsafe { rule.RemoteAddresses() }) else {
            continue;
        };
        let remote_ips = addresses
            .to_string()
            .split(',')
            .map(|a| a.split('/').next().unwrap_or("").trim().to_string())
            .filter(|a| !a.is_empty())
            .collect();

        found.push(ExistingBlockRule { name: name.clone(), remote_ips });
    }

    Ok(found)
}

pub fn remove_rules_by_name(names: &[String]) -> Result<(), String> {
    let _com = ComGuard::new();
    let rules = open_rules().map_err(|e| e.to_string())?;
    for name in names {
        remove_rule_if_present(&rules, &BSTR::from(name.as_str()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_rules_split_tcp_and_udp() {
        let rules = block_rules("fra");
        assert_eq!(
            rules,
            vec![
                ("deadlock_plus_fra_tcp".to_string(), PROTOCOL_TCP),
                ("deadlock_plus_fra_udp".to_string(), PROTOCOL_UDP)
            ]
        );
    }

    #[test]
    fn owned_rule_names_include_the_single_any_rule_from_older_builds() {
        let names = owned_rule_names("fra");
        assert!(names.contains(&"deadlock_plus_fra".to_string()));
        assert!(names.contains(&"deadlock_plus_fra_tcp".to_string()));
        assert!(names.contains(&"deadlock_plus_fra_udp".to_string()));
    }
}
