use serde::Serialize;
use ts_rs::TS;

use super::FrameStats;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum CaptureState {
    #[default]
    Idle,
    WaitingForGame,
    Capturing,
    Failed,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct CaptureStatus {
    pub state: CaptureState,
    pub error: Option<String>,
    pub frames: u32,
    pub elapsed_ms: u32,
    pub other_process_events: u32,
    pub truncated: bool,
    pub game_focused: bool,
    pub recent_frametimes_ms: Vec<f32>,
}

#[derive(Default)]
pub struct FrameCapture;

impl FrameCapture {
    pub fn start(&self, _is_game: fn(&std::ffi::OsStr) -> bool) {}
    pub fn status(&self) -> CaptureStatus {
        CaptureStatus::default()
    }
    pub fn stop(&self) -> FrameStats {
        FrameStats::default()
    }
}
