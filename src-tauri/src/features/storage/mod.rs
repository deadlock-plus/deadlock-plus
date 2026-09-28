use std::path::{Component, Path, PathBuf};
use ts_rs::TS;

use serde::{Deserialize, Serialize};

const DEADLOCK_APP_ID: &str = "1422450";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
pub enum EntryId {
    Replays,
    Addons,
    AddonsBackups,
    ConfigBackups,
    GameinfoBackups,
    HeroPresenceCache,
    ShaderCache,
    ConsoleLog,
    VoiceBanBackups,
    AppData,
    Logs,
}

pub const ALL_ENTRIES: [EntryId; 11] = [
    EntryId::Replays,
    EntryId::Addons,
    EntryId::AddonsBackups,
    EntryId::ConfigBackups,
    EntryId::GameinfoBackups,
    EntryId::HeroPresenceCache,
    EntryId::ShaderCache,
    EntryId::ConsoleLog,
    EntryId::VoiceBanBackups,
    EntryId::AppData,
    EntryId::Logs,
];

#[derive(Debug, Default, Clone)]
pub struct Roots {
    pub install: Option<PathBuf>,
    pub userdata: Option<PathBuf>,
    pub app_data: Option<PathBuf>,
    pub logs: Option<PathBuf>,
}

impl Roots {
    fn citadel(&self) -> Option<PathBuf> {
        Some(self.install.as_ref()?.join("game").join("citadel"))
    }

    fn remote(&self) -> Option<PathBuf> {
        Some(self.userdata.as_ref()?.join(DEADLOCK_APP_ID).join("remote"))
    }

    fn allowed(&self) -> Vec<&Path> {
        [&self.install, &self.userdata, &self.app_data, &self.logs]
            .into_iter()
            .flatten()
            .map(PathBuf::as_path)
            .collect()
    }
}

struct Item {
    path: PathBuf,
    skip: &'static [&'static str],
}

impl Item {
    fn whole(path: PathBuf) -> Self {
        Self { path, skip: &[] }
    }
}

/// Only things the game or this app regenerates on its own. Everything under `addons` and the
/// `gameinfo.gi*` files belong to the game or the mod manager and are never offered.
pub fn is_clearable(id: EntryId) -> bool {
    matches!(id, EntryId::ShaderCache | EntryId::ConsoleLog | EntryId::VoiceBanBackups | EntryId::Logs)
}

/// Matches `<name>.backup-<unix seconds>` with an optional `-<n>` suffix, as written by the
/// mute list editor. The name part must be present and the suffix purely numeric, so a user's
/// own `voice_ban.dt.backup-old` is never picked up.
pub fn is_our_backup(name: &str) -> bool {
    let Some((base, suffix)) = name.rsplit_once(".backup-") else {
        return false;
    };
    let mut parts = suffix.splitn(2, '-');
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    !base.is_empty() && parts.next().is_some_and(digits) && parts.next().is_none_or(digits)
}

/// Lexical check: the path must sit strictly under `root` with no `..` segments.
pub fn within(root: &Path, path: &Path) -> bool {
    let clean = !path.components().any(|c| matches!(c, Component::ParentDir));
    clean && path != root && path.starts_with(root)
}

fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
}

/// Symlinks and junctions count as zero and are not entered: the game's `replays` link would
/// otherwise be counted twice, and a link can point outside the folder being measured.
/// `skip_top_level` names direct children left out of the total.
pub fn dir_size(path: &Path, skip_top_level: &[&str]) -> u64 {
    fn walk(path: &Path, skip: &[&str]) -> u64 {
        let Ok(meta) = std::fs::symlink_metadata(path) else {
            return 0;
        };
        if meta.file_type().is_symlink() {
            return 0;
        }
        if meta.is_file() {
            return meta.len();
        }
        let Ok(read) = std::fs::read_dir(path) else {
            return 0;
        };
        read.filter_map(Result::ok)
            .filter(|e| !skip.iter().any(|s| e.file_name() == *s))
            .map(|e| walk(&e.path(), &[]))
            .sum()
    }
    walk(path, skip_top_level)
}

fn children_matching(dir: &Path, keep: impl Fn(&str, bool) -> bool) -> Vec<PathBuf> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    read.filter_map(Result::ok)
        .filter(|e| {
            let is_dir = e.file_type().is_ok_and(|t| t.is_dir());
            keep(&e.file_name().to_string_lossy(), is_dir)
        })
        .map(|e| e.path())
        .collect()
}

fn items(id: EntryId, roots: &Roots) -> Vec<Item> {
    let citadel = roots.citadel();
    let in_citadel = |name: &str| citadel.as_ref().map(|c| Item::whole(c.join(name)));
    match id {
        EntryId::Replays => in_citadel("addons").map(|i| Item::whole(i.path.join("replays"))).into_iter().collect(),
        EntryId::Addons => {
            in_citadel("addons").map(|i| Item { path: i.path, skip: &["replays"] }).into_iter().collect()
        }
        EntryId::AddonsBackups => in_citadel("addons-backups").into_iter().collect(),
        EntryId::HeroPresenceCache => in_citadel("hero_presence_cache.json").into_iter().collect(),
        EntryId::ShaderCache => in_citadel("shadercache").into_iter().collect(),
        EntryId::ConsoleLog => in_citadel("console.log").into_iter().collect(),
        EntryId::ConfigBackups => citadel
            .iter()
            .flat_map(|c| children_matching(c, |n, dir| dir && n.starts_with("_config_backup_")))
            .map(Item::whole)
            .collect(),
        EntryId::GameinfoBackups => citadel
            .iter()
            .flat_map(|c| children_matching(c, |n, dir| !dir && n.starts_with("gameinfo.gi.bak")))
            .map(Item::whole)
            .collect(),
        EntryId::VoiceBanBackups => roots
            .remote()
            .iter()
            .flat_map(|r| children_matching(r, |n, dir| !dir && is_our_backup(n)))
            .map(Item::whole)
            .collect(),
        EntryId::AppData => roots.app_data.clone().map(Item::whole).into_iter().collect(),
        EntryId::Logs => roots.logs.clone().map(Item::whole).into_iter().collect(),
    }
}

/// Folder shown and opened for "reveal": the thing itself for single-target entries, the folder
/// holding the matches for the backup entries.
pub fn location(id: EntryId, roots: &Roots) -> Option<PathBuf> {
    match id {
        EntryId::ConfigBackups | EntryId::GameinfoBackups => roots.citadel(),
        EntryId::VoiceBanBackups => roots.remote(),
        _ => items(id, roots).into_iter().next().map(|i| i.path),
    }
}

pub fn entry_size(id: EntryId, roots: &Roots) -> u64 {
    items(id, roots).iter().map(|i| dir_size(&i.path, i.skip)).sum()
}

/// What "clear" would remove. Shader cache clears the contents and keeps the folder itself, so
/// the game does not have to recreate it.
fn clear_paths(id: EntryId, roots: &Roots) -> Result<Vec<PathBuf>, String> {
    if !is_clearable(id) {
        return Err("This can't be cleared from here.".into());
    }
    let citadel = roots.citadel();
    let paths: Vec<PathBuf> = match id {
        EntryId::ShaderCache => {
            citadel.iter().flat_map(|c| children_matching(&c.join("shadercache"), |_, _| true)).collect()
        }
        EntryId::ConsoleLog => citadel.iter().map(|c| c.join("console.log")).filter(|p| p.is_file()).collect(),
        // The active `latest.log`/`debug.log`/`trace.log` stay open for the running process; only
        // already-rolled archives can be removed on demand.
        EntryId::Logs => {
            roots.logs.iter().flat_map(|d| children_matching(d, |n, is_dir| !is_dir && n.ends_with(".log.gz"))).collect()
        }
        _ => items(id, roots).into_iter().map(|i| i.path).collect(),
    };
    let allowed = roots.allowed();
    Ok(paths.into_iter().filter(|p| !is_symlink(p) && allowed.iter().any(|r| within(r, p))).collect())
}

#[derive(Debug, Default, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ClearReport {
    pub freed_bytes: u64,
    pub removed: usize,
    pub failed: Vec<String>,
}

pub fn clear(id: EntryId, roots: &Roots) -> Result<ClearReport, String> {
    let mut report = ClearReport::default();
    for path in clear_paths(id, roots)? {
        let size = dir_size(&path, &[]);
        let result = if path.is_dir() { std::fs::remove_dir_all(&path) } else { std::fs::remove_file(&path) };
        match result {
            Ok(()) => {
                report.freed_bytes += size;
                report.removed += 1;
            }
            Err(e) => {
                let name = path.file_name().unwrap_or_default().to_string_lossy();
                log::warn!("could not clear {name}: {e}");
                report.failed.push(format!("{name}: {e}"));
            }
        }
    }
    log::info!(
        "storage cleared ({id:?}): {} removed, {} bytes freed, {} failed",
        report.removed,
        report.freed_bytes,
        report.failed.len()
    );
    Ok(report)
}

pub fn game_install_dir() -> Option<PathBuf> {
    let steam = steamlocate::SteamDir::locate().ok()?;
    let (app, library) = steam.find_app(DEADLOCK_APP_ID.parse().ok()?).ok()??;
    Some(library.resolve_app_dir(&app))
}

fn current_roots(app: &tauri::AppHandle) -> Roots {
    use tauri::Manager;
    Roots {
        install: game_install_dir(),
        userdata: crate::features::steam_account::current_account().and_then(|a| a.userdata_dir).map(PathBuf::from),
        app_data: app.path().app_data_dir().ok(),
        logs: app.path().app_log_dir().ok(),
    }
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct EntryInfo {
    pub id: EntryId,
    pub path: Option<String>,
    pub clearable: bool,
}

pub mod commands {
    use super::*;

    #[tauri::command]
    pub fn storage_entries(app: tauri::AppHandle) -> Vec<EntryInfo> {
        let roots = current_roots(&app);
        ALL_ENTRIES
            .into_iter()
            .map(|id| EntryInfo {
                id,
                path: location(id, &roots).filter(|p| p.exists()).map(|p| p.to_string_lossy().into_owned()),
                clearable: is_clearable(id),
            })
            .collect()
    }

    #[tauri::command]
    pub async fn storage_entry_size(app: tauri::AppHandle, id: EntryId) -> Result<u64, String> {
        let roots = current_roots(&app);
        tauri::async_runtime::spawn_blocking(move || entry_size(id, &roots)).await.map_err(|e| e.to_string())
    }

    /// The frontend sends an entry id, never a path.
    #[tauri::command]
    pub fn storage_reveal(app: tauri::AppHandle, id: EntryId) -> Result<(), String> {
        let path = location(id, &current_roots(&app)).filter(|p| p.exists()).ok_or("That location doesn't exist.")?;
        let mut cmd = std::process::Command::new("explorer");
        if path.is_file() {
            cmd.arg(format!("/select,{}", path.display()));
        } else {
            cmd.arg(&path);
        }
        cmd.spawn().map_err(|e| {
            log::error!("could not reveal a storage location ({id:?}): {e}");
            e.to_string()
        })?;
        Ok(())
    }

    #[tauri::command]
    pub async fn storage_clear(app: tauri::AppHandle, id: EntryId) -> Result<ClearReport, String> {
        if !is_clearable(id) {
            return Err("This can't be cleared from here.".into());
        }
        if crate::features::voice_bans::game_running() {
            log::warn!("storage clear refused ({id:?}): Deadlock is running");
            return Err("Deadlock is running. Close the game before clearing files.".into());
        }
        let roots = current_roots(&app);
        tauri::async_runtime::spawn_blocking(move || clear(id, &roots)).await.map_err(|e| e.to_string())?
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-storage-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, bytes: usize) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, vec![0u8; bytes]).unwrap();
    }

    /// A fake install, account folder and app data folder with a known byte count in each place.
    fn fixture(name: &str) -> (PathBuf, Roots) {
        let base = temp_dir(name);
        let citadel = base.join("install").join("game").join("citadel");
        write(&citadel.join("shadercache").join("a.bin"), 10);
        write(&citadel.join("shadercache").join("sub").join("b.bin"), 20);
        write(&citadel.join("console.log"), 5);
        write(&citadel.join("hero_presence_cache.json"), 3);
        write(&citadel.join("gameinfo.gi"), 100);
        write(&citadel.join("gameinfo.gi.bak"), 7);
        write(&citadel.join("gameinfo.gi.bak2"), 8);
        write(&citadel.join("_config_backup_20260813").join("x.cfg"), 11);
        write(&citadel.join("pak01_dir.vpk"), 1000);
        write(&citadel.join("addons").join("replays").join("1.dem"), 400);
        write(&citadel.join("addons").join("pak01_dir.vpk"), 60);
        write(&citadel.join("addons-backups").join("old.vpk"), 30);
        let remote = base.join("userdata").join("42").join(DEADLOCK_APP_ID).join("remote");
        write(&remote.join("voice_ban.dt"), 50);
        write(&remote.join("voice_ban.dt.backup-100"), 4);
        write(&remote.join("voice_ban.dt.backup-100-2"), 6);
        write(&remote.join("voice_ban.dt.backup-old"), 9);
        write(&base.join("appdata").join("pins.json"), 2);
        write(&base.join("logs").join("debug.log"), 40);
        write(&base.join("logs").join("2026-09-01-1.log.gz"), 15);
        let roots = Roots {
            install: Some(base.join("install")),
            userdata: Some(base.join("userdata").join("42")),
            app_data: Some(base.join("appdata")),
            logs: Some(base.join("logs")),
        };
        (base, roots)
    }

    #[test]
    fn dir_size_adds_nested_files() {
        let (base, _) = fixture("nested");
        assert_eq!(dir_size(&base.join("install").join("game").join("citadel").join("shadercache"), &[]), 30);
    }

    #[test]
    fn dir_size_of_a_file_is_its_length_and_of_a_missing_path_is_zero() {
        let (base, _) = fixture("single");
        let citadel = base.join("install").join("game").join("citadel");
        assert_eq!(dir_size(&citadel.join("console.log"), &[]), 5);
        assert_eq!(dir_size(&citadel.join("nope"), &[]), 0);
    }

    #[test]
    fn dir_size_leaves_out_named_top_level_children_only() {
        let base = temp_dir("skip");
        write(&base.join("replays").join("a"), 100);
        write(&base.join("keep").join("replays").join("a"), 7);
        write(&base.join("keep").join("b"), 1);
        assert_eq!(dir_size(&base, &["replays"]), 8);
    }

    #[test]
    fn dir_size_does_not_follow_symlinks() {
        let base = temp_dir("link");
        write(&base.join("real").join("big"), 500);
        write(&base.join("outer").join("small"), 1);
        #[cfg(windows)]
        let made = std::os::windows::fs::symlink_dir(base.join("real"), base.join("outer").join("link")).is_ok();
        #[cfg(unix)]
        let made = std::os::unix::fs::symlink(base.join("real"), base.join("outer").join("link")).is_ok();
        if !made {
            eprintln!("symlinks unavailable here; skipping");
            return;
        }
        assert_eq!(dir_size(&base.join("outer"), &[]), 1);
    }

    #[test]
    fn addons_size_excludes_replays_so_nothing_is_counted_twice() {
        let (_, roots) = fixture("addons");
        assert_eq!(entry_size(EntryId::Addons, &roots), 60);
        assert_eq!(entry_size(EntryId::Replays, &roots), 400);
    }

    #[test]
    fn single_and_matching_entries_sum_their_targets() {
        let (_, roots) = fixture("sums");
        assert_eq!(entry_size(EntryId::ShaderCache, &roots), 30);
        assert_eq!(entry_size(EntryId::ConsoleLog, &roots), 5);
        assert_eq!(entry_size(EntryId::HeroPresenceCache, &roots), 3);
        assert_eq!(entry_size(EntryId::AddonsBackups, &roots), 30);
        assert_eq!(entry_size(EntryId::GameinfoBackups, &roots), 15);
        assert_eq!(entry_size(EntryId::ConfigBackups, &roots), 11);
        assert_eq!(entry_size(EntryId::VoiceBanBackups, &roots), 10);
        assert_eq!(entry_size(EntryId::AppData, &roots), 2);
        assert_eq!(entry_size(EntryId::Logs, &roots), 55);
    }

    #[test]
    fn entries_without_their_root_are_empty_not_errors() {
        let roots = Roots::default();
        for id in ALL_ENTRIES {
            assert_eq!(entry_size(id, &roots), 0);
            assert_eq!(location(id, &roots), None);
        }
    }

    #[test]
    fn backup_names_need_a_name_and_numeric_suffixes() {
        for ok in ["voice_ban.dt.backup-1700000000", "voice_ban.dt.backup-42-3", "a.backup-1"] {
            assert!(is_our_backup(ok), "{ok}");
        }
        for bad in [
            "voice_ban.dt",
            "voice_ban.dt.backup-old",
            ".backup-1",
            "x.backup-",
            "x.backup-1-",
            "x.backup-1-a",
            "x.backup-1-2-3",
            "x.backup-1x",
            "x.backup",
        ] {
            assert!(!is_our_backup(bad), "{bad}");
        }
    }

    #[test]
    fn within_requires_a_strict_descendant_without_parent_segments() {
        let root = Path::new("R:/game");
        assert!(within(root, &root.join("a").join("b")));
        assert!(!within(root, root));
        assert!(!within(root, Path::new("R:/other/a")));
        assert!(!within(root, &root.join("..").join("x")));
        assert!(!within(root, Path::new("R:/game-two/a")));
    }

    #[test]
    fn only_regenerable_entries_are_clearable() {
        let clearable: Vec<_> = ALL_ENTRIES.into_iter().filter(|id| is_clearable(*id)).collect();
        assert_eq!(
            clearable,
            vec![EntryId::ShaderCache, EntryId::ConsoleLog, EntryId::VoiceBanBackups, EntryId::Logs]
        );
    }

    #[test]
    fn clearing_a_read_only_entry_is_refused_and_touches_nothing() {
        let (base, roots) = fixture("refuse");
        for id in ALL_ENTRIES.into_iter().filter(|id| !is_clearable(*id)) {
            assert!(clear(id, &roots).is_err(), "{id:?}");
        }
        let citadel = base.join("install").join("game").join("citadel");
        assert!(citadel.join("gameinfo.gi.bak").exists());
        assert!(citadel.join("addons").join("replays").join("1.dem").exists());
        assert!(citadel.join("addons-backups").join("old.vpk").exists());
    }

    #[test]
    fn clearing_the_shader_cache_empties_it_but_keeps_the_folder() {
        let (base, roots) = fixture("shader");
        let report = clear(EntryId::ShaderCache, &roots).unwrap();
        let dir = base.join("install").join("game").join("citadel").join("shadercache");
        assert_eq!((report.freed_bytes, report.removed), (30, 2));
        assert!(dir.is_dir());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
    }

    #[test]
    fn clearing_the_console_log_leaves_neighbours_alone() {
        let (base, roots) = fixture("log");
        let report = clear(EntryId::ConsoleLog, &roots).unwrap();
        let citadel = base.join("install").join("game").join("citadel");
        assert_eq!(report.freed_bytes, 5);
        assert!(!citadel.join("console.log").exists());
        assert!(citadel.join("gameinfo.gi").exists());
        assert!(citadel.join("shadercache").join("a.bin").exists());
    }

    #[test]
    fn clearing_logs_removes_archives_but_leaves_the_active_file_open_for_writing() {
        let (base, roots) = fixture("logs");
        let report = clear(EntryId::Logs, &roots).unwrap();
        let logs = base.join("logs");
        assert_eq!((report.freed_bytes, report.removed), (15, 1));
        assert!(logs.join("debug.log").exists());
        assert!(!logs.join("2026-09-01-1.log.gz").exists());
    }

    #[test]
    fn clearing_mute_backups_removes_only_our_numbered_backups() {
        let (base, roots) = fixture("vb");
        let report = clear(EntryId::VoiceBanBackups, &roots).unwrap();
        let remote = base.join("userdata").join("42").join(DEADLOCK_APP_ID).join("remote");
        assert_eq!((report.freed_bytes, report.removed), (10, 2));
        assert!(remote.join("voice_ban.dt").exists());
        assert!(remote.join("voice_ban.dt.backup-old").exists());
        assert!(!remote.join("voice_ban.dt.backup-100").exists());
        assert!(!remote.join("voice_ban.dt.backup-100-2").exists());
    }

    #[test]
    fn clearing_with_nothing_there_frees_nothing() {
        let report = clear(EntryId::ShaderCache, &Roots::default()).unwrap();
        assert_eq!((report.freed_bytes, report.removed), (0, 0));
    }

    #[test]
    fn a_path_outside_every_root_is_dropped_from_the_clear_list() {
        let (base, mut roots) = fixture("guard");
        roots.install = Some(base.join("install"));
        let outside = base.join("outside");
        write(&outside.join("x"), 1);
        // A userdata root pointing somewhere unrelated still only ever yields backups inside it.
        roots.userdata = Some(outside.clone());
        assert!(clear_paths(EntryId::VoiceBanBackups, &roots).unwrap().is_empty());
        assert!(outside.join("x").exists());
    }

    #[test]
    fn locations_point_at_the_folder_holding_the_files() {
        let (base, roots) = fixture("loc");
        let citadel = base.join("install").join("game").join("citadel");
        assert_eq!(location(EntryId::GameinfoBackups, &roots), Some(citadel.clone()));
        assert_eq!(location(EntryId::ShaderCache, &roots), Some(citadel.join("shadercache")));
        assert_eq!(location(EntryId::Replays, &roots), Some(citadel.join("addons").join("replays")));
    }
}
