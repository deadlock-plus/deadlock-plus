use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::bbcode;
use crate::embed::{Embedder, EMBEDDING_DIM};
use crate::parse::{parse_body, section_header, PatchLine};
use dp_versioned::{self, Migration};

/// Which feed a patch's content actually came from. Steam is preferred: it's the fuller, better
/// formatted source, so a patch keeps `Forum` only until a real Steam News post is found for it
/// (see `reconcile_steam_news`), and never goes back once it has one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "lowercase")]
pub enum PatchOrigin {
    #[default]
    Forum,
    Steam,
}

/// The bits of a patch identity `patch_notes` needs, independent of whichever feed found it.
/// `alerts`' `/v2/patches` ingest and `steam_news`'s `ISteamNews` ingest each build this from
/// their own item shape.
pub struct PatchSource {
    pub id: String,
    pub title: String,
    pub published: String,
    pub link: String,
    pub origin: PatchOrigin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedLine {
    pub line: PatchLine,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedPatch {
    pub id: String,
    pub title: String,
    pub published: String,
    pub link: String,
    /// Defaults to `Forum` for index files written before this field existed: they'll pick up
    /// `Steam` naturally the next time a real Steam News post reconciles against them.
    #[serde(default)]
    pub origin: PatchOrigin,
    pub lines: Vec<IndexedLine>,
    /// Resolved, Steam-hosted `[img]` URLs pulled out of the body (see `bbcode::strip_bbcode`).
    /// Only ever populated from a Steam News fetch — the `/v2/patches` HTML feed's images aren't
    /// collected here. Defaults to empty for index files written before this field existed.
    #[serde(default)]
    pub images: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Index {
    pub patches: Vec<IndexedPatch>,
}

/// A patch's full content for the in-app viewer: every line, structured, with no embeddings.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PatchDetail {
    pub id: String,
    pub title: String,
    pub published: String,
    pub link: String,
    pub origin: PatchOrigin,
    pub lines: Vec<PatchLine>,
    pub images: Vec<String>,
}

impl From<&IndexedPatch> for PatchDetail {
    fn from(patch: &IndexedPatch) -> Self {
        PatchDetail {
            id: patch.id.clone(),
            title: patch.title.clone(),
            published: patch.published.clone(),
            link: patch.link.clone(),
            origin: patch.origin,
            lines: patch.lines.iter().map(|l| l.line.clone()).collect(),
            images: patch.images.clone(),
        }
    }
}

const MIGRATIONS: &[Migration] = &[];
pub const FILE_NAME: &str = "patch-notes-index.json";

pub fn load(path: &Path) -> Index {
    let mut index = dp_versioned::read(path, MIGRATIONS)
        .unwrap_or_else(|e| {
            log::warn!("could not read {FILE_NAME}, starting empty: {e}");
            None
        })
        .unwrap_or_default();
    backfill_legacy_origin(&mut index);
    backfill_section_headers(&mut index);
    index
}

/// `section_re` didn't used to tolerate the `**...**` a bold-wrapped header gets after
/// `strip_bbcode` (see that regex's own doc comment), so patches indexed before that fix have the
/// header itself stuck in `lines` as ordinary body text, under whatever section came before it —
/// and every real line that followed it wrongly stuck at that same stale section. Re-checks every
/// already-indexed line against the fixed rule and, same as `backfill_legacy_origin`, fixes it up
/// in memory only: cheap enough to redo on every load, and it only ever removes a line that was
/// never real content to begin with.
fn backfill_section_headers(index: &mut Index) {
    for patch in &mut index.patches {
        let mut section: Option<String> = None;
        patch.lines.retain_mut(|indexed| {
            if let Some(name) = section_header(&indexed.line.raw) {
                section = Some(name);
                return false;
            }
            if let Some(name) = &section {
                indexed.line.section = name.clone();
            }
            true
        });
    }
}

/// `origin` was added to `IndexedPatch` after real patches were already on disk; `#[serde(default)]`
/// silently reads every one of them back as `Forum`, including genuinely full Steam-sourced posts —
/// confirmed live against a real index file predating the field, where all 51 stored patches came
/// back `Forum`, several with 100+ real lines. `id` is a reliable stand-in for the origin a patch was
/// actually ingested with and never changes afterwards: a genuine Steam-sourced item's id is always
/// either the dedicated `steam_news` module's `steam-news:` prefix (see `steam_news::id_of`) or the
/// Steam post's own `store.steampowered.com` link (see `alerts::feed::parse_item`), never something
/// a forum-only item's id could produce. In-memory only, not written back — cheap enough to redo on
/// every load, and it never needs to touch `images` (those can't be recovered from what's already
/// stored; only a real re-fetch can supply them).
fn backfill_legacy_origin(index: &mut Index) {
    for patch in &mut index.patches {
        if patch.origin == PatchOrigin::Forum && is_steam_sourced_id(&patch.id) {
            patch.origin = PatchOrigin::Steam;
        }
    }
}

fn is_steam_sourced_id(id: &str) -> bool {
    id.starts_with("steam-news:") || id.contains("store.steampowered.com/news/app/")
}

pub fn save(path: &Path, index: &Index) -> std::io::Result<()> {
    dp_versioned::write(path, MIGRATIONS, index)
}

/// Embeds `parsed` against what's already indexed for this patch, reusing an existing line's
/// embedding whenever its full parsed content (not just its `raw` text — the pool key is only a
/// fast lookup) is unchanged, and only calling the embedder for what's new or different. Passing
/// an empty `existing` (a brand new patch) embeds everything, since nothing can be reused.
/// `on_embed_start`/`on_embed_done` bracket each such call, so a caller can track progress without
/// any of this touching shared state itself.
fn merge_lines(
    source: &PatchSource,
    existing: &[IndexedLine],
    parsed: Vec<PatchLine>,
    embedder: &Embedder,
    on_embed_start: &mut impl FnMut(&PatchSource),
    on_embed_done: &mut impl FnMut(),
) -> Vec<IndexedLine> {
    let mut pool: HashMap<&str, Vec<&IndexedLine>> = HashMap::new();
    for line in existing {
        pool.entry(line.line.raw.as_str()).or_default().push(line);
    }
    parsed
        .into_iter()
        .filter_map(|line| {
            if let Some(bucket) = pool.get_mut(line.raw.as_str()) {
                if let Some(pos) = bucket.iter().position(|l| l.line == line) {
                    return Some(bucket.remove(pos).clone());
                }
            }
            on_embed_start(source);
            // An image marker's "text" is a positional sentinel, not real content (see
            // `bbcode::parse_image_marker`) — embedding it for real would risk it surfacing as a
            // nonsense semantic search hit. A zero vector can never cross `search::SEMANTIC_FLOOR`.
            let embedding = if bbcode::parse_image_marker(&line.raw).is_some() {
                Ok(vec![0.0; EMBEDDING_DIM])
            } else {
                embedder.embed(&line.raw)
            };
            on_embed_done();
            match embedding {
                Ok(embedding) => Some(IndexedLine { line, embedding }),
                Err(e) => {
                    log::warn!("could not embed a patch line, dropping it: {e}");
                    None
                }
            }
        })
        .collect()
}

/// Parses and embeds the body of every item whose id is not in `known`. A pure function that
/// touches no shared state and holds no lock: embedding a whole feed's worth of patches can take
/// a while, and it must never happen while something else (a search) is blocked waiting on the
/// index mutex. Callers hold the lock only to check `known` and to extend the index afterwards.
pub fn build_new_patches(
    items: &[(PatchSource, String)],
    known: &HashSet<String>,
    embedder: &Embedder,
    mut on_embed_start: impl FnMut(&PatchSource),
    mut on_embed_done: impl FnMut(),
) -> Vec<IndexedPatch> {
    items
        .iter()
        .filter(|(source, _)| !known.contains(&source.id))
        .map(|(source, full_text)| {
            let lines =
                merge_lines(source, &[], parse_body(full_text), embedder, &mut on_embed_start, &mut on_embed_done);
            IndexedPatch {
                id: source.id.clone(),
                title: source.title.clone(),
                published: source.published.clone(),
                link: source.link.clone(),
                origin: source.origin,
                lines,
                images: Vec::new(),
            }
        })
        .collect()
}

fn normalize_title(title: &str) -> String {
    title.trim().to_ascii_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A patch's calendar day, UTC, from its ISO 8601 `published` string. Both feeds format
/// `published` the same way, so a plain prefix compare is enough.
fn published_day(published: &str) -> Option<&str> {
    published.get(..10)
}

/// Total size of a patch's *parsed* bullet lines — comparable only against another `parse_len`,
/// never against a raw, unparsed body: `parse_body` drops section headers and blank lines, so a
/// freshly re-fetched but unchanged body is always shorter as raw text than the bullets it parsed
/// into. Comparing raw-vs-parsed made every Steam News poll look "fuller" than what was already
/// indexed and re-embed it from scratch, even when nothing had changed.
fn parse_len(lines: &[PatchLine]) -> usize {
    lines.iter().map(|l| l.raw.chars().count() + 1).sum()
}

fn body_len(patch: &IndexedPatch) -> usize {
    patch.lines.iter().map(|l| l.line.raw.chars().count() + 1).sum()
}

/// Finds the already-indexed patch a Steam News item is a fuller copy of: the same post arriving
/// through `/v2/patches` (title match) or, for a patch whose Steam-post twin already scrolled out
/// of that feed's window, the shallow forum-only entry left behind (same day, see
/// `plans/patch-notes-full-text.md`). Title match is tried across every patch before falling back
/// to the weaker day match, so a real title match never loses to a same-day coincidence.
fn find_match_idx(patches: &[IndexedPatch], source: &PatchSource) -> Option<usize> {
    let title = normalize_title(&source.title);
    if let Some(pos) = patches.iter().position(|p| normalize_title(&p.title) == title) {
        return Some(pos);
    }
    let day = published_day(&source.published);
    patches.iter().position(|p| day.is_some() && published_day(&p.published) == day)
}

fn find_match<'a>(patches: &'a mut [IndexedPatch], source: &PatchSource) -> Option<&'a mut IndexedPatch> {
    patches.get_mut(find_match_idx(patches, source)?)
}

/// How many of `parsed` are not already covered by an unchanged line in `existing` — the same
/// cache-hit check `merge_lines` does, without actually embedding anything. Used to know a batch's
/// real size *before* embedding starts (see `PatchNotesState::begin_batch`'s doc comment for why
/// that matters).
fn count_missing(existing: &[IndexedLine], parsed: &[PatchLine]) -> usize {
    let mut pool: HashMap<&str, Vec<&IndexedLine>> = HashMap::new();
    for line in existing {
        pool.entry(line.line.raw.as_str()).or_default().push(line);
    }
    parsed
        .iter()
        .filter(|line| match pool.get_mut(line.raw.as_str()) {
            Some(bucket) => match bucket.iter().position(|l| l.line == **line) {
                Some(pos) => {
                    bucket.remove(pos);
                    false
                }
                None => true,
            },
            None => true,
        })
        .count()
}

/// Total lines that `build_new_patches` will actually embed for `items` — every line of every
/// not-yet-known patch, since a brand new patch has nothing cached to reuse.
pub fn count_new_patches_lines(items: &[(PatchSource, String)], known: &HashSet<String>) -> usize {
    items
        .iter()
        .filter(|(source, _)| !known.contains(&source.id))
        .map(|(_, full_text)| parse_body(full_text).len())
        .sum()
}

/// Total lines that `reconcile_steam_news` will actually embed for `items` against `patches`,
/// mirroring its own match-then-gate logic exactly (a match whose body isn't fuller embeds
/// nothing; no match embeds everything, same as a brand new patch).
pub fn count_steam_news_lines(patches: &[IndexedPatch], items: &[(PatchSource, String, Vec<String>)]) -> usize {
    items
        .iter()
        .map(|(source, full_text, _)| {
            let parsed = parse_body(full_text);
            match find_match_idx(patches, source) {
                Some(idx) if parse_len(&parsed) > body_len(&patches[idx]) => {
                    count_missing(&patches[idx].lines, &parsed)
                }
                Some(_) => 0,
                None => parsed.len(),
            }
        })
        .sum()
}

/// Reconciles Steam News items against what is already indexed: a match whose new body is fuller
/// than what is stored gets merged in place (upgrading a shallow forum-only patch, or a patch's
/// `/v2/patches`-derived twin, without disturbing its id or position, and without re-embedding any
/// line whose content hasn't actually changed); anything with no match at all becomes a new patch,
/// indexed under its own Steam-sourced id. Returns how many patches were added or upgraded. Same
/// shape as `build_new_patches`: a pure function, no lock held during embedding.
pub fn reconcile_steam_news(
    patches: &mut Vec<IndexedPatch>,
    items: &[(PatchSource, String, Vec<String>)],
    embedder: &Embedder,
    mut on_embed_start: impl FnMut(&PatchSource),
    mut on_embed_done: impl FnMut(),
) -> usize {
    let mut changed = 0;
    for (source, full_text, images) in items {
        let parsed = parse_body(full_text);
        match find_match(patches, source) {
            Some(existing) => {
                let fuller_body = parse_len(&parsed) > body_len(existing);
                if fuller_body {
                    existing.lines =
                        merge_lines(source, &existing.lines, parsed, embedder, &mut on_embed_start, &mut on_embed_done);
                    // This branch only ever runs from a Steam News fetch, so a real merge means
                    // the patch now genuinely has a Steam post — and Steam is never demoted back.
                    existing.origin = source.origin;
                }
                // Independent of whether the body grew: most patches are indexed once and never
                // change again, so gating `images` on the same "fuller body" check as line content
                // left every patch that happened to already be fully indexed *before* image
                // extraction resolved anything permanently stuck at `images: []`.
                let images_changed = !images.is_empty() && existing.images != *images;
                if images_changed {
                    existing.images = images.clone();
                }
                if fuller_body || images_changed {
                    changed += 1;
                }
            }
            None => {
                let lines = merge_lines(source, &[], parsed, embedder, &mut on_embed_start, &mut on_embed_done);
                patches.push(IndexedPatch {
                    id: source.id.clone(),
                    title: source.title.clone(),
                    published: source.published.clone(),
                    link: source.link.clone(),
                    origin: source.origin,
                    lines,
                    images: images.clone(),
                });
                changed += 1;
            }
        }
    }
    changed
}

/// Convenience for tests: builds and appends in one call. Production code goes through
/// `build_new_patches` directly so the index lock isn't held during embedding (see
/// `PatchNotesState::ingest_new`).
#[cfg(test)]
pub fn ingest(index: &mut Index, items: &[(PatchSource, String)], embedder: &Embedder) -> usize {
    let known: HashSet<String> = index.patches.iter().map(|p| p.id.clone()).collect();
    let new_patches = build_new_patches(items, &known, embedder, |_| {}, || {});
    let added = new_patches.len();
    index.patches.extend(new_patches);
    added
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::embedder;

    fn alert(id: &str) -> PatchSource {
        source(id, &format!("title {id}"), "2026-09-16T00:00:00Z", PatchOrigin::Forum)
    }

    fn source(id: &str, title: &str, published: &str, origin: PatchOrigin) -> PatchSource {
        PatchSource {
            id: id.into(),
            title: title.into(),
            published: published.into(),
            link: format!("https://example.test/{id}"),
            origin,
        }
    }

    fn temp_path(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join(FILE_NAME)
    }

    #[test]
    fn ingest_parses_and_embeds_every_new_patch() {
        let mut index = Index::default();
        let items = vec![(alert("a"), "[ Heroes ]\n- Abrams: Infernal Resilience T3 increased from +8% to +9%".into())];
        let added = ingest(&mut index, &items, embedder().unwrap());
        assert_eq!(added, 1);
        assert_eq!(index.patches[0].lines.len(), 1);
        assert_eq!(index.patches[0].lines[0].line.subject.as_deref(), Some("Abrams"));
        assert_eq!(index.patches[0].lines[0].embedding.len(), 384);
    }

    #[test]
    fn an_image_marker_line_gets_a_zero_embedding_instead_of_a_real_one() {
        // A marker line's "text" is a sentinel (see `bbcode::parse_image_marker`), not real
        // content — embedding it for real would let it show up as a nonsense semantic search
        // result. A zero vector can never cross `search::SEMANTIC_FLOOR` (cosine against it is
        // always exactly 0), so it's the correct "never matches" embedding, not a placeholder.
        let mut index = Index::default();
        let body = format!("- Before\n{}\n- After", bbcode::image_marker(0));
        let items = vec![(alert("a"), body)];
        ingest(&mut index, &items, embedder().unwrap());

        let marker_line = &index.patches[0].lines[1];
        assert!(bbcode::parse_image_marker(&marker_line.line.raw).is_some());
        assert!(marker_line.embedding.iter().all(|&v| v == 0.0));
        assert_eq!(marker_line.embedding.len(), EMBEDDING_DIM);
    }

    #[test]
    fn already_indexed_patches_are_never_reprocessed() {
        let mut index = Index::default();
        let items = vec![(alert("a"), "- Guardian bounty increased by 10%".into())];
        assert_eq!(ingest(&mut index, &items, embedder().unwrap()), 1);
        assert_eq!(ingest(&mut index, &items, embedder().unwrap()), 0);
        assert_eq!(index.patches.len(), 1);
    }

    #[test]
    fn the_index_round_trips_through_disk() {
        let path = temp_path("roundtrip");
        let mut index = Index::default();
        let items = vec![(alert("a"), "- Guardian bounty increased by 10%".into())];
        ingest(&mut index, &items, embedder().unwrap());
        save(&path, &index).unwrap();
        let back = load(&path);
        assert_eq!(back.patches.len(), 1);
        assert_eq!(back.patches[0].id, "a");
        assert_eq!(back.patches[0].lines[0].embedding.len(), 384);
    }

    #[test]
    fn loading_backfills_origin_for_a_steam_sourced_id_stuck_on_the_forum_default() {
        // Live bug: a real index file written before `origin` existed came back with every single
        // patch defaulted to `Forum`, including genuinely full Steam-sourced posts with 100+ lines
        // — `PatchDetail.origin` alone can't be trusted for data written before this field existed.
        let path = temp_path("legacy-origin");
        let mut index = Index::default();
        index.patches.push(IndexedPatch {
            id: "https://store.steampowered.com/news/app/1422450/view/1".into(),
            title: "Minor Update - 09-16-2026".into(),
            published: "2026-09-16T00:00:00Z".into(),
            link: "https://store.steampowered.com/news/app/1422450/view/1".into(),
            origin: PatchOrigin::Forum, // the pre-field-existing default, not a real forum post
            lines: vec![],
            images: vec![],
        });
        index.patches.push(IndexedPatch {
            id: "steam-news:123".into(),
            title: "Major Update".into(),
            published: "2026-09-16T00:00:00Z".into(),
            link: "https://store.steampowered.com/news/app/1422450/view/2".into(),
            origin: PatchOrigin::Forum,
            lines: vec![],
            images: vec![],
        });
        index.patches.push(IndexedPatch {
            id: "urn:forum:1".into(),
            title: "A real forum-only post".into(),
            published: "2026-09-16T00:00:00Z".into(),
            link: "https://forums.playdeadlock.com/t/1".into(),
            origin: PatchOrigin::Forum,
            lines: vec![],
            images: vec![],
        });
        save(&path, &index).unwrap();

        let back = load(&path);
        assert_eq!(back.patches[0].origin, PatchOrigin::Steam, "a store.steampowered.com id is always Steam-sourced");
        assert_eq!(back.patches[1].origin, PatchOrigin::Steam, "a steam-news: id is always Steam-sourced");
        assert_eq!(back.patches[2].origin, PatchOrigin::Forum, "a genuine forum id must not be touched");
    }

    #[test]
    fn loading_backfills_a_bold_wrapped_header_that_was_indexed_as_a_body_line() {
        // Live bug: `section_re` didn't tolerate the `**...**` `strip_bbcode` wraps a bold header
        // in, so patches indexed before the fix have "**[ Heroes ]**" stuck in the index as its
        // own line, under whatever the section was *before* it (here, the "General" default),
        // and every real line after it wrongly stuck at that same stale section too.
        let path = temp_path("legacy-section-header");
        let mut index = Index::default();
        let bad_header = PatchLine {
            section: "General".into(),
            subject: None,
            tier: None,
            description: "**[ Heroes ]**".into(),
            verb: None,
            old_value: None,
            new_value: None,
            raw: "**[ Heroes ]**".into(),
        };
        let misfiled = PatchLine {
            section: "General".into(),
            subject: Some("Abrams".into()),
            tier: None,
            description: "Infernal Resilience".into(),
            verb: Some("increased".into()),
            old_value: Some("+8%".into()),
            new_value: Some("+9%".into()),
            raw: "- Abrams: Infernal Resilience increased from +8% to +9%".into(),
        };
        index.patches.push(IndexedPatch {
            id: "a".into(),
            title: "title a".into(),
            published: "2026-09-16T00:00:00Z".into(),
            link: "https://example.test/a".into(),
            origin: PatchOrigin::Forum,
            lines: vec![
                IndexedLine { line: bad_header, embedding: vec![0.0; 384] },
                IndexedLine { line: misfiled, embedding: vec![0.0; 384] },
            ],
            images: vec![],
        });
        save(&path, &index).unwrap();

        let back = load(&path);
        assert_eq!(back.patches[0].lines.len(), 1, "the header line is dropped, not shown as body text");
        assert_eq!(
            back.patches[0].lines[0].line.section, "Heroes",
            "the real line is reassigned to the section its header named"
        );
        assert_eq!(back.patches[0].lines[0].line.raw, "- Abrams: Infernal Resilience increased from +8% to +9%");
    }

    #[test]
    fn a_missing_or_corrupt_file_loads_as_an_empty_index() {
        let dir = std::env::temp_dir().join(format!("deadlock-plus-test-{}-corrupt", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(load(&dir.join("nope.json")).patches.is_empty());
        std::fs::write(dir.join("bad.json"), b"not json").unwrap();
        assert!(load(&dir.join("bad.json")).patches.is_empty());
    }

    #[test]
    fn a_steam_news_item_upgrades_its_shallow_forum_twin_by_title() {
        let mut index = Index::default();
        let forum = source("urn:forum:1", "Minor Update - 09-16-2026", "2026-09-16T22:41:46Z", PatchOrigin::Forum);
        ingest(&mut index, &[(forum, "Guardian bounty & more...".into())], embedder().unwrap());
        assert_eq!(index.patches[0].origin, PatchOrigin::Forum);

        let steam = source("steam-news:1", "Minor Update - 09-16-2026", "2026-09-16T20:16:43Z", PatchOrigin::Steam);
        let full_body = "[ General ]\n- Guardian bounty increased by 10%\n- Parry is in-line with the client";
        let images = vec!["https://clan.akamai.steamstatic.com/images/1/a.png".to_string()];
        let changed = reconcile_steam_news(
            &mut index.patches,
            &[(steam, full_body.into(), images.clone())],
            embedder().unwrap(),
            |_| {},
            || {},
        );

        assert_eq!(changed, 1);
        assert_eq!(index.patches[0].images, images, "a real merge also picks up the fuller post's images");
        assert_eq!(index.patches.len(), 1, "the forum entry is upgraded in place, not duplicated");
        assert_eq!(index.patches[0].id, "urn:forum:1", "upgrading never changes the id");
        assert_eq!(index.patches[0].lines.len(), 2);
        assert_eq!(
            index.patches[0].origin,
            PatchOrigin::Steam,
            "a real Steam post upgrade prefers Steam as the origin"
        );
    }

    #[test]
    fn upgrading_a_patch_only_reembeds_the_lines_that_are_actually_new() {
        let mut index = Index::default();
        let forum = source("urn:forum:7", "Minor Update - 09-16-2026", "2026-09-16T22:41:46Z", PatchOrigin::Forum);
        ingest(&mut index, &[(forum, "[ General ]\n- Guardian bounty increased by 10%".into())], embedder().unwrap());
        let original_embedding = index.patches[0].lines[0].embedding.clone();

        let steam = source("steam-news:7", "Minor Update - 09-16-2026", "2026-09-16T20:16:43Z", PatchOrigin::Steam);
        let fuller_body = "[ General ]\n- Guardian bounty increased by 10%\n- Parry is in-line with the client";
        let mut embed_calls = 0;
        let changed = reconcile_steam_news(
            &mut index.patches,
            &[(steam, fuller_body.into(), vec![])],
            embedder().unwrap(),
            |_| embed_calls += 1,
            || {},
        );

        assert_eq!(changed, 1);
        assert_eq!(embed_calls, 1, "only the genuinely new line should be embedded");
        assert_eq!(index.patches[0].lines.len(), 2);
        assert_eq!(
            index.patches[0].lines[0].embedding, original_embedding,
            "the unchanged line keeps its old embedding"
        );
    }

    /// The progress bar needs the real batch size *before* embedding starts (see
    /// `PatchNotesState::begin_batch`): `count_steam_news_lines` must predict the exact number of
    /// `on_embed_start` calls `reconcile_steam_news` is about to make, not just an upper bound.
    #[test]
    fn count_steam_news_lines_predicts_the_exact_number_of_embed_calls() {
        let mut index = Index::default();
        let forum = source("urn:forum:8", "Minor Update - 09-16-2026", "2026-09-16T22:41:46Z", PatchOrigin::Forum);
        ingest(&mut index, &[(forum, "[ General ]\n- Guardian bounty increased by 10%".into())], embedder().unwrap());

        let steam = source("steam-news:8", "Minor Update - 09-16-2026", "2026-09-16T20:16:43Z", PatchOrigin::Steam);
        let new_item = source("steam-news:9", "Major Update - 03-01-2026", "2026-03-01T00:00:00Z", PatchOrigin::Steam);
        let items = vec![
            (
                steam,
                "[ General ]\n- Guardian bounty increased by 10%\n- Parry is in-line with the client".to_string(),
                vec![],
            ),
            (new_item, "- Two brand new lines\n- Both should count".to_string(), vec![]),
        ];

        let predicted = count_steam_news_lines(&index.patches, &items);
        let mut embed_calls = 0;
        reconcile_steam_news(&mut index.patches, &items, embedder().unwrap(), |_| embed_calls += 1, || {});

        assert_eq!(predicted, embed_calls, "the upfront count must match the real embedding work exactly");
        assert_eq!(predicted, 3, "1 genuinely new line on the upgraded patch + 2 lines on the brand new one");
    }

    #[test]
    fn count_new_patches_lines_predicts_the_exact_number_of_embed_calls() {
        let known: HashSet<String> = HashSet::new();
        let items = vec![
            (alert("a"), "- Guardian bounty increased by 10%\n- Parry is in-line with the client".to_string()),
            (alert("b"), "- One more line".to_string()),
        ];

        let predicted = count_new_patches_lines(&items, &known);
        let mut embed_calls = 0;
        build_new_patches(&items, &known, embedder().unwrap(), |_| embed_calls += 1, || {});

        assert_eq!(predicted, embed_calls);
        assert_eq!(predicted, 3);
    }

    /// Regression test: a body with section headers (`[ General ]`) is always shorter once parsed
    /// than its raw source, since `parse_body` drops the header lines. Comparing the *raw* length
    /// of a re-fetched body against the *parsed* length already stored made every repeat poll of
    /// an unchanged Steam News item look fuller than what was indexed, and re-embed it forever.
    #[test]
    fn an_unchanged_steam_news_body_with_section_headers_is_not_reembedded_on_a_repeat_poll() {
        let mut index = Index::default();
        let steam = source("steam-news:6", "Minor Update - 09-16-2026", "2026-09-16T20:16:43Z", PatchOrigin::Steam);
        let full_body = "[ General ]\n- Guardian bounty increased by 10%\n- Parry is in-line with the client";
        let changed = reconcile_steam_news(
            &mut index.patches,
            &[(steam, full_body.into(), vec![])],
            embedder().unwrap(),
            |_| {},
            || {},
        );
        assert_eq!(changed, 1, "first sighting always indexes");

        let steam_again =
            source("steam-news:6", "Minor Update - 09-16-2026", "2026-09-16T20:16:43Z", PatchOrigin::Steam);
        let changed_again = reconcile_steam_news(
            &mut index.patches,
            &[(steam_again, full_body.into(), vec![])],
            embedder().unwrap(),
            |_| {},
            || {},
        );
        assert_eq!(changed_again, 0, "the exact same body polled again must not count as fuller");
    }

    #[test]
    fn a_steam_news_item_upgrades_a_same_day_forum_only_patch_when_titles_differ() {
        let mut index = Index::default();
        let forum = source("urn:forum:2", "09-16-2026 Update", "2026-09-16T22:41:46Z", PatchOrigin::Forum);
        ingest(&mut index, &[(forum, "...".into())], embedder().unwrap());

        let steam = source("steam-news:2", "Minor Update - 09-16-2026", "2026-09-16T20:16:43Z", PatchOrigin::Steam);
        let full_body = "- Guardian bounty increased by 10%";
        let changed = reconcile_steam_news(
            &mut index.patches,
            &[(steam, full_body.into(), vec![])],
            embedder().unwrap(),
            |_| {},
            || {},
        );

        assert_eq!(changed, 1);
        assert_eq!(index.patches[0].id, "urn:forum:2");
        assert_eq!(index.patches[0].lines.len(), 1);
    }

    #[test]
    fn a_steam_news_item_with_no_match_is_added_as_a_new_patch() {
        let mut index = Index::default();
        let steam = source("steam-news:3", "Major Update - 03-01-2026", "2026-03-01T00:00:00Z", PatchOrigin::Steam);
        let changed = reconcile_steam_news(
            &mut index.patches,
            &[(steam, "- Guardian bounty increased by 10%".into(), vec![])],
            embedder().unwrap(),
            |_| {},
            || {},
        );
        assert_eq!(changed, 1);
        assert_eq!(index.patches[0].id, "steam-news:3");
    }

    #[test]
    fn a_shorter_steam_news_body_never_overwrites_a_fuller_one_but_still_picks_up_images() {
        let mut index = Index::default();
        let forum = source("urn:forum:4", "Minor Update - 09-16-2026", "2026-09-16T22:41:46Z", PatchOrigin::Forum);
        let full_body = "- Guardian bounty increased by 10%\n- Parry is in-line with the client";
        ingest(&mut index, &[(forum, full_body.into())], embedder().unwrap());

        let steam = source("steam-news:4", "Minor Update - 09-16-2026", "2026-09-16T20:16:43Z", PatchOrigin::Steam);
        let images = vec!["https://clan.akamai.steamstatic.com/images/1/a.png".to_string()];
        let changed = reconcile_steam_news(
            &mut index.patches,
            &[(steam, "Guardian bounty & more...".into(), images.clone())],
            embedder().unwrap(),
            |_| {},
            || {},
        );

        // Item 8: most patches are indexed once and never change again, so gating `images` on the
        // same "fuller body" check as line content left them permanently stuck at `images: []`.
        // Images are picked up independently of whether the text was fuller.
        assert_eq!(changed, 1);
        assert_eq!(index.patches[0].lines.len(), 2, "a rejected (non-fuller) body must not touch line content");
        assert_eq!(index.patches[0].origin, PatchOrigin::Forum, "a rejected body update must not touch origin");
        assert_eq!(index.patches[0].images, images);
    }

    #[test]
    fn images_are_not_touched_when_the_incoming_item_resolves_none_or_the_same_ones() {
        let mut index = Index::default();
        let images = vec!["https://clan.akamai.steamstatic.com/images/1/a.png".to_string()];
        let steam = source("steam-news:10", "Minor Update - 09-16-2026", "2026-09-16T20:16:43Z", PatchOrigin::Steam);
        reconcile_steam_news(
            &mut index.patches,
            &[(steam, "- a".into(), images.clone())],
            embedder().unwrap(),
            |_| {},
            || {},
        );

        let steam_again =
            source("steam-news:10", "Minor Update - 09-16-2026", "2026-09-16T20:16:43Z", PatchOrigin::Steam);
        let changed = reconcile_steam_news(
            &mut index.patches,
            &[(steam_again, "- a".into(), vec![])],
            embedder().unwrap(),
            |_| {},
            || {},
        );

        assert_eq!(changed, 0, "no new images and an unchanged body is genuinely nothing changing");
        assert_eq!(
            index.patches[0].images, images,
            "an item resolving no images must not clear what is already stored"
        );
    }

    #[test]
    fn an_unchanged_body_that_newly_resolves_an_image_it_did_not_have_before_still_picks_it_up() {
        let mut index = Index::default();
        let steam = source("steam-news:11", "Minor Update - 09-16-2026", "2026-09-16T20:16:43Z", PatchOrigin::Steam);
        reconcile_steam_news(&mut index.patches, &[(steam, "- a".into(), vec![])], embedder().unwrap(), |_| {}, || {});
        assert!(index.patches[0].images.is_empty());

        let images = vec!["https://clan.akamai.steamstatic.com/images/1/a.png".to_string()];
        let steam_again =
            source("steam-news:11", "Minor Update - 09-16-2026", "2026-09-16T20:16:43Z", PatchOrigin::Steam);
        let mut embed_calls = 0;
        let changed = reconcile_steam_news(
            &mut index.patches,
            &[(steam_again, "- a".into(), images.clone())],
            embedder().unwrap(),
            |_| embed_calls += 1,
            || {},
        );

        assert_eq!(changed, 1);
        assert_eq!(embed_calls, 0, "the unchanged text must not be re-embedded just because an image showed up");
        assert_eq!(index.patches[0].images, images);
    }

    #[test]
    fn patch_detail_carries_every_line_with_no_embeddings() {
        let mut index = Index::default();
        let body = "[ Heroes ]\n- Abrams: Infernal Resilience T3 increased from +8% to +9%";
        ingest(&mut index, &[(alert("a"), body.into())], embedder().unwrap());

        let detail = PatchDetail::from(&index.patches[0]);

        assert_eq!(detail.id, "a");
        assert_eq!(detail.lines.len(), 1);
        assert_eq!(detail.lines[0].subject.as_deref(), Some("Abrams"));
        assert_eq!(detail.lines[0].section, "Heroes");
    }

    #[test]
    fn reconciling_is_idempotent_across_repeated_polls() {
        let mut index = Index::default();
        let steam = source("steam-news:5", "Minor Update - 09-16-2026", "2026-09-16T20:16:43Z", PatchOrigin::Steam);
        let full_body = "- Guardian bounty increased by 10%";
        assert_eq!(
            reconcile_steam_news(
                &mut index.patches,
                &[(steam, full_body.into(), vec![])],
                embedder().unwrap(),
                |_| {},
                || {}
            ),
            1
        );

        let steam_again =
            source("steam-news:5", "Minor Update - 09-16-2026", "2026-09-16T20:16:43Z", PatchOrigin::Steam);
        assert_eq!(
            reconcile_steam_news(
                &mut index.patches,
                &[(steam_again, full_body.into(), vec![])],
                embedder().unwrap(),
                |_| {},
                || {}
            ),
            0
        );
        assert_eq!(index.patches.len(), 1);
    }
}
