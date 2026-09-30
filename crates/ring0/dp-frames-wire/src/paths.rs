use std::path::{Path, PathBuf};

pub const FRAME_FILE_EXTENSION: &str = "dpf";

/// `$XDG_DATA_HOME` when it is an absolute path, else `$HOME/.local/share`, as the XDG spec says.
pub fn data_home(xdg_data_home: Option<&Path>, home: Option<&Path>) -> Option<PathBuf> {
    match xdg_data_home {
        Some(p) if p.is_absolute() => Some(p.to_path_buf()),
        _ => home.filter(|h| h.is_absolute()).map(|h| h.join(".local").join("share")),
    }
}

/// The folder the layer writes one file per process into and the app reads from.
pub fn frames_dir(data_home: &Path) -> PathBuf {
    data_home.join("deadlock-plus").join("frames")
}

/// One of the folders the Vulkan loader scans for implicit layer manifests.
pub fn implicit_layer_dir(data_home: &Path) -> PathBuf {
    data_home.join("vulkan").join("implicit_layer.d")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    const ABS: &str = "/data";
    #[cfg(windows)]
    const ABS: &str = "C:\\data";

    #[test]
    fn xdg_data_home_wins_when_absolute() {
        let got = data_home(Some(Path::new(ABS)), Some(Path::new(ABS)));
        assert_eq!(got, Some(PathBuf::from(ABS)));
    }

    #[test]
    fn relative_or_missing_xdg_falls_back_to_home() {
        let expected = Some(Path::new(ABS).join(".local").join("share"));
        assert_eq!(data_home(Some(Path::new("relative")), Some(Path::new(ABS))), expected);
        assert_eq!(data_home(None, Some(Path::new(ABS))), expected);
    }

    #[test]
    fn nothing_usable_gives_none() {
        assert_eq!(data_home(None, None), None);
        assert_eq!(data_home(Some(Path::new("x")), Some(Path::new("y"))), None);
    }

    #[test]
    fn folders_sit_under_the_data_home() {
        let base = Path::new(ABS);
        assert_eq!(frames_dir(base), base.join("deadlock-plus").join("frames"));
        assert_eq!(implicit_layer_dir(base), base.join("vulkan").join("implicit_layer.d"));
    }
}
