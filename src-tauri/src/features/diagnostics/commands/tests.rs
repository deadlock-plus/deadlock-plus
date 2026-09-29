use super::*;
use crate::features::diagnostics::test_util::{block, vjs_c};
use crate::features::mini_source2::vpk;

const LEAKY_JS: &str = "var h = null;\nfunction f() { h = null; h = $.Schedule(1, f); }\nf();\n";

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("dlp-cmd-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn listing_reports_addons_and_dir() {
    let dir = scratch("list");
    std::fs::write(dir.join("pak01_dir.vpk"), b"x").unwrap();
    let listing = list_addons_from(Some(&dir));
    assert_eq!(listing.dir.as_deref(), Some(dir.to_string_lossy().as_ref()));
    assert_eq!(listing.addons.len(), 1);
    assert_eq!(listing.addons[0].file_name, "pak01_dir.vpk");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn listing_without_a_game_dir_is_empty() {
    let listing = list_addons_from(None);
    assert!(listing.dir.is_none() && listing.addons.is_empty());
}

#[test]
fn scan_names_the_addon_and_reports_findings() {
    let dir = scratch("scan");
    let script = vjs_c(&[
        block(b"RED2", b"SearchPath\0citadel_addons/build_thing\0"),
        block(b"DATA", LEAKY_JS.as_bytes()),
    ]);
    let bytes = vpk::write(&[("panorama/scripts/a.vjs_c", &script)]);
    std::fs::write(dir.join("pak01_dir.vpk"), bytes).unwrap();
    let scan = scan_addon_in(&dir, "pak01_dir.vpk").unwrap();
    assert_eq!(scan.file_name, "pak01_dir.vpk");
    assert_eq!(scan.label, "build_thing");
    assert_eq!(scan.scripts_scanned, 1);
    assert_eq!(scan.scripts.len(), 1);
    assert_eq!(scan.scripts[0].path, "panorama/scripts/a.vjs_c");
    assert_eq!(scan.scripts[0].findings[0].function, "f");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn scan_rejects_names_outside_the_addons_dir() {
    let dir = scratch("reject");
    assert!(scan_addon_in(&dir, "../pak01_dir.vpk").is_err());
    assert!(scan_addon_in(&dir, "missing_dir.vpk").is_err());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn scan_reports_a_corrupt_vpk_as_an_error() {
    let dir = scratch("corrupt");
    std::fs::write(dir.join("pak01_dir.vpk"), b"not a vpk at all").unwrap();
    assert!(scan_addon_in(&dir, "pak01_dir.vpk").is_err());
    std::fs::remove_dir_all(dir).unwrap();
}
