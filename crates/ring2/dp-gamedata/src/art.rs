use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;

use deadlock_data::art::{Art, ArtArchive, HeroArtKind, ItemArtKind, RankArtKind};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankImage {
    Lg,
    Chalk,
}

impl RankImage {
    const ALL: [RankImage; 2] = [RankImage::Lg, RankImage::Chalk];

    fn suffix(self) -> &'static str {
        match self {
            RankImage::Lg => "lg",
            RankImage::Chalk => "chalk",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum ItemImage {
    Icon,
    Shop,
}

impl ItemImage {
    fn suffix(self) -> &'static str {
        match self {
            ItemImage::Icon => "icon",
            ItemImage::Shop => "shop",
        }
    }
}

/// Most class names one `ability_art` / `item_art` call decodes; the rest are ignored.
pub const MAX_CLASS_BATCH: usize = 200;

/// Where decoded images come from. Each `write_*` returns whether `dest` now holds a complete PNG;
/// a missing texture and an undecodable one are both `false`.
pub trait ArtSource: Send + Sync + 'static {
    fn write_hero(&self, class_name: &str, image: HeroImage, dest: &Path) -> bool;
    fn write_rank(&self, tier: u8, image: RankImage, dest: &Path) -> bool;
    fn write_ability(&self, class_name: &str, dest: &Path) -> bool;
    fn write_item(&self, class_name: &str, image: ItemImage, dest: &Path) -> bool;
}

struct ArchiveSource(ArtArchive);

impl ArchiveSource {
    fn write(&self, art: &Art<'_>, dest: &Path, what: &str) -> bool {
        if !self.0.contains(art) {
            return false;
        }
        match self.0.write_png(art, dest) {
            Ok(()) => true,
            Err(e) => {
                log::debug!("no local art for {what}: {e}");
                false
            }
        }
    }
}

impl ArtSource for ArchiveSource {
    fn write_hero(&self, class_name: &str, image: HeroImage, dest: &Path) -> bool {
        let kind = match image {
            HeroImage::Sm => HeroArtKind::Sm,
            HeroImage::Card => HeroArtKind::Card,
        };
        self.write(&Art::hero(class_name, kind), dest, &format!("{class_name} {}", image.suffix()))
    }

    fn write_rank(&self, tier: u8, image: RankImage, dest: &Path) -> bool {
        let kind = match image {
            RankImage::Lg => RankArtKind::Large,
            RankImage::Chalk => RankArtKind::Chalk,
        };
        self.write(&Art::rank(tier, kind), dest, &format!("rank {tier} {}", image.suffix()))
    }

    fn write_ability(&self, class_name: &str, dest: &Path) -> bool {
        self.write(&Art::ability(class_name), dest, class_name)
    }

    fn write_item(&self, class_name: &str, image: ItemImage, dest: &Path) -> bool {
        let kind = match image {
            ItemImage::Icon => ItemArtKind::Icon,
            ItemImage::Shop => ItemArtKind::Shop,
        };
        self.write(&Art::item(class_name, kind), dest, &format!("{class_name} {}", image.suffix()))
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RankArt {
    pub tier: u8,
    pub lg: Option<String>,
    pub chalk: Option<String>,
}

/// A local image for an ability or item, by its class name in the game's ability table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ClassArt {
    pub class_name: String,
    pub path: String,
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
        self.with_build(|build, build_dir| {
            let mut out = Vec::new();
            for hero in heroes {
                if !file_safe(&hero.class_name) {
                    continue;
                }
                let mut entry = HeroArt { id: hero.id, sm: None, card: None };
                for image in HeroImage::ALL {
                    let name = format!("{}_{}.png", hero.class_name, image.suffix());
                    let found = self.ensure(build, build_dir, name, |s, p| s.write_hero(&hero.class_name, image, p));
                    match image {
                        HeroImage::Sm => entry.sm = found,
                        HeroImage::Card => entry.card = found,
                    }
                }
                if entry.sm.is_some() || entry.card.is_some() {
                    out.push(entry);
                }
            }
            out
        })
        .unwrap_or_default()
    }

    /// Local `lg` and `chalk` badge files per rank tier, decoding what the cache lacks. Tiers with no
    /// local badge are left out; this never fails.
    pub fn rank_art(&self, tiers: &[u8]) -> Vec<RankArt> {
        self.with_build(|build, build_dir| {
            let mut out = Vec::new();
            for &tier in tiers {
                let mut entry = RankArt { tier, lg: None, chalk: None };
                for image in RankImage::ALL {
                    let name = format!("rank{tier:02}_{}.png", image.suffix());
                    let found = self.ensure(build, build_dir, name, |s, p| s.write_rank(tier, image, p));
                    match image {
                        RankImage::Lg => entry.lg = found,
                        RankImage::Chalk => entry.chalk = found,
                    }
                }
                if entry.lg.is_some() || entry.chalk.is_some() {
                    out.push(entry);
                }
            }
            out
        })
        .unwrap_or_default()
    }

    /// Local icons for the named abilities, decoding what the cache lacks. Abilities whose icon is
    /// missing, vector-only or undecodable are left out; this never fails.
    pub fn ability_art(&self, class_names: &[String]) -> Vec<ClassArt> {
        self.class_art(class_names, |class| format!("ability_{class}.png"), |s, class, p| s.write_ability(class, p))
    }

    /// Local icon or shop images for the named items, with the same rules as `ability_art`.
    pub fn item_art(&self, class_names: &[String], image: ItemImage) -> Vec<ClassArt> {
        self.class_art(
            class_names,
            |class| format!("item_{class}_{}.png", image.suffix()),
            |s, class, p| s.write_item(class, image, p),
        )
    }

    fn class_art(
        &self,
        class_names: &[String],
        file_name: impl Fn(&str) -> String,
        write: impl Fn(&dyn ArtSource, &str, &Path) -> bool,
    ) -> Vec<ClassArt> {
        self.with_build(|build, build_dir| {
            let mut seen = HashSet::new();
            let mut out = Vec::new();
            for class in class_names.iter().take(MAX_CLASS_BATCH) {
                if !file_safe(class) || !seen.insert(class.as_str()) {
                    continue;
                }
                let found = self.ensure(build, build_dir, file_name(class), |s, p| write(s, class, p));
                if let Some(path) = found {
                    out.push(ClassArt { class_name: class.clone(), path });
                }
            }
            out
        })
        .unwrap_or_default()
    }

    /// Runs `f` on the current build and its directory, starting a fresh build when the game changed.
    /// `None` when there is no installed game to decode from.
    fn with_build<R>(&self, f: impl FnOnce(&mut Build, &Path) -> R) -> Option<R> {
        let mut guard = self.shared.state.lock().unwrap_or_else(|e| e.into_inner());
        let dir = self.shared.install.citadel_dir();
        let fingerprint = dir.as_deref().and_then(fingerprint);
        let stale = guard.as_ref().is_none_or(|b| b.dir != dir || b.fingerprint != fingerprint);
        if stale {
            self.evict_except(fingerprint.as_deref());
            *guard = Some(Build { dir, fingerprint, source: None, failed: HashSet::new() });
        }
        let build = guard.as_mut().expect("set above");
        let build_dir = build.fingerprint.as_ref().map(|f| self.shared.root.join(f))?;
        Some(f(build, &build_dir))
    }

    fn ensure(
        &self,
        build: &mut Build,
        build_dir: &Path,
        name: String,
        write: impl FnOnce(&dyn ArtSource, &Path) -> bool,
    ) -> Option<String> {
        let path = build_dir.join(&name);
        let ready = path.is_file() || {
            !build.failed.contains(&name) && {
                let citadel = build.dir.clone();
                let source =
                    build.source.get_or_insert_with(|| citadel.as_deref().and_then(|dir| (self.shared.opener)(dir)));
                let written = source.as_deref().is_some_and(|s| write(s, &path));
                if !written {
                    build.failed.insert(name);
                }
                written
            }
        };
        ready.then(|| path.to_string_lossy().into_owned())
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
        failing_tiers: HashSet<u8>,
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

        fn write_rank(&self, tier: u8, _image: RankImage, dest: &Path) -> bool {
            self.writes.fetch_add(1, Ordering::Relaxed);
            if self.failing_tiers.contains(&tier) {
                return false;
            }
            fs::create_dir_all(dest.parent().unwrap()).unwrap();
            fs::write(dest, b"png").is_ok()
        }

        fn write_ability(&self, class_name: &str, dest: &Path) -> bool {
            self.write_class(class_name, dest)
        }

        fn write_item(&self, class_name: &str, _image: ItemImage, dest: &Path) -> bool {
            self.write_class(class_name, dest)
        }
    }

    impl FakeSource {
        fn write_class(&self, class_name: &str, dest: &Path) -> bool {
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

        fn write_rank(&self, tier: u8, image: RankImage, dest: &Path) -> bool {
            self.0.write_rank(tier, image, dest)
        }

        fn write_ability(&self, class_name: &str, dest: &Path) -> bool {
            self.0.write_ability(class_name, dest)
        }

        fn write_item(&self, class_name: &str, image: ItemImage, dest: &Path) -> bool {
            self.0.write_item(class_name, image, dest)
        }
    }

    struct ChalkOnly(Arc<FakeSource>);

    impl ArtSource for ChalkOnly {
        fn write_hero(&self, class_name: &str, image: HeroImage, dest: &Path) -> bool {
            self.0.write_hero(class_name, image, dest)
        }

        fn write_rank(&self, tier: u8, image: RankImage, dest: &Path) -> bool {
            image == RankImage::Chalk && self.0.write_rank(tier, image, dest)
        }

        fn write_ability(&self, class_name: &str, dest: &Path) -> bool {
            self.0.write_ability(class_name, dest)
        }

        fn write_item(&self, class_name: &str, image: ItemImage, dest: &Path) -> bool {
            self.0.write_item(class_name, image, dest)
        }
    }

    #[test]
    fn rank_badges_are_files_named_by_tier_beside_the_hero_art() {
        let rig = Rig::new("rank-paths", &[]);
        let cache = rig.cache();
        let hero = cache.hero_art(&[hero(1, "hero_inferno")]);
        let ranks = cache.rank_art(&[5, 11]);
        assert_eq!(ranks.iter().map(|r| r.tier).collect::<Vec<_>>(), vec![5, 11]);
        let lg = PathBuf::from(ranks[0].lg.as_ref().unwrap());
        let chalk = PathBuf::from(ranks[0].chalk.as_ref().unwrap());
        assert_eq!(lg.file_name().unwrap(), "rank05_lg.png");
        assert_eq!(chalk.file_name().unwrap(), "rank05_chalk.png");
        assert!(lg.is_file() && chalk.is_file());
        assert_eq!(lg.parent(), PathBuf::from(hero[0].sm.as_ref().unwrap()).parent());
    }

    #[test]
    fn rank_art_is_decoded_once_per_build() {
        let rig = Rig::new("rank-idempotent", &[]);
        let cache = rig.cache();
        let first = cache.rank_art(&[3]);
        assert_eq!(rig.source.writes.load(Ordering::Relaxed), 2);
        assert_eq!(cache.rank_art(&[3]), first);
        assert_eq!(rig.source.writes.load(Ordering::Relaxed), 2);
        assert_eq!(rig.cache().rank_art(&[3]), first);
        assert_eq!(rig.source.writes.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn a_tier_with_no_badge_is_left_out_and_not_retried() {
        let rig = Rig::new("rank-missing", &[]);
        let source = Arc::new(FakeSource { failing_tiers: HashSet::from([4]), ..Default::default() });
        let counted = Arc::clone(&source);
        let cache = ArtCache::with_opener(
            FakeInstall(Mutex::new(Some(rig.citadel.clone()))),
            rig.root.clone(),
            Box::new(move |_| Some(Arc::clone(&source) as Arc<dyn ArtSource>)),
        );
        let ranks = cache.rank_art(&[3, 4]);
        assert_eq!(ranks.iter().map(|r| r.tier).collect::<Vec<_>>(), vec![3]);
        let writes = counted.writes.load(Ordering::Relaxed);
        cache.rank_art(&[3, 4]);
        assert_eq!(counted.writes.load(Ordering::Relaxed), writes);
    }

    #[test]
    fn one_badge_failing_keeps_the_other() {
        let rig = Rig::new("rank-half", &[]);
        let source = Arc::new(FakeSource::default());
        let cache = ArtCache::with_opener(
            FakeInstall(Mutex::new(Some(rig.citadel.clone()))),
            rig.root.clone(),
            Box::new(move |_| Some(Arc::new(ChalkOnly(Arc::clone(&source))) as Arc<dyn ArtSource>)),
        );
        let ranks = cache.rank_art(&[1]);
        assert_eq!(ranks.len(), 1);
        assert!(ranks[0].lg.is_none());
        assert!(ranks[0].chalk.is_some());
    }

    #[test]
    fn without_an_install_there_are_no_rank_badges() {
        let rig = Rig::new("rank-noinstall", &[]);
        let cache =
            ArtCache::with_opener(FakeInstall(Mutex::new(None)), rig.root.clone(), Box::new(|_| panic!("no install")));
        assert!(cache.rank_art(&[1]).is_empty());
    }

    fn names(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn ability_and_item_icons_are_files_in_the_build_directory_without_name_clashes() {
        let rig = Rig::new("class-paths", &[]);
        let cache = rig.cache();
        let hero = cache.hero_art(&[hero(1, "hero_inferno")]);
        let ability = cache.ability_art(&names(&["ability_dash"]));
        let icon = cache.item_art(&names(&["upgrade_a"]), ItemImage::Icon);
        let shop = cache.item_art(&names(&["upgrade_a"]), ItemImage::Shop);
        let file = |a: &[ClassArt]| PathBuf::from(&a[0].path);
        assert_eq!(a_name(&ability[0]), "ability_dash");
        assert_eq!(file(&ability).file_name().unwrap(), "ability_ability_dash.png");
        assert_eq!(file(&icon).file_name().unwrap(), "item_upgrade_a_icon.png");
        assert_eq!(file(&shop).file_name().unwrap(), "item_upgrade_a_shop.png");
        for f in [file(&ability), file(&icon), file(&shop)] {
            assert!(f.is_file());
            assert_eq!(f.parent(), PathBuf::from(hero[0].sm.as_ref().unwrap()).parent());
        }
    }

    fn a_name(a: &ClassArt) -> &str {
        &a.class_name
    }

    #[test]
    fn only_the_requested_classes_are_decoded_and_each_once() {
        let rig = Rig::new("class-lazy", &[]);
        let cache = rig.cache();
        let first = cache.item_art(&names(&["upgrade_a", "upgrade_b"]), ItemImage::Icon);
        assert_eq!(first.len(), 2);
        assert_eq!(rig.source.writes.load(Ordering::Relaxed), 2);
        cache.item_art(&names(&["upgrade_b", "upgrade_c"]), ItemImage::Icon);
        assert_eq!(rig.source.writes.load(Ordering::Relaxed), 3);
        assert_eq!(rig.cache().item_art(&names(&["upgrade_a"]), ItemImage::Icon), first[..1]);
        assert_eq!(rig.source.writes.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn classes_without_a_local_image_are_left_out_and_not_retried() {
        let rig = Rig::new("class-missing", &["ability_svg"]);
        let cache = rig.cache();
        let out = cache.ability_art(&names(&["ability_svg", "ability_png"]));
        assert_eq!(out.iter().map(|a| a.class_name.as_str()).collect::<Vec<_>>(), vec!["ability_png"]);
        let writes = rig.source.writes.load(Ordering::Relaxed);
        cache.ability_art(&names(&["ability_svg"]));
        assert_eq!(rig.source.writes.load(Ordering::Relaxed), writes);
    }

    #[test]
    fn class_batches_are_bounded_deduplicated_and_file_safe() {
        let rig = Rig::new("class-bound", &[]);
        let cache = rig.cache();
        let many: Vec<String> = (0..MAX_CLASS_BATCH + 50).map(|i| format!("upgrade_{i}")).collect();
        assert_eq!(cache.item_art(&many, ItemImage::Icon).len(), MAX_CLASS_BATCH);
        let out = cache.ability_art(&names(&["../evil", "ability_a", "ability_a", ""]));
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn without_an_install_there_are_no_class_images() {
        let rig = Rig::new("class-noinstall", &[]);
        let cache =
            ArtCache::with_opener(FakeInstall(Mutex::new(None)), rig.root.clone(), Box::new(|_| panic!("no install")));
        assert!(cache.ability_art(&names(&["ability_a"])).is_empty());
        assert!(cache.item_art(&names(&["upgrade_a"]), ItemImage::Shop).is_empty());
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
