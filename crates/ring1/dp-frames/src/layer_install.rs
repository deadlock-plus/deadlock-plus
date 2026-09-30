use std::io;
use std::path::{Path, PathBuf};

use dp_frames_wire::{data_home, implicit_layer_dir};
use serde::Serialize;
use ts_rs::TS;

pub const LAYER_NAME: &str = "VK_LAYER_DEADLOCKPLUS_frames";
pub const ENABLE_VARIABLE: &str = "DEADLOCK_PLUS_FRAMES";
/// The layer stays off unless this variable is set, so a stray copy can never affect another game.
pub const DISABLE_VARIABLE: &str = "DEADLOCK_PLUS_FRAMES_DISABLE";
pub const LIBRARY_FILE: &str = "libdp_frames_layer.so";
pub const MANIFEST_FILE: &str = "deadlock_plus_frames.json";
/// Pasted into the game's Steam launch options.
pub const LAUNCH_OPTION: &str = "DEADLOCK_PLUS_FRAMES=1 %command%";

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LayerStatus {
    /// False on platforms with no way to capture frames this way.
    pub supported: bool,
    pub installed: bool,
    /// Whether the app found the layer library it ships, which install copies into place.
    pub library_available: bool,
    pub launch_option: String,
    pub manifest_path: Option<String>,
}

pub struct Layout {
    pub manifest: PathBuf,
    pub library: PathBuf,
}

/// The manifest goes where the Vulkan loader looks; the library sits under the app's own folder so the
/// manifest can point at a stable absolute path.
pub fn layout(data_home: &Path) -> Layout {
    Layout {
        manifest: implicit_layer_dir(data_home).join(MANIFEST_FILE),
        library: data_home.join("deadlock-plus").join("layer").join(LIBRARY_FILE),
    }
}

pub fn system_data_home() -> Option<PathBuf> {
    let xdg = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from);
    let home = std::env::var_os("HOME").map(PathBuf::from);
    data_home(xdg.as_deref(), home.as_deref())
}

/// `None` if `library` is not valid UTF-8, which a JSON manifest cannot hold.
pub fn manifest_json(library: &Path) -> Option<String> {
    let manifest = serde_json::json!({
        "file_format_version": "1.0.0",
        "layer": {
            "name": LAYER_NAME,
            "type": "GLOBAL",
            "library_path": library.to_str()?,
            "api_version": "1.3.0",
            "implementation_version": "1",
            "description": "Deadlock+ frame timing",
            "functions": {
                "vkNegotiateLoaderLayerInterfaceVersion": "vkNegotiateLoaderLayerInterfaceVersion"
            },
            "enable_environment": { ENABLE_VARIABLE: "1" },
            "disable_environment": { DISABLE_VARIABLE: "1" }
        }
    });
    serde_json::to_string_pretty(&manifest).ok()
}

/// Where the app looks for the layer library it ships: beside the executable, in a `lib` folder next to
/// its `bin` folder, and in the package's own `lib/deadlock-plus` folder.
pub fn find_library(exe_dir: &Path) -> Option<PathBuf> {
    let mut candidates = vec![exe_dir.join(LIBRARY_FILE)];
    if let Some(prefix) = exe_dir.parent() {
        candidates.push(prefix.join("lib").join("deadlock-plus").join(LIBRARY_FILE));
        candidates.push(prefix.join("lib").join(LIBRARY_FILE));
    }
    candidates.into_iter().find(|c| c.is_file())
}

pub fn is_installed(data_home: &Path) -> bool {
    let layout = layout(data_home);
    let Ok(text) = std::fs::read_to_string(&layout.manifest) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    json["layer"]["library_path"].as_str().is_some_and(|p| Path::new(p) == layout.library) && layout.library.is_file()
}

pub fn status(supported: bool, data_home: Option<&Path>, exe_dir: Option<&Path>) -> LayerStatus {
    LayerStatus {
        supported,
        installed: supported && data_home.is_some_and(is_installed),
        library_available: exe_dir.and_then(find_library).is_some(),
        launch_option: LAUNCH_OPTION.to_string(),
        manifest_path: data_home.map(|d| layout(d).manifest.to_string_lossy().into_owned()),
    }
}

/// Copies the library into place and then writes the manifest, so the loader never sees a manifest
/// that points at a missing file. The library is replaced by rename: a running game keeps its old
/// mapping instead of having the file overwritten under it.
pub fn install(data_home: &Path, source_library: &Path) -> io::Result<()> {
    let layout = layout(data_home);
    let manifest = manifest_json(&layout.library)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "the data folder path is not valid UTF-8"))?;
    let library_dir = layout.library.parent().expect("library path has a parent");
    let manifest_dir = layout.manifest.parent().expect("manifest path has a parent");
    std::fs::create_dir_all(library_dir)?;
    std::fs::create_dir_all(manifest_dir)?;

    let staged = layout.library.with_extension("so.tmp");
    let copied = std::fs::copy(source_library, &staged).and_then(|_| std::fs::rename(&staged, &layout.library));
    if let Err(e) = copied {
        let _ = std::fs::remove_file(&staged);
        return Err(e);
    }
    dp_atomic::write_atomic(&layout.manifest, manifest.as_bytes())
}

pub fn uninstall(data_home: &Path) -> io::Result<()> {
    let layout = layout(data_home);
    for path in [&layout.manifest, &layout.library] {
        match std::fs::remove_file(path) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dp-frames-install-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn fake_library(dir: &Path) -> PathBuf {
        let path = dir.join("source.so");
        std::fs::write(&path, b"library bytes").unwrap();
        path
    }

    #[test]
    fn manifest_is_a_gated_implicit_layer() {
        let text = manifest_json(Path::new("/home/u/.local/share/deadlock-plus/layer/libdp_frames_layer.so")).unwrap();
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        let layer = &json["layer"];
        assert_eq!(json["file_format_version"], "1.0.0");
        assert_eq!(layer["name"], LAYER_NAME);
        assert_eq!(layer["type"], "GLOBAL");
        assert_eq!(layer["library_path"], "/home/u/.local/share/deadlock-plus/layer/libdp_frames_layer.so");
        assert_eq!(layer["enable_environment"][ENABLE_VARIABLE], "1");
        assert!(layer["disable_environment"][DISABLE_VARIABLE].is_string());
        assert_eq!(
            layer["functions"]["vkNegotiateLoaderLayerInterfaceVersion"],
            "vkNegotiateLoaderLayerInterfaceVersion"
        );
    }

    #[test]
    fn launch_option_sets_the_variable_the_manifest_gates_on() {
        assert!(LAUNCH_OPTION.starts_with(&format!("{ENABLE_VARIABLE}=1 ")));
        assert!(LAUNCH_OPTION.ends_with("%command%"));
    }

    #[test]
    fn install_copies_the_library_and_writes_a_manifest_that_points_at_it() {
        let data = temp_dir("install");
        let source = fake_library(&data);
        assert!(!is_installed(&data));
        install(&data, &source).unwrap();
        let l = layout(&data);
        assert_eq!(std::fs::read(&l.library).unwrap(), b"library bytes");
        assert!(l.manifest.starts_with(data.join("vulkan").join("implicit_layer.d")));
        assert!(is_installed(&data));
        let _ = std::fs::remove_dir_all(&data);
    }

    #[test]
    fn install_twice_replaces_the_library_and_leaves_no_temp_file() {
        let data = temp_dir("reinstall");
        let source = fake_library(&data);
        install(&data, &source).unwrap();
        std::fs::write(&source, b"newer").unwrap();
        install(&data, &source).unwrap();
        let l = layout(&data);
        assert_eq!(std::fs::read(&l.library).unwrap(), b"newer");
        assert!(!l.library.with_extension("so.tmp").exists());
        let _ = std::fs::remove_dir_all(&data);
    }

    #[test]
    fn a_missing_source_fails_without_writing_a_manifest() {
        let data = temp_dir("missing");
        assert!(install(&data, &data.join("nope.so")).is_err());
        assert!(!layout(&data).manifest.exists());
        let _ = std::fs::remove_dir_all(&data);
    }

    #[test]
    fn a_manifest_pointing_elsewhere_or_at_a_missing_library_is_not_installed() {
        let data = temp_dir("stale");
        let source = fake_library(&data);
        install(&data, &source).unwrap();
        std::fs::remove_file(layout(&data).library).unwrap();
        assert!(!is_installed(&data));
        install(&data, &source).unwrap();
        std::fs::write(layout(&data).manifest, manifest_json(Path::new("/elsewhere/lib.so")).unwrap()).unwrap();
        assert!(!is_installed(&data));
        std::fs::write(layout(&data).manifest, "not json").unwrap();
        assert!(!is_installed(&data));
        let _ = std::fs::remove_dir_all(&data);
    }

    #[test]
    fn uninstall_removes_both_files_and_tolerates_missing_ones() {
        let data = temp_dir("uninstall");
        let source = fake_library(&data);
        install(&data, &source).unwrap();
        uninstall(&data).unwrap();
        assert!(!layout(&data).manifest.exists() && !layout(&data).library.exists());
        uninstall(&data).unwrap();
        let _ = std::fs::remove_dir_all(&data);
    }

    #[test]
    fn find_library_checks_the_known_places_in_order() {
        let root = temp_dir("find");
        let bin = root.join("usr").join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        assert_eq!(find_library(&bin), None);
        let packaged = root.join("usr").join("lib").join("deadlock-plus");
        std::fs::create_dir_all(&packaged).unwrap();
        std::fs::write(packaged.join(LIBRARY_FILE), b"x").unwrap();
        assert_eq!(find_library(&bin), Some(packaged.join(LIBRARY_FILE)));
        std::fs::write(bin.join(LIBRARY_FILE), b"x").unwrap();
        assert_eq!(find_library(&bin), Some(bin.join(LIBRARY_FILE)));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn status_on_an_unsupported_platform_never_reports_installed() {
        let data = temp_dir("unsupported");
        let source = fake_library(&data);
        install(&data, &source).unwrap();
        assert!(!status(false, Some(&data), None).installed);
        let s = status(true, Some(&data), Some(&data));
        assert!(s.installed && !s.library_available);
        assert_eq!(s.launch_option, LAUNCH_OPTION);
        let _ = std::fs::remove_dir_all(&data);
    }
}
