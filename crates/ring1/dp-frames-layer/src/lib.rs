mod clock;
mod layer;
mod recorder;
mod registry;
mod vk;

pub use layer::vkNegotiateLoaderLayerInterfaceVersion;

#[cfg(test)]
mod test_util {
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    use dp_frames_wire::{Reader, Record};

    pub fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dp-frames-layer-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    /// Reads the file until it holds `count` records or five seconds pass.
    pub fn read_until(path: &Path, count: usize) -> Vec<Record> {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let mut reader = Reader::default();
            let mut out = Vec::new();
            if let Ok(bytes) = std::fs::read(path) {
                reader.feed(&bytes, &mut out).unwrap();
            }
            if out.len() >= count || Instant::now() > deadline {
                return out;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
