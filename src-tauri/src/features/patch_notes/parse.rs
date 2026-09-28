use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One bullet line from a patch body, split into whatever structure the text actually offers.
/// Real patch text is prose, not machine-generated data: `subject`/`tier`/`old_value`/`new_value`
/// are best-effort hints for scoring and filtering, never the source of truth. `raw` is always
/// the original line and is what gets shown to the user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PatchLine {
    pub section: String,
    pub subject: Option<String>,
    pub tier: Option<u8>,
    pub description: String,
    pub verb: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub raw: String,
}

/// `**...**` on either side is optional: `bbcode::strip_bbcode` turns a bold-wrapped
/// `[b][ General ][/b]` into `**[ General ]**`, and that still reads as a header, not body text.
fn section_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(?:\*\*)?\[\s*(?P<name>.+?)\s*\](?:\*\*)?$").unwrap())
}

/// `Subject: description verb [from old to new | by amount]`. The subject charclass excludes
/// `:` so there is only one place the split can happen; no backtracking ambiguity.
fn change_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?x)
            ^
            (?:(?P<subject>[A-Z][^:]{0,80}):\s*)?
            (?P<desc>.+?)
            \s+(?P<verb>increased|reduced|decreased)
            (?:
                \s+from\s+(?P<old>.+?)\s+to\s+(?P<new>.+)
              | \s+by\s+(?P<amount>.+)
            )?
            $
            ",
        )
        .unwrap()
    })
}

fn subject_only_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(?P<subject>[A-Z][^:]{0,80}):\s*(?P<desc>.+)$").unwrap())
}

fn tier_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\bT([1-3])\b").unwrap())
}

/// Pulls a `T1`/`T2`/`T3` tier marker out of a description, collapsing the whitespace it leaves
/// behind so "Infernal Resilience T3" becomes ("Infernal Resilience", Some(3)).
fn extract_tier(desc: &str) -> (String, Option<u8>) {
    match tier_re().captures(desc) {
        Some(caps) => {
            let tier = caps.get(1).unwrap().as_str().parse().ok();
            let whole = caps.get(0).unwrap();
            let cleaned = format!("{}{}", &desc[..whole.start()], &desc[whole.end()..]);
            (cleaned.split_whitespace().collect::<Vec<_>>().join(" "), tier)
        }
        None => (desc.to_owned(), None),
    }
}

fn parse_line(section: &str, raw: &str) -> PatchLine {
    let bullet = raw.strip_prefix("- ").unwrap_or(raw);

    let (subject, desc, verb, old_value, new_value) = if let Some(caps) = change_re().captures(bullet) {
        let get = |name: &str| caps.name(name).map(|m| m.as_str().trim().to_owned());
        (get("subject"), get("desc").unwrap_or_default(), get("verb"), get("old"), get("new").or_else(|| get("amount")))
    } else if let Some(caps) = subject_only_re().captures(bullet) {
        (
            Some(caps["subject"].trim().to_owned()),
            caps["desc"].trim().to_owned(),
            None,
            None,
            None,
        )
    } else {
        (None, bullet.to_owned(), None, None, None)
    };

    let (description, tier) = extract_tier(&desc);

    PatchLine { section: section.to_owned(), subject, tier, description, verb, old_value, new_value, raw: raw.to_owned() }
}

/// Whether an already-trimmed line is a `[ Section ]` header (bold-wrapped or not) rather than
/// real content, returning its name when it is. Shared with `store::backfill_section_headers`,
/// which re-checks already-indexed lines against this same rule after it changes.
pub(crate) fn section_header(raw: &str) -> Option<String> {
    section_re().captures(raw).map(|caps| caps["name"].to_owned())
}

/// Splits a full, stripped patch body into lines, tracking the `[ Section ]` header each one
/// falls under. Blank lines and the headers themselves are dropped; everything else becomes a
/// `PatchLine`, structured as well as the text allows.
pub fn parse_body(full_text: &str) -> Vec<PatchLine> {
    let mut section = "General".to_owned();
    let mut lines = Vec::new();
    for raw in full_text.lines() {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        if let Some(name) = section_header(raw) {
            section = name;
            continue;
        }
        lines.push(parse_line(&section, raw));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(raw: &str) -> PatchLine {
        parse_line("Heroes", raw)
    }

    #[test]
    fn a_hero_ability_change_with_a_tier_is_fully_structured() {
        let l = line("- Abrams: Infernal Resilience T3 increased from +8% to +9%");
        assert_eq!(l.subject.as_deref(), Some("Abrams"));
        assert_eq!(l.description, "Infernal Resilience");
        assert_eq!(l.tier, Some(3));
        assert_eq!(l.verb.as_deref(), Some("increased"));
        assert_eq!(l.old_value.as_deref(), Some("+8%"));
        assert_eq!(l.new_value.as_deref(), Some("+9%"));
    }

    #[test]
    fn an_item_change_without_a_tier_has_no_tier() {
        let l = line("- Weakening Headshot: Bullet Resist Reduction reduced from -13% to -12%");
        assert_eq!(l.subject.as_deref(), Some("Weakening Headshot"));
        assert_eq!(l.description, "Bullet Resist Reduction");
        assert_eq!(l.tier, None);
        assert_eq!(l.old_value.as_deref(), Some("-13%"));
        assert_eq!(l.new_value.as_deref(), Some("-12%"));
    }

    #[test]
    fn a_value_containing_its_own_arrow_still_splits_on_the_final_to() {
        let l = line("- Lash: Gun falloff range reduced from 18m->54m to 16m->48m");
        assert_eq!(l.old_value.as_deref(), Some("18m->54m"));
        assert_eq!(l.new_value.as_deref(), Some("16m->48m"));
    }

    #[test]
    fn a_delta_style_change_has_no_old_value() {
        let l = line("- Guardian bounty increased by 10%");
        assert_eq!(l.subject, None);
        assert_eq!(l.description, "Guardian bounty");
        assert_eq!(l.old_value, None);
        assert_eq!(l.new_value.as_deref(), Some("10%"));
    }

    #[test]
    fn a_line_with_a_subject_but_no_recognised_verb_keeps_the_subject() {
        let l = line("- Kelvin: Frozen Shelter base ability health regen now scales with spirit power (0.2)");
        assert_eq!(l.subject.as_deref(), Some("Kelvin"));
        assert_eq!(l.description, "Frozen Shelter base ability health regen now scales with spirit power (0.2)");
        assert_eq!(l.verb, None);
    }

    #[test]
    fn a_bug_fix_line_still_keeps_its_subject() {
        let l = line("- Paige: Fixed some collision issues with Rallying Charge");
        assert_eq!(l.subject.as_deref(), Some("Paige"));
        assert_eq!(l.verb, None);
        assert_eq!(l.old_value, None);
    }

    #[test]
    fn a_line_with_no_colon_and_no_verb_is_kept_as_plain_text() {
        let l = line("- Spiritual Overflow: Buildup is 35% slower");
        assert_eq!(l.subject.as_deref(), Some("Spiritual Overflow"));
        assert_eq!(l.description, "Buildup is 35% slower");
        assert_eq!(l.verb, None);
    }

    #[test]
    fn raw_always_keeps_the_original_bullet_including_the_dash() {
        let l = line("- Abrams: Infernal Resilience T3 increased from +8% to +9%");
        assert_eq!(l.raw, "- Abrams: Infernal Resilience T3 increased from +8% to +9%");
    }

    #[test]
    fn parse_body_tracks_sections_and_skips_headers_and_blank_lines() {
        let body = "[ General ]\n\n- Guardian bounty increased by 10%\n\n[ Heroes ]\n- Abrams: Infernal Resilience T3 increased from +8% to +9%\n";
        let lines = parse_body(body);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].section, "General");
        assert_eq!(lines[1].section, "Heroes");
        assert_eq!(lines[1].subject.as_deref(), Some("Abrams"));
    }

    #[test]
    fn a_line_before_any_section_header_defaults_to_general() {
        let lines = parse_body("- Guardian bounty increased by 10%");
        assert_eq!(lines[0].section, "General");
    }

    #[test]
    fn an_image_marker_line_survives_as_an_ordinary_line_at_its_own_position() {
        // `bbcode::strip_bbcode` leaves an image marker as its own line, on purpose, so its
        // position in the body survives this split — it must come through untouched, not get
        // mistaken for a section header or mangled by the change/subject parsing rules.
        let body = "[ General ]\n- Before\n\u{fffc}0\u{fffc}\n- After";
        let lines = parse_body(body);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[1].raw, "\u{fffc}0\u{fffc}");
        assert_eq!(lines[1].section, "General");
        assert_eq!(lines[1].subject, None);
    }

    #[test]
    fn a_bold_wrapped_bracket_header_is_read_as_a_real_section_not_a_text_line() {
        // Real post shape after `bbcode::strip_bbcode`: `[b][ General ][/b]` becomes
        // `**[ General ]**` (see that module's own test). Confirmed live: the app rendered the
        // literal text "**[ General ]**" as a bolded body line instead of a section heading,
        // because `section_re` never accounted for the surrounding `**`.
        let body = "**[ General ]**\n- Guardian bounty increased by 10%";
        let lines = parse_body(body);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].section, "General");
        assert_eq!(lines[0].raw, "- Guardian bounty increased by 10%");
    }
}
