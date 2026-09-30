use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CaptureState {
    #[default]
    Idle,
    WaitingForGame,
    Capturing,
    Failed,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptureStatus {
    pub state: CaptureState,
    pub error: Option<String>,
    pub frames: u32,
    pub elapsed_ms: u32,
    /// Present events that are not counted as game frames: other processes on Windows, secondary
    /// swapchains on Linux. Nonzero with zero `frames` on Windows means the game is not presenting
    /// through DXGI.
    pub other_process_events: u32,
    pub truncated: bool,
    pub game_focused: bool,
    pub recent_frametimes_ms: Vec<f32>,
}
