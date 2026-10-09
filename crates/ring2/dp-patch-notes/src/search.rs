use std::collections::HashSet;

use serde::Serialize;
use ts_rs::TS;

use crate::bbcode;
use crate::embed::{cosine, Embed};
use crate::parse::PatchLine;
use crate::store::{Index, IndexedPatch, PatchOrigin};
use crate::synonyms;

/// A keyword match with at least this many overlapping tokens (or a whole-subject hit, worth 2)
/// is trusted on its own; below it, the semantic layer also runs to fill in what keywords missed.
const CONFIDENT_KEYWORD_SCORE: f32 = 2.0;
const MIN_KEYWORD_SCORE: f32 = 1.0;
/// Below this cosine similarity a "semantic" hit is noise, not a fuzzy match (see `embed.rs`
/// tests: an unrelated line scores well under this against a genuinely related one).
const SEMANTIC_FLOOR: f32 = 0.35;

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PatchSearchResult {
    pub patch_id: String,
    pub title: String,
    pub published: String,
    pub link: String,
    pub origin: PatchOrigin,
    pub section: String,
    pub snippet: String,
    pub score: f32,
}

fn tokenize(text: &str) -> HashSet<String> {
    text.split_whitespace()
        .map(|t| t.trim_matches(|c: char| c.is_ascii_punctuation() && c != '%' && c != '+' && c != '-'))
        .filter(|t| !t.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

/// What keyword search needs from a line, derived from the line alone so it can be computed once
/// at ingest (or on first use after a load) instead of on every query. Never written to disk.
#[derive(Debug, Clone)]
pub(crate) struct LineDerived {
    pub is_marker: bool,
    tokens: HashSet<String>,
    subject_tokens: HashSet<String>,
}

impl LineDerived {
    pub(crate) fn of(line: &PatchLine) -> Self {
        let mut haystack = String::new();
        if let Some(subject) = &line.subject {
            haystack.push_str(subject);
            haystack.push(' ');
        }
        haystack.push_str(&line.description);
        for value in line.old_value.iter().chain(&line.new_value) {
            haystack.push(' ');
            haystack.push_str(value);
        }
        Self {
            is_marker: bbcode::parse_image_marker(&line.raw).is_some(),
            tokens: tokenize(&haystack),
            subject_tokens: line.subject.as_deref().map(tokenize).unwrap_or_default(),
        }
    }
}

/// Token overlap between the query and the line's structured fields, plus a bonus when every
/// token of a multi-word subject (e.g. "Weakening Headshot") is present in the query.
fn keyword_score(query_tokens: &HashSet<String>, derived: &LineDerived) -> f32 {
    let overlap = derived.tokens.intersection(query_tokens).count() as f32;
    let subject_bonus = !derived.subject_tokens.is_empty() && derived.subject_tokens.is_subset(query_tokens);
    overlap + if subject_bonus { 2.0 } else { 0.0 }
}

fn to_result(patch: &IndexedPatch, line: &PatchLine, score: f32) -> PatchSearchResult {
    PatchSearchResult {
        patch_id: patch.id.clone(),
        title: patch.title.clone(),
        published: patch.published.clone(),
        link: patch.link.clone(),
        origin: patch.origin,
        section: line.section.clone(),
        snippet: line.raw.clone(),
        score,
    }
}

/// Structured/keyword matching first (cheap, exact on names and values), then a semantic pass
/// over embeddings when keywords alone did not turn up a confident answer.
///
/// `embedder` is only called when the semantic pass actually runs, so a query the keywords answer
/// never loads the model.
pub fn search<E: Embed>(
    index: &Index,
    query: &str,
    embedder: impl FnOnce() -> Result<E, &'static str>,
    limit: usize,
) -> Vec<PatchSearchResult> {
    let variants = synonyms::expand(query);
    let variant_tokens: Vec<HashSet<String>> = variants.iter().map(|v| tokenize(v)).collect();

    let mut keyword_hits: Vec<(f32, &IndexedPatch, &PatchLine)> = Vec::new();
    for patch in &index.patches {
        for line in &patch.lines {
            // An image marker is a positional sentinel, not real content (see
            // `bbcode::parse_image_marker`) — it must never surface as a search result.
            let derived = line.derived();
            if derived.is_marker {
                continue;
            }
            let best = variant_tokens.iter().map(|t| keyword_score(t, derived)).fold(0.0f32, f32::max);
            if best >= MIN_KEYWORD_SCORE {
                keyword_hits.push((best, patch, &line.line));
            }
        }
    }
    keyword_hits.sort_by(|a, b| b.0.total_cmp(&a.0));

    let confident = keyword_hits.first().is_some_and(|(score, ..)| *score >= CONFIDENT_KEYWORD_SCORE);
    let mut results: Vec<PatchSearchResult> =
        keyword_hits.iter().take(limit).map(|(score, patch, line)| to_result(patch, line, *score)).collect();

    if !confident || results.len() < limit {
        if let Ok(embedder) = embedder() {
            if let Ok(query_embedding) = embedder.embed(query) {
                let seen: HashSet<(&str, &str)> =
                    keyword_hits.iter().map(|(_, p, l)| (p.id.as_str(), l.raw.as_str())).collect();
                let mut semantic_hits: Vec<(f32, &IndexedPatch, &PatchLine)> = index
                    .patches
                    .iter()
                    .flat_map(|p| p.lines.iter().map(move |l| (p, l)))
                    .filter(|(p, l)| !seen.contains(&(p.id.as_str(), l.line.raw.as_str())))
                    .filter(|(_, l)| !l.derived().is_marker)
                    .map(|(p, l)| (cosine(&query_embedding, &l.embedding), p, &l.line))
                    .filter(|(score, ..)| *score >= SEMANTIC_FLOOR)
                    .collect();
                semantic_hits.sort_by(|a, b| b.0.total_cmp(&a.0));
                results.extend(
                    semantic_hits.into_iter().take(limit).map(|(score, patch, line)| to_result(patch, line, score)),
                );
            }
        }
    }

    results.sort_by(|a, b| b.score.total_cmp(&a.score));
    results.truncate(limit);
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{ingest, PatchSource};

    fn alert(id: &str, title: &str) -> PatchSource {
        PatchSource {
            id: id.into(),
            title: title.into(),
            published: "2026-09-16T00:00:00Z".into(),
            link: format!("https://example.test/{id}"),
            origin: PatchOrigin::Forum,
        }
    }

    fn fixture_index() -> Index {
        let mut index = Index::default();
        let embedder = crate::embed::embedder().unwrap();
        let body = "[ Heroes ]\n\
            - Abrams: Infernal Resilience T3 increased from +8% to +9%\n\
            - Pocket: Affliction T3 now gives -100% healing reduction\n\
            [ Items ]\n\
            - Weakening Headshot: Bullet Resist Reduction reduced from -13% to -12%\n\
            [ General ]\n\
            - Guardian bounty increased by 10%\n";
        ingest(&mut index, &[(alert("p1", "Minor Update - 09-16-2026"), body.to_string())], embedder);
        index
    }

    #[test]
    fn a_hero_and_value_query_finds_the_exact_structured_line() {
        let index = fixture_index();

        let results = search(&index, "Abrams Infernal Resilience T3", crate::embed::embedder, 5);
        assert!(!results.is_empty());
        assert!(results[0].snippet.contains("Abrams"), "{results:?}");
    }

    #[test]
    fn a_loosely_phrased_query_still_finds_the_line_via_the_semantic_fallback() {
        let index = fixture_index();

        let results =
            search(&index, "When was Pocket's Affliction given full healing removal?", crate::embed::embedder, 5);
        assert!(results.iter().any(|r| r.snippet.contains("Pocket")), "{results:?}");
    }

    #[test]
    fn an_unrelated_query_returns_nothing_above_the_noise_floor() {
        let index = fixture_index();

        let results = search(&index, "what is the weather today", crate::embed::embedder, 5);
        assert!(results.is_empty(), "{results:?}");
    }

    #[test]
    fn results_link_back_to_their_source_patch() {
        let index = fixture_index();

        let results = search(&index, "Weakening Headshot", crate::embed::embedder, 5);
        assert_eq!(results[0].link, "https://example.test/p1");
        assert_eq!(results[0].title, "Minor Update - 09-16-2026");
    }

    #[test]
    fn an_image_marker_line_never_surfaces_as_a_search_result() {
        let mut index = Index::default();
        let embedder = crate::embed::embedder().unwrap();
        let marker = bbcode::image_marker(0);
        let body = format!("[ General ]\n- Before\n{marker}\n- After");
        ingest(&mut index, &[(alert("p1", "Minor Update"), body)], embedder);
        // The exact marker text as the query is the strongest possible keyword match against
        // itself (an exact token, not a fuzzy one) — if the marker guard in `search` were ever
        // removed, this is what would catch it turning up as a result.
        let results = search(&index, &marker, crate::embed::embedder, 5);
        assert!(results.is_empty(), "{results:?}");
    }

    struct Unreachable;

    impl Embed for Unreachable {
        fn embed(&self, _: &str) -> Result<Vec<f32>, String> {
            panic!("the embedder must not be used");
        }
    }

    fn modelless_index() -> Index {
        let lines = crate::parse::parse_body("- Abrams: Infernal Resilience T3 increased from +8% to +9%")
            .into_iter()
            .map(|line| crate::store::IndexedLine::new(line, vec![0.0; 384]))
            .collect();
        let patch = IndexedPatch {
            id: "p1".into(),
            title: "t".into(),
            published: "2026-09-16T00:00:00Z".into(),
            link: "l".into(),
            origin: PatchOrigin::Forum,
            lines,
            images: Vec::new(),
        };
        Index { patches: vec![patch] }
    }

    #[test]
    fn a_confident_keyword_answer_never_asks_for_the_model() {
        let index = modelless_index();
        let results = search(
            &index,
            "Abrams Infernal Resilience T3",
            || -> Result<Unreachable, _> { panic!("model requested") },
            1,
        );
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn a_line_read_back_from_disk_matches_the_same_keywords_as_a_fresh_one() {
        let index = modelless_index();
        let json = serde_json::to_string(&index).unwrap();
        assert!(!json.contains("tokens") && !json.contains("is_marker"), "derived data must not be stored");
        let back: Index = serde_json::from_str(&json).unwrap();
        let fresh = search(&index, "Abrams Infernal Resilience T3", || Ok(Unreachable), 1);
        let loaded = search(&back, "Abrams Infernal Resilience T3", || Ok(Unreachable), 1);
        assert_eq!(fresh.len(), 1);
        assert_eq!(fresh[0].snippet, loaded[0].snippet);
        assert_eq!(fresh[0].score, loaded[0].score);
    }
}
