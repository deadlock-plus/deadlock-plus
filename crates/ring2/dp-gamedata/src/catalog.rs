use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::SystemTime;

use deadlock_data::{HeroCatalog, ItemCatalog};
use serde::Serialize;
use ts_rs::TS;

use crate::locale::game_language;

pub trait Install: Send + Sync + 'static {
    fn citadel_dir(&self) -> Option<PathBuf>;
}

pub struct SteamInstall;

impl Install for SteamInstall {
    fn citadel_dir(&self) -> Option<PathBuf> {
        dp_steam::game_install_dir().map(|dir| dir.join("game").join("citadel"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct HeroEntry {
    pub id: u32,
    pub class_name: String,
    pub name: String,
    pub localised: bool,
    pub selectable: bool,
    pub in_development: bool,
    pub disabled: bool,
    pub pre_release: bool,
    pub portrait: Option<String>,
    pub card: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ItemEntry {
    pub id: u32,
    pub class_name: String,
    pub name: String,
    pub localised: bool,
    pub kind: String,
}

struct Loaded {
    dir: Option<PathBuf>,
    stamp: Option<SystemTime>,
    heroes: HeroCatalog,
    items: Option<ItemCatalog>,
    hero_lists: HashMap<&'static str, Arc<Vec<HeroEntry>>>,
    item_lists: HashMap<&'static str, Arc<Vec<ItemEntry>>>,
}

struct Shared<I> {
    install: I,
    state: Mutex<Option<Loaded>>,
    builds: AtomicUsize,
}

pub struct GameData<I: Install = SteamInstall> {
    shared: Arc<Shared<I>>,
}

impl<I: Install> Clone for GameData<I> {
    fn clone(&self) -> Self {
        Self { shared: Arc::clone(&self.shared) }
    }
}

impl Default for GameData {
    fn default() -> Self {
        Self::new(SteamInstall)
    }
}

fn archive_stamp(dir: &Path) -> Option<SystemTime> {
    std::fs::metadata(dir.join("pak01_dir.vpk")).and_then(|m| m.modified()).ok()
}

impl<I: Install> GameData<I> {
    pub fn new(install: I) -> Self {
        Self { shared: Arc::new(Shared { install, state: Mutex::new(None), builds: AtomicUsize::new(0) }) }
    }

    pub fn builds(&self) -> usize {
        self.shared.builds.load(Ordering::Relaxed)
    }

    pub fn heroes(&self, locale: &str) -> Arc<Vec<HeroEntry>> {
        let language = game_language(locale);
        let mut guard = self.current();
        let loaded = guard.as_mut().expect("current() fills the state");
        if let Some(list) = loaded.hero_lists.get(language) {
            return Arc::clone(list);
        }
        let mut catalog = loaded.heroes.clone();
        if let Some(dir) = &loaded.dir {
            if let Err(e) = catalog.merge_game_dir_lang(dir, language) {
                log::debug!("hero names for {language} unavailable: {e}");
            }
        }
        let list = Arc::new(hero_entries(&catalog));
        loaded.hero_lists.insert(language, Arc::clone(&list));
        list
    }

    pub fn items(&self, locale: &str) -> Arc<Vec<ItemEntry>> {
        let language = game_language(locale);
        let mut guard = self.current();
        let loaded = guard.as_mut().expect("current() fills the state");
        if let Some(list) = loaded.item_lists.get(language) {
            return Arc::clone(list);
        }
        let base = loaded.items.get_or_insert_with(|| ItemCatalog::for_game(loaded.dir.as_deref()));
        let mut catalog = base.clone();
        if let Some(dir) = &loaded.dir {
            if let Err(e) = catalog.merge_game_dir_lang(dir, language) {
                log::debug!("item names for {language} unavailable: {e}");
            }
        }
        let list = Arc::new(item_entries(&catalog));
        loaded.item_lists.insert(language, Arc::clone(&list));
        list
    }

    fn current(&self) -> MutexGuard<'_, Option<Loaded>> {
        let mut guard = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
        let dir = self.shared.install.citadel_dir();
        let stamp = dir.as_deref().and_then(archive_stamp);
        let fresh = guard.as_ref().is_some_and(|l| l.dir == dir && l.stamp == stamp);
        if !fresh {
            self.shared.builds.fetch_add(1, Ordering::Relaxed);
            let heroes = build_heroes(dir.as_deref());
            *guard = Some(Loaded {
                dir,
                stamp,
                heroes,
                items: None,
                hero_lists: HashMap::new(),
                item_lists: HashMap::new(),
            });
        }
        guard
    }
}

fn build_heroes(dir: Option<&Path>) -> HeroCatalog {
    use deadlock_data::{Catalogs, Source};
    match dir {
        Some(dir) => Catalogs::new().sources([Source::Client, Source::Bundled]).game_dir(dir).heroes(),
        None => HeroCatalog::bundled(),
    }
}

/// Localised names can start with grammatical-gender markers such as `#|m|#` that the game
/// resolves itself; they are not part of the name.
fn display_name(raw: &str) -> String {
    let mut rest = raw;
    while let Some(tail) = rest.strip_prefix("#|") {
        match tail.split_once("|#") {
            Some((_, after)) => rest = after,
            None => break,
        }
    }
    rest.to_string()
}

fn hero_entries(catalog: &HeroCatalog) -> Vec<HeroEntry> {
    let names = catalog.names();
    catalog
        .all()
        .iter()
        .map(|hero| {
            let art = catalog.art(hero.id);
            HeroEntry {
                id: hero.id.get(),
                class_name: hero.class_name.clone(),
                name: display_name(names.get(&hero.class_name).unwrap_or(&hero.class_name)),
                localised: catalog.is_localised(hero.id),
                selectable: hero.selectable(),
                in_development: hero.in_development,
                disabled: hero.disabled,
                pre_release: hero.pre_release,
                portrait: art.and_then(|a| a.portrait.clone()),
                card: art.and_then(|a| a.card.clone()),
            }
        })
        .collect()
}

fn item_entries(catalog: &ItemCatalog) -> Vec<ItemEntry> {
    let names = catalog.names();
    catalog
        .all()
        .iter()
        .map(|item| ItemEntry {
            id: item.id.get(),
            class_name: item.class_name.clone(),
            name: display_name(names.get(&item.class_name).unwrap_or(&item.class_name)),
            localised: catalog.is_localised(item.id),
            kind: format!("{:?}", item.kind).to_lowercase(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadlock_data::localization::{LocalizationFile, HERO_NAMES_BUNDLE};
    use std::fs;

    struct FakeInstall(Mutex<Option<PathBuf>>);

    impl FakeInstall {
        fn at(dir: &Path) -> Self {
            Self(Mutex::new(Some(dir.to_path_buf())))
        }
        fn none() -> Self {
            Self(Mutex::new(None))
        }
    }

    impl Install for FakeInstall {
        fn citadel_dir(&self) -> Option<PathBuf> {
            self.0.lock().unwrap().clone()
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dp-gamedata-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_hero_names(dir: &Path, language: &str, body: &str) {
        let path = dir.join(LocalizationFile::relative_path(HERO_NAMES_BUNDLE, language));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, format!("\"lang\"\n{{\n\"Language\" \"{language}\"\n\"Tokens\"\n{{\n{body}\n}}\n}}\n"))
            .unwrap();
    }

    fn fake_archive(dir: &Path) -> PathBuf {
        let path = dir.join("pak01_dir.vpk");
        fs::write(&path, b"not a real archive").unwrap();
        path
    }

    fn find<'a>(list: &'a [HeroEntry], class: &str) -> &'a HeroEntry {
        list.iter().find(|h| h.class_name == class).unwrap_or_else(|| panic!("{class} missing"))
    }

    #[test]
    fn without_an_install_the_bundled_roster_answers_in_english() {
        let data = GameData::new(FakeInstall::none());
        let list = data.heroes("de");
        let inferno = find(&list, "hero_inferno");
        assert_eq!(inferno.id, 1);
        assert_eq!(inferno.name, "Infernus");
    }

    #[test]
    fn the_requested_locale_picks_the_installed_translation() {
        let dir = temp_dir("locale");
        write_hero_names(&dir, "german", "\"hero_inferno\" \"Infernus DE\"");
        write_hero_names(&dir, "english", "\"hero_inferno\" \"Infernus EN\"");
        let data = GameData::new(FakeInstall::at(&dir));
        assert_eq!(find(&data.heroes("de"), "hero_inferno").name, "Infernus DE");
        assert_eq!(find(&data.heroes("en"), "hero_inferno").name, "Infernus EN");
        assert_eq!(find(&data.heroes("af"), "hero_inferno").name, "Infernus EN");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn gender_markers_are_not_part_of_the_name() {
        let dir = temp_dir("gender");
        write_hero_names(&dir, "russian", "\"hero_inferno\" \"#|m|#Инфернус\"");
        write_hero_names(&dir, "english", "\"hero_inferno\" \"Infernus EN\"");
        let data = GameData::new(FakeInstall::at(&dir));
        assert_eq!(find(&data.heroes("ru"), "hero_inferno").name, "Инфернус");
        assert_eq!(display_name("#|f|##|p|#Name"), "Name");
        assert_eq!(display_name("#|broken"), "#|broken");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn a_missing_translation_file_keeps_the_english_name() {
        let dir = temp_dir("missing");
        write_hero_names(&dir, "english", "\"hero_inferno\" \"Infernus EN\"");
        let data = GameData::new(FakeInstall::at(&dir));
        assert_eq!(find(&data.heroes("ja"), "hero_inferno").name, "Infernus EN");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn art_urls_come_through_from_the_bundled_facet() {
        let data = GameData::new(FakeInstall::none());
        let list = data.heroes("en");
        let inferno = find(&list, "hero_inferno");
        assert!(inferno.portrait.is_some() || inferno.card.is_some());
    }

    #[test]
    fn the_catalog_is_built_once_until_the_archive_changes() {
        let dir = temp_dir("stamp");
        let archive = fake_archive(&dir);
        let data = GameData::new(FakeInstall::at(&dir));
        data.heroes("en");
        data.heroes("de");
        data.items("en");
        assert_eq!(data.builds(), 1);

        let file = fs::OpenOptions::new().write(true).open(&archive).unwrap();
        file.set_modified(SystemTime::now() + std::time::Duration::from_secs(60)).unwrap();
        drop(file);
        data.heroes("en");
        assert_eq!(data.builds(), 2);
        data.heroes("en");
        assert_eq!(data.builds(), 2);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn a_new_install_is_picked_up_after_none() {
        let dir = temp_dir("late");
        write_hero_names(&dir, "english", "\"hero_inferno\" \"Infernus Installed\"");
        fake_archive(&dir);
        let data = GameData::new(FakeInstall::none());
        assert_eq!(find(&data.heroes("en"), "hero_inferno").name, "Infernus");
        *data.shared.install.0.lock().unwrap() = Some(dir.clone());
        assert_eq!(find(&data.heroes("en"), "hero_inferno").name, "Infernus Installed");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn items_are_named_and_typed() {
        let data = GameData::new(FakeInstall::none());
        let items = data.items("en");
        assert!(items.len() > 100);
        assert!(items.iter().any(|i| i.kind == "upgrade" && i.localised));
        assert!(items.iter().any(|i| i.kind == "ability"));
    }
}
