use crate::layer_capture::LayerCapture;
use crate::layer_install::system_data_home;
pub use crate::status::{CaptureState, CaptureStatus};
use crate::FrameStats;

/// Reads present timestamps from the Vulkan layer's files. The layer is loaded into the game only, so
/// no process filter is needed and `start` ignores `is_game`.
pub struct FrameCapture {
    inner: LayerCapture,
}

impl Default for FrameCapture {
    fn default() -> Self {
        Self { inner: LayerCapture::new(system_data_home().map(|d| dp_frames_wire::frames_dir(&d))) }
    }
}

impl FrameCapture {
    pub fn start(&self, _is_game: fn(&std::ffi::OsStr) -> bool) {
        self.inner.start();
    }

    pub fn status(&self) -> CaptureStatus {
        self.inner.status()
    }

    pub fn stop(&self) -> FrameStats {
        self.inner.stop()
    }
}
