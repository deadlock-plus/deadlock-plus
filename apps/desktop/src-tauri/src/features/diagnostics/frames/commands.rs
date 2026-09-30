use tauri::State;

use dp_frames::capture::{CaptureStatus, FrameCapture};
use dp_frames::layer_install::{self, LayerStatus};
use dp_frames::FrameStats;

#[tauri::command]
pub async fn start_frame_capture(capture: State<'_, FrameCapture>) -> Result<(), String> {
    capture.start(dp_game::is_process);
    Ok(())
}

#[tauri::command]
pub async fn frame_capture_status(capture: State<'_, FrameCapture>) -> Result<CaptureStatus, String> {
    Ok(capture.status())
}

#[tauri::command]
pub async fn stop_frame_capture(capture: State<'_, FrameCapture>) -> Result<FrameStats, String> {
    Ok(capture.stop())
}

fn layer_status() -> LayerStatus {
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
    layer_install::status(cfg!(target_os = "linux"), layer_install::system_data_home().as_deref(), exe_dir.as_deref())
}

#[tauri::command]
pub async fn frame_layer_status() -> Result<LayerStatus, String> {
    Ok(layer_status())
}

#[tauri::command]
pub async fn install_frame_layer() -> Result<LayerStatus, String> {
    if !cfg!(target_os = "linux") {
        return Err("The frame layer is only available on Linux.".into());
    }
    let data_home = layer_install::system_data_home().ok_or("Could not find your home folder.")?;
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
    let source = exe_dir
        .as_deref()
        .and_then(layer_install::find_library)
        .ok_or("This build of Deadlock+ does not include the frame layer.")?;
    layer_install::install(&data_home, &source).map_err(|e| format!("Could not install the frame layer: {e}"))?;
    Ok(layer_status())
}

#[tauri::command]
pub async fn uninstall_frame_layer() -> Result<LayerStatus, String> {
    if let Some(data_home) = layer_install::system_data_home() {
        layer_install::uninstall(&data_home).map_err(|e| format!("Could not remove the frame layer: {e}"))?;
    }
    Ok(layer_status())
}
