use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ClientKind {
    Stable,
    Ptb,
    Canary,
    Other,
}

impl ClientKind {
    pub const ALL: [ClientKind; 4] = [ClientKind::Stable, ClientKind::Ptb, ClientKind::Canary, ClientKind::Other];

    pub fn all() -> HashSet<ClientKind> {
        Self::ALL.into_iter().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pipe {
    pub index: u8,
    pub kind: Option<ClientKind>,
    pub(crate) path: Option<std::path::PathBuf>,
}

impl Pipe {
    pub fn new(index: u8, kind: Option<ClientKind>) -> Self {
        Self { index, kind, path: None }
    }

    pub fn effective_kind(&self) -> ClientKind {
        self.kind.unwrap_or(ClientKind::Other)
    }
}

/// Normalizes one name or path component (extension, sandbox prefixes, separators) and matches it
/// against the known Discord channel names. Anything else, including other Electron builds
/// like Vesktop, is `Other`.
fn classify_component(name: &str) -> ClientKind {
    let lower = name.to_ascii_lowercase();
    let mut s = lower.as_str();
    for suffix in [".exe", ".app"] {
        s = s.strip_suffix(suffix).unwrap_or(s);
    }
    for prefix in ["com.discordapp.", "snap."] {
        s = s.strip_prefix(prefix).unwrap_or(s);
    }
    let squashed: String = s.chars().filter(|c| !matches!(c, ' ' | '-' | '_')).collect();
    match squashed.as_str() {
        "discord" => ClientKind::Stable,
        "discordptb" => ClientKind::Ptb,
        "discordcanary" => ClientKind::Canary,
        _ => ClientKind::Other,
    }
}

pub fn classify_exe_name(name: &str) -> ClientKind {
    classify_component(name)
}

pub fn classify_bundle(name: &str) -> ClientKind {
    classify_component(name)
}

/// Checks components from the file name outward, so the executable name wins over the install
/// folder, and the first recognizable component decides.
pub fn classify_path(path: &str) -> ClientKind {
    path.rsplit(['/', '\\']).map(classify_component).find(|k| *k != ClientKind::Other).unwrap_or(ClientKind::Other)
}

pub fn select_targets(selected: &HashSet<ClientKind>, pipes: &[Pipe]) -> Vec<Pipe> {
    pipes.iter().filter(|p| selected.contains(&p.effective_kind())).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ClientKind::*;

    #[test]
    fn exe_names() {
        assert_eq!(classify_exe_name("Discord.exe"), Stable);
        assert_eq!(classify_exe_name("discord.EXE"), Stable);
        assert_eq!(classify_exe_name("DiscordPTB.exe"), Ptb);
        assert_eq!(classify_exe_name("DiscordCanary.exe"), Canary);
        assert_eq!(classify_exe_name("DiscordDevelopment.exe"), Other);
        assert_eq!(classify_exe_name("Vesktop.exe"), Other);
        assert_eq!(classify_exe_name("arRPC-bridge.exe"), Other);
        assert_eq!(classify_exe_name(""), Other);
    }

    #[test]
    fn linux_exe_names() {
        assert_eq!(classify_exe_name("Discord"), Stable);
        assert_eq!(classify_exe_name("discord"), Stable);
        assert_eq!(classify_exe_name("DiscordPTB"), Ptb);
        assert_eq!(classify_exe_name("discord-ptb"), Ptb);
        assert_eq!(classify_exe_name("discord-canary"), Canary);
    }

    #[test]
    fn windows_paths() {
        assert_eq!(classify_path(r"C:\Users\a\AppData\Local\Discord\app-1.0.9\Discord.exe"), Stable);
        assert_eq!(classify_path(r"C:\Users\a\AppData\Local\DiscordPTB\app-1.0.1\DiscordPTB.exe"), Ptb);
        assert_eq!(classify_path(r"C:\Users\a\AppData\Local\DiscordCanary\app-1.0.1\DiscordCanary.exe"), Canary);
        assert_eq!(classify_path(r"C:\Users\a\AppData\Local\Discord\app-1.0.9\Update.exe"), Stable);
        assert_eq!(classify_path(r"C:\Users\a\AppData\Local\DiscordCanary\Update.exe"), Canary);
        assert_eq!(classify_path(r"C:\Users\a\AppData\Local\Vesktop\Vesktop.exe"), Other);
    }

    #[test]
    fn exe_name_beats_folder() {
        assert_eq!(classify_path(r"C:\Discord\DiscordPTB.exe"), Ptb);
    }

    #[test]
    fn unix_paths() {
        assert_eq!(classify_path("/usr/share/discord/Discord"), Stable);
        assert_eq!(classify_path("/opt/discord-ptb/DiscordPTB"), Ptb);
        assert_eq!(classify_path("/opt/discord-canary/DiscordCanary"), Canary);
        assert_eq!(classify_path("/usr/bin/vesktop"), Other);
    }

    #[test]
    fn sandbox_socket_paths() {
        assert_eq!(classify_path("/run/user/1000/app/com.discordapp.Discord/discord-ipc-0"), Stable);
        assert_eq!(classify_path("/run/user/1000/app/com.discordapp.DiscordCanary/discord-ipc-1"), Canary);
        assert_eq!(classify_path("/run/user/1000/app/com.discordapp.DiscordPTB/discord-ipc-0"), Ptb);
        assert_eq!(classify_path("/run/user/1000/snap.discord/discord-ipc-0"), Stable);
        assert_eq!(classify_path("/run/user/1000/snap.discord-canary/discord-ipc-0"), Canary);
        assert_eq!(classify_path("/run/user/1000/discord-ipc-0"), Other);
        assert_eq!(classify_path("/run/user/1000/app/dev.vencord.Vesktop/discord-ipc-0"), Other);
    }

    #[test]
    fn bundles() {
        assert_eq!(classify_bundle("Discord.app"), Stable);
        assert_eq!(classify_bundle("Discord PTB.app"), Ptb);
        assert_eq!(classify_bundle("Discord Canary.app"), Canary);
        assert_eq!(classify_bundle("Vesktop.app"), Other);
        assert_eq!(classify_path("/Applications/Discord PTB.app/Contents/MacOS/Discord PTB"), Ptb);
        assert_eq!(classify_path("/Applications/Discord.app/Contents/MacOS/Discord"), Stable);
    }

    fn pipes() -> Vec<Pipe> {
        vec![
            Pipe::new(0, Some(Stable)),
            Pipe::new(1, Some(Canary)),
            Pipe::new(2, None),
            Pipe::new(3, Some(Other)),
            Pipe::new(4, Some(Stable)),
        ]
    }

    fn indexes(p: Vec<Pipe>) -> Vec<u8> {
        p.into_iter().map(|p| p.index).collect()
    }

    #[test]
    fn all_selected_returns_everything() {
        assert_eq!(indexes(select_targets(&ClientKind::all(), &pipes())), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn same_kind_pipes_all_receive() {
        let sel: HashSet<_> = [Stable].into();
        assert_eq!(indexes(select_targets(&sel, &pipes())), vec![0, 4]);
    }

    #[test]
    fn unlabelled_counts_as_other() {
        let sel: HashSet<_> = [Other].into();
        assert_eq!(indexes(select_targets(&sel, &pipes())), vec![2, 3]);
    }

    #[test]
    fn missing_selected_kind_is_skipped() {
        let sel: HashSet<_> = [Ptb, Canary].into();
        assert_eq!(indexes(select_targets(&sel, &pipes())), vec![1]);
    }

    #[test]
    fn empty_selection_targets_nothing() {
        assert!(select_targets(&HashSet::new(), &pipes()).is_empty());
    }

    #[test]
    fn serde_names() {
        assert_eq!(serde_json::to_string(&Ptb).unwrap(), "\"ptb\"");
        assert_eq!(serde_json::from_str::<ClientKind>("\"canary\"").unwrap(), Canary);
    }
}
