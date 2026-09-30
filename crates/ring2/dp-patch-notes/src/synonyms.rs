//! Hand-maintained bridge between how players talk about changes and how patch notes word them.
//! Consulted when a keyword match against the structured fields is close but not exact. Longer
//! phrases are listed first so they are tried before a shorter phrase they contain.
const SYNONYMS: &[(&str, &str)] = &[
    ("full healing removal", "-100% healing reduction"),
    ("healing removal", "healing reduction"),
    ("healing reduction", "healing removal"),
    ("cooldown", "cd"),
    ("cd", "cooldown"),
    ("dmg", "damage"),
    ("damage", "dmg"),
    ("hp", "health"),
    ("health", "hp"),
    ("nerfed", "reduced"),
    ("nerf", "reduced"),
    ("buffed", "increased"),
    ("buff", "increased"),
    ("removed", "reduced"),
    ("full", "-100%"),
];

/// The query, plus one variant per synonym phrase it contains with that phrase swapped for its
/// pair. Case-insensitive; the replacement keeps the query's original casing everywhere else.
pub fn expand(query: &str) -> Vec<String> {
    let lower = query.to_ascii_lowercase();
    let mut variants = vec![query.to_owned()];
    for (from, to) in SYNONYMS {
        if let Some(pos) = lower.find(from) {
            let mut variant = query.to_owned();
            variant.replace_range(pos..pos + from.len(), to);
            if !variants.contains(&variant) {
                variants.push(variant);
            }
        }
    }
    variants
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_query_itself_is_always_included() {
        assert_eq!(expand("no synonyms here"), vec!["no synonyms here"]);
    }

    #[test]
    fn a_known_phrase_is_bridged_to_patch_note_wording() {
        let variants = expand("Pocket's Affliction full healing removal");
        assert!(variants.iter().any(|v| v.contains("-100% healing reduction")), "{variants:?}");
    }

    #[test]
    fn shorter_overlapping_phrases_still_produce_their_own_variant() {
        let variants = expand("healing removal on cooldown");
        assert!(variants.iter().any(|v| v.contains("healing reduction")), "{variants:?}");
        assert!(variants.iter().any(|v| v.contains("cd")), "{variants:?}");
    }

    #[test]
    fn matching_is_case_insensitive_but_preserves_the_rest_of_the_casing() {
        let variants = expand("Full Healing Removal");
        assert!(variants.iter().any(|v| v == "-100% Healing Removal"), "{variants:?}");
    }
}
