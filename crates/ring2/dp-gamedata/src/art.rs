use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;

use deadlock_data::art::{Art, ArtArchive, HeroArtKind};
use serde::Serialize;
use ts_rs::TS;

use crate::catalog::{archive_stamp, HeroEntry, Install, SteamInstall};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeroImage {
    Sm,
    Card,
}

impl HeroImage {
    const ALL: [HeroImage; 2] = [HeroImage::Sm, HeroImage::Card];

    fn suffix(self) -> &'static str {
        match self {
            HeroImage::Sm => "sm",
            HeroImage::Card => "card",
        }
    }
}

/// Where decoded images come from. `write_hero` returns whether `dest` now holds a complete PNG;
/// a missing texture and an undecodable one are both `false`.
pub trait ArtSource: Send + Sync + 'static {
    fn write_hero(&self, class_name: &str, image: HeroImage, dest: &Path) -> bool;
}

struct ArchiveSource(ArtArchive);

impl ArtSource for ArchiveSource {
    fn write_hero(&self, class_name: &str, image: HeroImage, dest: &Path) -> bool {
        let kind = match image {
            HeroImage::Sm => HeroArtKind::Sm,
            HeroImage::Card => HeroArtKind::Card,
        };
        let art = Art::hero(class_name, kind);
        if !self.0.contains(&art) {
            return false;
        }
        match self.0.write_png(&art, dest) {
            Ok(()) => true,
            Err(e) => {
                log::debug!("no local art for {class_name} {}: {e}", image.suffix());
                false
            }
        }
    }
}

type Opener = Box<dyn Fn(&Path) -> Option<Arc<dyn ArtSource>> + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct HeroArt {
    pub id: u32,
    pub sm: Option<String>,
    pub card: Option<String>,
}

struct Build {
    dir: Option<PathBuf>,
    fingerprint: Option<String>,
    source: Option<Option<Arc<dyn ArtSource>>>,
    failed: HashSet<String>,
}

struct Shared<I> {
    install: I,
    root: PathBuf,
    opener: Opener,
    state: Mutex<Option<Build>>,
}

/// PNG files decoded from the installed game, one directory per game build under `root`.
pub struct ArtCache<I: Install = SteamInstall> {
    shared: Arc<Shared<I>>,
}

impl<I: Install> Clone for ArtCache<I> {
    fn clone(&self) -> Self {
        Self { shared: Arc::clone(&self.shared) }
    }
}

impl ArtCache {
    pub fn new(root: PathBuf) -> Self {
        Self::with_opener(SteamInstall, root, Box::new(open_archive))
    }
}

fn open_archive(citadel_dir: &Path) -> Option<Arc<dyn ArtSource>> {
    match ArtArchive::open(citadel_dir) {
        Ok(archive) => Some(Arc::new(ArchiveSource(archive))),
        Err(e) => {
            log::debug!("game art unavailable: {e}");
            None
        }
    }
}

fn fingerprint(dir: &Path) -> Option<String> {
    let nanos = archive_stamp(dir)?.duration_since(UNIX_EPOCH).ok()?.as_nanos();
    Some(format!("{nanos:x}"))
}

fn file_safe(class_name: &str) -> bool {
    !class_name.is_empty() && class_name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

impl<I: Install> ArtCache<I> {
    pub fn with_opener(install: I, root: PathBuf, opener: Opener) -> Self {
        Self { shared: Arc::new(Shared { install, root, opener, state: Mutex::new(None) }) }
    }

    /// Local `sm` and `card` files per hero, decoding what the cache lacks. Heroes with no local
    /// image are left out; this never fails.
    pub fn hero_art(&self, heroes: &[HeroEntry]) -> Vec<HeroArt> {
        let mut guard = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
        let dir = self.shared.install.citadel_dir();
        let fingerprint = dir.as_deref().and_then(fingerprint);
        let stale = guard.as_ref().is_none_or(|b| b.dir != dir || b.fingerprint != fingerprint);
        if stale {
            self.evict_except(fingerprint.as_deref());
            *guard = Some(Build { dir, fingerprint, source: None, failed: HashSet::new() });
        }
        let build = guard.as_mut().expect("set above");
        let Some(build_dir) = build.fingerprint.as_ref().map(|f| self.shared.root.join(f)) else {
            return Vec::new();
        };
        let citadel = build.dir.clone();

        let mut out = Vec::new();
        for hero in heroes {
            if !file_safe(&hero.class_name) {
                continue;
            }
            let mut entry = HeroArt { id: hero.id, sm: None, card: None };
            for image in HeroImage::ALL {
                let name = format!("{}_{}.png", hero.class_name, image.suffix());
                let path = build_dir.join(&name);
                let ready = path.is_file() || {
                    !build.failed.contains(&name) && {
                        let source = build
                            .source
                            .get_or_insert_with(|| citadel.as_deref().and_then(|dir| (self.shared.opener)(dir)));
                        let written = source.as_ref().is_some_and(|s| s.write_hero(&hero.class_name, image, &path));
                        if !written {
                            build.failed.insert(name);
                        }
                        written
                    }
                };
                if ready {
                    let text = path.to_string_lossy().into_owned();
                    match image {
                        HeroImage::Sm => entry.sm = Some(text),
                        HeroImage::Card => entry.card = Some(text),
                    }
                }
            }
            if entry.sm.is_some() || entry.card.is_some() {
                out.push(entry);
            }
        }
        out
    }

    fn evict_except(&self, keep: Option<&str>) {
        let Ok(entries) = std::fs::read_dir(&self.shared.root) else { return };
        for entry in entries.flatten() {
            if Some(entry.file_name().to_string_lossy().as_ref()) == keep {
                continue;
            }
            let path = entry.path();
            let removed = if path.is_dir() { std::fs::remove_dir_all(&path) } else { std::fs::remove_file(&path) };
            if let Err(e) = removed {
                log::debug!("could not remove old art {}: {e}", path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{HeroEntry, Install};
    use std::collections::HashSet;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::SystemTime;

    struct FakeInstall(Mutex<Option<PathBuf>>);

    impl Install for FakeInstall {
        fn citadel_dir(&self) -> Option<PathBuf> {
            self.0.lock().unwrap().clone()
        }
    }

    #[derive(Default)]
    struct FakeSource {
        writes: AtomicUsize,
        failing: HashSet<&'static str>,
    }

    impl ArtSource for FakeSource {
        fn write_hero(&self, class_name: &str, _image: HeroImage, dest: &Path) -> bool {
            self.writes.fetch_add(1, Ordering::Relaxed);
            if self.failing.contains(class_name) {
                return false;
            }
            fs::create_dir_all(dest.parent().unwrap()).unwrap();
            fs::write(dest, b"png").is_ok()
        }
    }

    struct Rig {
        base: PathBuf,
        citadel: PathBuf,
        root: PathBuf,
        source: Arc<FakeSource>,
        opens: Arc<AtomicUsize>,
    }

    impl Rig {
        fn new(name: &str, failing: &[&'static str]) -> Self {
            let base = std::env::temp_dir().join(format!("dp-gamedata-art-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&base);
            let citadel = base.join("citadel");
            let root = base.join("cache");
            fs::create_dir_all(&citadel).unwrap();
            fs::write(citadel.join("pak01_dir.vpk"), b"x").unwrap();
            let source = Arc::new(FakeSource { failing: failing.iter().copied().collect(), ..Default::default() });
            Self { base, citadel, root, source, opens: Arc::new(AtomicUsize::new(0)) }
        }

        fn cache(&self) -> ArtCache<FakeInstall> {
            let source = Arc::clone(&self.source);
            let opens = Arc::clone(&self.opens);
            ArtCache::with_opener(
                FakeInstall(Mutex::new(Some(self.citadel.clone()))),
                self.root.clone(),
                Box::new(move |_| {
                    opens.fetch_add(1, Ordering::Relaxed);
                    Some(Arc::clone(&source) as Arc<dyn ArtSource>)
                }),
            )
        }

        fn touch_archive(&self, secs: u64) {
            let file = fs::OpenOptions::new().write(true).open(self.citadel.join("pak01_dir.vpk")).unwrap();
            file.set_modified(SystemTime::now() + std::time::Duration::from_secs(secs)).unwrap();
        }
    }

    impl Drop for Rig {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.base).ok();
        }
    }

    fn hero(id: u32, class: &str) -> HeroEntry {
        HeroEntry {
            id,
            class_name: class.to_string(),
            name: class.to_string(),
            localised: true,
            selectable: true,
            in_development: false,
            disabled: false,
            pre_release: false,
            portrait: None,
            card: None,
        }
    }

    #[test]
    fn decoded_art_is_returned_as_files_under_the_build_directory() {
        let rig = Rig::new("paths", &[]);
        let art = rig.cache().hero_art(&[hero(1, "hero_inferno")]);
        assert_eq!(art.len(), 1);
        assert_eq!(art[0].id, 1);
        let sm = PathBuf::from(art[0].sm.as_ref().unwrap());
        let card = PathBuf::from(art[0].card.as_ref().unwrap());
        assert!(sm.is_file() && card.is_file());
        assert!(sm.starts_with(&rig.root) && card.starts_with(&rig.root));
        assert_eq!(sm.file_name().unwrap(), "hero_inferno_sm.png");
        assert_eq!(card.file_name().unwrap(), "hero_inferno_card.png");
        assert_eq!(sm.parent(), card.parent());
        assert_ne!(sm.parent().unwrap(), rig.root);
    }

    #[test]
    fn a_second_call_decodes_nothing_more() {
        let rig = Rig::new("idempotent", &[]);
        let cache = rig.cache();
        let heroes = [hero(1, "hero_inferno")];
        let first = cache.hero_art(&heroes);
        let writes = rig.source.writes.load(Ordering::Relaxed);
        assert_eq!(writes, 2);
        assert_eq!(cache.hero_art(&heroes), first);
        assert_eq!(rig.source.writes.load(Ordering::Relaxed), writes);
        assert_eq!(rig.opens.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn cached_files_serve_a_new_session_without_opening_the_archive() {
        let rig = Rig::new("warm", &[]);
        let heroes = [hero(1, "hero_inferno")];
        let first = rig.cache().hero_art(&heroes);
        let opens = rig.opens.load(Ordering::Relaxed);
        assert_eq!(rig.cache().hero_art(&heroes), first);
        assert_eq!(rig.opens.load(Ordering::Relaxed), opens);
        assert_eq!(rig.source.writes.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn a_failed_decode_means_no_local_art_for_that_hero_only() {
        let rig = Rig::new("failing", &["hero_vindicta"]);
        let cache = rig.cache();
        let heroes = [hero(1, "hero_inferno"), hero(2, "hero_vindicta")];
        let art = cache.hero_art(&heroes);
        assert_eq!(art.len(), 1);
        assert_eq!(art[0].id, 1);
        let writes = rig.source.writes.load(Ordering::Relaxed);
        assert_eq!(cache.hero_art(&heroes), art);
        assert_eq!(rig.source.writes.load(Ordering::Relaxed), writes, "failures are not retried within a build");
    }

    #[test]
    fn one_image_failing_keeps_the_other() {
        let rig = Rig::new("half", &[]);
        let source = Arc::new(FakeSource::default());
        let cache = ArtCache::with_opener(
            FakeInstall(Mutex::new(Some(rig.citadel.clone()))),
            rig.root.clone(),
            Box::new(move |_| Some(Arc::new(CardOnly(Arc::clone(&source))) as Arc<dyn ArtSource>)),
        );
        let art = cache.hero_art(&[hero(1, "hero_inferno")]);
        assert_eq!(art.len(), 1);
        assert!(art[0].sm.is_none());
        assert!(art[0].card.is_some());
    }

    struct CardOnly(Arc<FakeSource>);

    impl ArtSource for CardOnly {
        fn write_hero(&self, class_name: &str, image: HeroImage, dest: &Path) -> bool {
            image == HeroImage::Card && self.0.write_hero(class_name, image, dest)
        }
    }

    #[test]
    fn a_new_build_evicts_older_builds_and_redecodes() {
        let rig = Rig::new("evict", &[]);
        let cache = rig.cache();
        let heroes = [hero(1, "hero_inferno")];
        let old = cache.hero_art(&heroes);
        let old_dir = PathBuf::from(old[0].sm.clone().unwrap()).parent().unwrap().to_path_buf();
        rig.touch_archive(120);
        let new = cache.hero_art(&heroes);
        let new_dir = PathBuf::from(new[0].sm.clone().unwrap()).parent().unwrap().to_path_buf();
        assert_ne!(old_dir, new_dir);
        assert!(!old_dir.exists());
        assert!(new_dir.join("hero_inferno_sm.png").is_file());
        assert_eq!(fs::read_dir(&rig.root).unwrap().count(), 1);
    }

    #[test]
    fn stale_builds_left_on_disk_are_removed_on_first_use() {
        let rig = Rig::new("leftover", &[]);
        let stale = rig.root.join("0-0");
        fs::create_dir_all(&stale).unwrap();
        fs::write(stale.join("x.png"), b"png").unwrap();
        rig.cache().hero_art(&[hero(1, "hero_inferno")]);
        assert!(!stale.exists());
    }

    #[test]
    fn without_an_install_there_is_no_local_art() {
        let rig = Rig::new("noinstall", &[]);
        let cache =
            ArtCache::with_opener(FakeInstall(Mutex::new(None)), rig.root.clone(), Box::new(|_| panic!("no install")));
        assert!(cache.hero_art(&[hero(1, "hero_inferno")]).is_empty());
    }

    #[test]
    fn an_archive_that_will_not_open_means_no_local_art() {
        let rig = Rig::new("noopen", &[]);
        let cache = ArtCache::with_opener(
            FakeInstall(Mutex::new(Some(rig.citadel.clone()))),
            rig.root.clone(),
            Box::new(|_| None),
        );
        assert!(cache.hero_art(&[hero(1, "hero_inferno")]).is_empty());
    }

    #[test]
    fn class_names_that_are_not_file_safe_get_no_art() {
        let rig = Rig::new("unsafe", &[]);
        let art = rig.cache().hero_art(&[hero(1, "../evil"), hero(2, "hero_a\\b"), hero(3, "hero_inferno")]);
        assert_eq!(art.iter().map(|a| a.id).collect::<Vec<_>>(), vec![3]);
    }
}
