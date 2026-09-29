pub mod cleanup;
pub mod delete;
pub mod metadata;
pub mod pin;

use std::io::Read;
use std::path::{Path, PathBuf};
use ts_rs::TS;

use serde::Serialize;

const DEADLOCK_APP_ID: u32 = 1422450;
const DEMO_MAGIC: &[u8; 8] = b"PBDEMS2\0";
const HEADER_READ_BYTES: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoKind {
    Complete,
    Partial,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum DemoStatus {
    Complete,
    Partial,
    Outdated,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DemoHeader {
    pub network_protocol: u32,
    pub build_num: u32,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, rename = "Demo")]
#[serde(rename_all = "camelCase")]
pub struct DemoEntry {
    pub match_id: u64,
    pub file_name: String,
    pub size: u64,
    pub modified_ms: u64,
    pub status: DemoStatus,
    pub build_num: Option<u32>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DemoListing {
    pub dir: Option<String>,
    pub reference_build: Option<u32>,
    pub demos: Vec<DemoEntry>,
}

pub fn parse_demo_filename(name: &str) -> Option<(u64, DemoKind)> {
    let (stem, kind) = match name.strip_suffix(".dem.partial") {
        Some(stem) => (stem, DemoKind::Partial),
        None => (name.strip_suffix(".dem")?, DemoKind::Complete),
    };
    if stem.is_empty() || !stem.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((stem.parse().ok()?, kind))
}

fn read_varint(bytes: &[u8], pos: &mut usize) -> Option<u64> {
    let mut value = 0u64;
    for shift in (0..64).step_by(7) {
        let byte = *bytes.get(*pos)?;
        *pos += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte < 0x80 {
            return Some(value);
        }
    }
    None
}

/// Layout: magic, two int32 offsets, then the first command (`DEM_FileHeader`, cmd 1) as
/// varint cmd, varint tick, varint size, protobuf body. Only fields 2 (`network_protocol`)
/// and 13 (`build_num`) are read.
pub fn parse_header(bytes: &[u8]) -> Option<DemoHeader> {
    if bytes.get(..DEMO_MAGIC.len())? != DEMO_MAGIC {
        return None;
    }
    let mut pos = DEMO_MAGIC.len() + 8;
    if read_varint(bytes, &mut pos)? != 1 {
        return None;
    }
    read_varint(bytes, &mut pos)?;
    let size = usize::try_from(read_varint(bytes, &mut pos)?).ok()?;
    let body = bytes.get(pos..pos.checked_add(size)?)?;

    let (mut network_protocol, mut build_num) = (None, None);
    let mut i = 0;
    while i < body.len() {
        let key = read_varint(body, &mut i)?;
        match key & 7 {
            0 => {
                let value = u32::try_from(read_varint(body, &mut i)?).ok();
                match key >> 3 {
                    2 => network_protocol = value,
                    13 => build_num = value,
                    _ => {}
                }
            }
            1 => i += 8,
            2 => i += usize::try_from(read_varint(body, &mut i)?).ok()?,
            5 => i += 4,
            _ => return None,
        }
    }
    Some(DemoHeader { network_protocol: network_protocol?, build_num: build_num? })
}

/// The game's own build number is not stored in a form comparable with `build_num`, so the
/// newest build found among the local replays stands in as the reference.
pub fn classify(kind: DemoKind, header: Option<DemoHeader>, reference_build: Option<u32>) -> DemoStatus {
    if kind == DemoKind::Partial {
        return DemoStatus::Partial;
    }
    match (header, reference_build) {
        (Some(h), Some(reference)) if h.build_num < reference => DemoStatus::Outdated,
        (Some(_), Some(_)) => DemoStatus::Complete,
        _ => DemoStatus::Unknown,
    }
}

pub fn is_deletable(replays_dir: &Path, path: &Path) -> bool {
    use std::path::Component;
    let name_ok = path.file_name().and_then(|n| n.to_str()).is_some_and(|n| parse_demo_filename(n).is_some());
    let clean = !path.components().any(|c| matches!(c, Component::ParentDir | Component::CurDir));
    name_ok && clean && path.parent() == Some(replays_dir)
}

pub fn replays_dir() -> Option<PathBuf> {
    let steam = steamlocate::SteamDir::locate().ok()?;
    let (app, library) = steam.find_app(DEADLOCK_APP_ID).ok()??;
    let dir = library.resolve_app_dir(&app).join("game").join("citadel").join("addons").join("replays");
    dir.is_dir().then_some(dir)
}

fn read_header(path: &Path) -> Option<DemoHeader> {
    let mut buf = Vec::with_capacity(HEADER_READ_BYTES);
    std::fs::File::open(path).ok()?.take(HEADER_READ_BYTES as u64).read_to_end(&mut buf).ok()?;
    parse_header(&buf)
}

pub fn list_demos_in(dir: &Path) -> Vec<DemoEntry> {
    struct Raw {
        entry: DemoEntry,
        kind: DemoKind,
        header: Option<DemoHeader>,
    }
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut raws: Vec<Raw> = read
        .filter_map(Result::ok)
        .filter_map(|e| {
            let file_name = e.file_name().to_string_lossy().into_owned();
            let (match_id, kind) = parse_demo_filename(&file_name)?;
            let meta = e.metadata().ok().filter(|m| m.is_file())?;
            let modified_ms = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_millis() as u64);
            let header = (kind == DemoKind::Complete).then(|| read_header(&e.path())).flatten();
            let entry = DemoEntry {
                match_id,
                file_name,
                size: meta.len(),
                modified_ms,
                status: DemoStatus::Unknown,
                build_num: header.map(|h| h.build_num),
            };
            Some(Raw { entry, kind, header })
        })
        .collect();
    let reference = raws.iter().filter_map(|r| r.header).map(|h| h.build_num).max();
    for raw in &mut raws {
        raw.entry.status = classify(raw.kind, raw.header, reference);
    }
    let mut demos: Vec<DemoEntry> = raws.into_iter().map(|r| r.entry).collect();
    demos.sort_by_key(|d| std::cmp::Reverse(d.modified_ms));
    demos
}

pub mod commands {
    use serde::{Deserialize, Serialize};

    use super::delete::{
        availability_for, bin_info, delete_permanently, delete_to_bin, resolve_targets, DeleteReport,
        RecycleAvailability,
    };
    use super::pin::{split_pinned, PinStore};
    use super::{list_demos_in, replays_dir, DemoListing};
    use tauri::{Manager, State};
    use ts_rs::TS;

    fn pins_for(app: &tauri::AppHandle, store: &PinStore) -> Result<super::pin::Pins, String> {
        let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        Ok(store.snapshot(&dir))
    }

    #[tauri::command]
    pub async fn list_demos() -> Result<DemoListing, String> {
        tauri::async_runtime::spawn_blocking(|| {
            let Some(dir) = replays_dir() else {
                return DemoListing { dir: None, reference_build: None, demos: Vec::new() };
            };
            let demos = list_demos_in(&dir);
            log::debug!("listed {} replays", demos.len());
            let reference_build = demos.iter().filter_map(|d| d.build_num).max();
            DemoListing { dir: Some(dir.to_string_lossy().into_owned()), reference_build, demos }
        })
        .await
        .map_err(|e| e.to_string())
    }

    #[derive(Serialize, TS)]
    #[ts(export)]
    #[serde(rename_all = "camelCase")]
    pub struct DeletePreview {
        pub count: usize,
        pub total_bytes: u64,
        pub recycle: RecycleAvailability,
        pub bin_free_bytes: Option<u64>,
    }

    #[derive(Deserialize, TS)]
    #[ts(export)]
    #[serde(rename_all = "camelCase")]
    pub enum DeleteMode {
        Recycle,
        Permanent,
    }

    #[tauri::command]
    pub async fn delete_preview(
        app: tauri::AppHandle,
        store: State<'_, PinStore>,
        file_names: Vec<String>,
    ) -> Result<DeletePreview, String> {
        let pins = pins_for(&app, &store)?;
        tauri::async_runtime::spawn_blocking(move || {
            let dir = replays_dir().ok_or("Replays folder not found.")?;
            let (file_names, _) = split_pinned(&file_names, &pins);
            let (targets, _) = resolve_targets(&dir, &file_names);
            let total_bytes = targets.iter().map(|t| t.size).sum();
            Ok(DeletePreview {
                count: targets.len(),
                total_bytes,
                recycle: availability_for(&dir, total_bytes),
                bin_free_bytes: bin_info(&dir).map(|b| b.limit.saturating_sub(b.used)),
            })
        })
        .await
        .map_err(|e| e.to_string())?
    }

    /// Recycling is re-checked here; a request the bin cannot hold is refused, never turned into
    /// a permanent delete.
    #[tauri::command]
    pub async fn delete_demos(
        app: tauri::AppHandle,
        store: State<'_, PinStore>,
        file_names: Vec<String>,
        mode: DeleteMode,
    ) -> Result<DeleteReport, String> {
        let pins = pins_for(&app, &store)?;
        tauri::async_runtime::spawn_blocking(move || {
            let dir = replays_dir().ok_or("Replays folder not found.")?;
            let (file_names, mut pinned_refusals) = split_pinned(&file_names, &pins);
            let (targets, mut failed) = resolve_targets(&dir, &file_names);
            failed.append(&mut pinned_refusals);
            let mut report = match mode {
                DeleteMode::Permanent => delete_permanently(targets),
                DeleteMode::Recycle => {
                    let total = targets.iter().map(|t| t.size).sum();
                    if availability_for(&dir, total) != RecycleAvailability::Available {
                        log::error!("replay delete refused: the Recycle Bin cannot hold {total} bytes");
                        return Err("The Recycle Bin cannot hold these replays.".to_string());
                    }
                    delete_to_bin(targets)
                }
            };
            report.failed.append(&mut failed);
            log::info!(
                "replays deleted: {} {}, {} failed or skipped",
                report.deleted.len(),
                match mode {
                    DeleteMode::Permanent => "permanently",
                    DeleteMode::Recycle => "to the Recycle Bin",
                },
                report.failed.len()
            );
            Ok(report)
        })
        .await
        .map_err(|e| e.to_string())?
    }

    #[tauri::command]
    pub fn open_replays_dir() -> Result<(), String> {
        let dir = replays_dir().ok_or("Replays folder not found.")?;
        crate::features::reveal::show(&dir).map_err(|e| {
            log::error!("could not open the replays folder: {e}");
            e
        })
    }

    /// The frontend passes a match id, never a path; the file is resolved inside the replays folder.
    #[tauri::command]
    pub fn reveal_demo(match_id: String, partial: bool) -> Result<(), String> {
        let id: u64 = match_id.parse().map_err(|_| "Invalid match id.".to_string())?;
        let dir = replays_dir().ok_or("Replays folder not found.")?;
        let name = if partial { format!("{id}.dem.partial") } else { format!("{id}.dem") };
        let path = dir.join(name);
        if !path.is_file() {
            log::warn!("reveal requested for a missing replay ({id})");
            return Err("That replay no longer exists.".into());
        }
        crate::features::reveal::show(&path).map_err(|e| {
            log::error!("could not reveal replay {id}: {e}");
            e
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn varint(mut v: u64) -> Vec<u8> {
        let mut out = Vec::new();
        loop {
            let byte = (v & 0x7f) as u8;
            v >>= 7;
            if v == 0 {
                out.push(byte);
                return out;
            }
            out.push(byte | 0x80);
        }
    }

    fn header_bytes(protocol: u32, build: u32) -> Vec<u8> {
        let mut msg = Vec::new();
        msg.extend([0x0a, 0x08]);
        msg.extend(DEMO_MAGIC.iter().take(8));
        msg.push(0x10);
        msg.extend(varint(protocol as u64));
        msg.extend([0x1a, 0x03]);
        msg.extend(b"abc");
        msg.push(13 << 3);
        msg.extend(varint(build as u64));

        let mut out = Vec::new();
        out.extend(DEMO_MAGIC);
        out.extend([0xa8, 0xdb, 0x64, 0x15, 0x44, 0xb4, 0x64, 0x15]);
        out.push(0x01);
        out.extend([0xff, 0xff, 0xff, 0xff, 0x0f]);
        out.extend(varint(msg.len() as u64));
        out.extend(msg);
        out
    }

    #[test]
    fn filenames_give_match_id_and_kind() {
        assert_eq!(parse_demo_filename("32433914.dem"), Some((32433914, DemoKind::Complete)));
        assert_eq!(parse_demo_filename("30709833.dem.partial"), Some((30709833, DemoKind::Partial)));
    }

    #[test]
    fn junk_filenames_are_rejected() {
        for name in
            ["", "notes.txt", ".dem", "abc.dem", "12.dem.bak", "12.DEM", "12.dem.partial.x", "-5.dem", "1 2.dem"]
        {
            assert_eq!(parse_demo_filename(name), None, "{name}");
        }
    }

    #[test]
    fn header_fields_are_read_from_the_file_header_message() {
        assert_eq!(parse_header(&header_bytes(48, 10854)), Some(DemoHeader { network_protocol: 48, build_num: 10854 }));
    }

    #[test]
    fn header_survives_a_multi_byte_size_prefix() {
        let mut bytes = header_bytes(48, 10854);
        // Pad the message past 127 bytes so its size prefix takes two bytes.
        let mut msg = Vec::new();
        msg.extend([0x22, 0x90, 0x01]);
        msg.extend(std::iter::repeat_n(b'x', 144));
        msg.push(0x10);
        msg.extend(varint(48));
        msg.push(13 << 3);
        msg.extend(varint(10725));
        bytes.truncate(8 + 8 + 1 + 5);
        bytes.extend(varint(msg.len() as u64));
        bytes.extend(msg);
        assert_eq!(parse_header(&bytes), Some(DemoHeader { network_protocol: 48, build_num: 10725 }));
    }

    #[test]
    fn header_rejects_zeroed_bad_magic_and_truncated_input() {
        assert_eq!(parse_header(&[0u8; 64]), None);
        assert_eq!(parse_header(b"PBDEMS2"), None);
        assert_eq!(parse_header(b""), None);
        let full = header_bytes(48, 10854);
        assert_eq!(parse_header(&full[..full.len() - 3]), None);
        let mut wrong = full.clone();
        wrong[0] = b'X';
        assert_eq!(parse_header(&wrong), None);
    }

    #[test]
    fn header_without_a_build_number_is_rejected() {
        let mut bytes = Vec::new();
        bytes.extend(DEMO_MAGIC);
        bytes.extend([0; 8]);
        bytes.extend([0x01, 0xff, 0xff, 0xff, 0xff, 0x0f, 0x02, 0x10, 0x30]);
        assert_eq!(parse_header(&bytes), None);
    }

    #[test]
    fn partial_files_are_partial_whatever_the_header_says() {
        let h = Some(DemoHeader { network_protocol: 48, build_num: 1 });
        assert_eq!(classify(DemoKind::Partial, h, Some(9)), DemoStatus::Partial);
        assert_eq!(classify(DemoKind::Partial, None, None), DemoStatus::Partial);
    }

    #[test]
    fn complete_files_are_compared_with_the_reference_build() {
        let h = |b| Some(DemoHeader { network_protocol: 48, build_num: b });
        assert_eq!(classify(DemoKind::Complete, h(10725), Some(10854)), DemoStatus::Outdated);
        assert_eq!(classify(DemoKind::Complete, h(10854), Some(10854)), DemoStatus::Complete);
        assert_eq!(classify(DemoKind::Complete, h(10900), Some(10854)), DemoStatus::Complete);
    }

    #[test]
    fn complete_files_with_no_readable_header_or_reference_are_unknown() {
        let h = Some(DemoHeader { network_protocol: 48, build_num: 5 });
        assert_eq!(classify(DemoKind::Complete, None, Some(10)), DemoStatus::Unknown);
        assert_eq!(classify(DemoKind::Complete, h, None), DemoStatus::Unknown);
    }

    #[test]
    fn only_demo_files_directly_inside_the_replays_dir_are_deletable() {
        let dir = Path::new("R:/replays");
        assert!(is_deletable(dir, &dir.join("1.dem")));
        assert!(is_deletable(dir, &dir.join("1.dem.partial")));
        assert!(!is_deletable(dir, &dir.join("notes.txt")));
        assert!(!is_deletable(dir, &dir.join("sub").join("1.dem")));
        assert!(!is_deletable(dir, Path::new("R:/other/1.dem")));
        assert!(!is_deletable(dir, &dir.join("..").join("1.dem")));
        assert!(!is_deletable(dir, dir));
        assert!(!is_deletable(dir, Path::new("1.dem")));
    }
}
