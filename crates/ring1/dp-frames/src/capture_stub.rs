pub use crate::status::{CaptureState, CaptureStatus};
use crate::FrameStats;

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
