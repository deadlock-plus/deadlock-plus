pub mod cleanup;
pub mod metadata;
pub mod pin;

use std::path::PathBuf;

pub fn replays_dir() -> Option<PathBuf> {
    let dir = dp_steam::replays_dir(&dp_steam::game_install_dir()?);
    dir.is_dir().then_some(dir)
}

pub mod commands {
    use serde::{Deserialize, Serialize};

    use super::replays_dir;
    use dp_demos::delete::{
        availability_for, bin_info, delete_permanently, delete_to_bin, resolve_targets, DeleteReport,
        RecycleAvailability,
    };
    use dp_demos::pin::{split_pinned, PinStore};
    use dp_demos::{list_demos_in, DemoListing};
    use tauri::{Manager, State};
    use ts_rs::TS;

    fn pins_for(app: &tauri::AppHandle, store: &PinStore) -> Result<dp_demos::pin::Pins, String> {
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
