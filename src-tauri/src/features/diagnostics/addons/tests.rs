use super::*;

const DMM: &str = r#"{
  "version": 1,
  "mods": {
    "637627": { "enabled": true, "order": 5, "currentVpks": ["pak01_dir.vpk"], "disabledVpks": [], "originalVpkNames": [] },
    "111": { "enabled": false, "order": null, "currentVpks": [], "disabledVpks": ["pak02_dir.vpk"], "originalVpkNames": [] }
  }
}"#;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dlp-addons-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn parses_mod_ids_and_enabled_state_by_vpk() {
    let map = parse_dmm(DMM);
    assert_eq!(map["pak01_dir.vpk"], DmmMod { id: "637627".into(), enabled: true });
    assert_eq!(map["pak02_dir.vpk"], DmmMod { id: "111".into(), enabled: false });
}

#[test]
fn bad_or_empty_dmm_json_is_an_empty_map() {
    assert!(parse_dmm("").is_empty());
    assert!(parse_dmm("{ nope").is_empty());
    assert!(parse_dmm("{}").is_empty());
}

#[test]
fn lists_dir_vpks_in_load_order_with_dmm_info() {
    let dir = scratch("list");
    for name in ["pak02_dir.vpk", "pak01_dir.vpk", "652420_pak17_dir.vpk", "pak01_000.vpk", "notes.txt"] {
        std::fs::write(dir.join(name), b"x").unwrap();
    }
    std::fs::write(dir.join(".dmm.json"), DMM).unwrap();
    let addons = list_addons_in(&dir);
    let names: Vec<_> = addons.iter().map(|a| a.file_name.as_str()).collect();
    assert_eq!(names, ["652420_pak17_dir.vpk", "pak01_dir.vpk", "pak02_dir.vpk"]);
    assert_eq!(addons[0].mod_id, None);
    assert_eq!(addons[1].mod_id.as_deref(), Some("637627"));
    assert_eq!(addons[1].enabled, Some(true));
    assert_eq!(addons[2].enabled, Some(false));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_dir_lists_nothing() {
    assert!(list_addons_in(Path::new("Z:/definitely/not/here")).is_empty());
}

#[test]
fn resolves_only_plain_dir_vpk_names_that_exist() {
    let dir = scratch("resolve");
    std::fs::write(dir.join("pak01_dir.vpk"), b"x").unwrap();
    std::fs::write(dir.join("pak01_000.vpk"), b"x").unwrap();
    assert_eq!(resolve_addon(&dir, "pak01_dir.vpk"), Some(dir.join("pak01_dir.vpk")));
    assert_eq!(resolve_addon(&dir, "pak02_dir.vpk"), None);
    assert_eq!(resolve_addon(&dir, "pak01_000.vpk"), None);
    assert_eq!(resolve_addon(&dir, "../pak01_dir.vpk"), None);
    assert_eq!(resolve_addon(&dir, "sub/pak01_dir.vpk"), None);
    assert_eq!(resolve_addon(&dir, r"sub\pak01_dir.vpk"), None);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn label_prefers_search_path_then_mod_id_then_file_name() {
    assert_eq!(label(Some("citadel_addons/build_qollock"), Some("637627"), "pak01_dir.vpk"), "build_qollock");
    assert_eq!(label(None, Some("637627"), "pak01_dir.vpk"), "GameBanana mod 637627");
    assert_eq!(label(None, None, "pak01_dir.vpk"), "pak01_dir.vpk");
}
