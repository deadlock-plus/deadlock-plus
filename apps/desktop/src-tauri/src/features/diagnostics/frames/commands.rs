use tauri::State;

use dp_frames::capture::{CaptureStatus, FrameCapture};
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
