use std::collections::HashMap;

use serde::Deserialize;

use super::Alert;

const SUMMARY_CHARS: usize = 360;
const SUMMARY_LINES: usize = 7;
const STEAM_NEWS_PREFIX: &str = "https://store.steampowered.com/news/app/";

#[derive(Deserialize)]
struct RawItem {
    source: Option<String>,
    title: Option<String>,
    pub_date: Option<String>,
    link: Option<String>,
    guid: Option<RawGuid>,
    content: Option<String>,
}

#[derive(Deserialize)]
struct RawGuid {
    text: Option<String>,
}

struct Parsed {
    alert: Alert,
    /// The Steam post a forum changelog entry embeds, when it has one.
    steam_ref: Option<String>,
}

/// Items without a title or a stable id are dropped: they cannot be deduplicated or shown.
/// A forum changelog entry that embeds a Steam post also present in the feed is folded into that
/// post, which carries the real update type and the full text; the forum entry only lends its image.
pub fn parse_feed(json: &str) -> Result<Vec<Alert>, serde_json::Error> {
    let raw: Vec<RawItem> = serde_json::from_str(json)?;
    let parsed: Vec<Parsed> = raw.into_iter().filter_map(parse_item).collect();

    let steam_links: HashMap<String, usize> = parsed
        .iter()
        .enumerate()
        .filter(|(_, p)| p.alert.source == "steam")
        .map(|(i, p)| (p.alert.link.clone(), i))
        .collect();

    let mut images: Vec<Option<String>> = parsed.iter().map(|p| p.alert.image.clone()).collect();
    let mut folded = vec![false; parsed.len()];
    for (i, p) in parsed.iter().enumerate() {
        if p.alert.source != "forum" {
            continue;
        }
        if let Some(&target) = p.steam_ref.as_ref().and_then(|r| steam_links.get(r)) {
            folded[i] = true;
            if images[target].is_none() {
                images[target] = p.alert.image.clone();
            }
        }
    }

    Ok(parsed
        .into_iter()
        .zip(images)
        .zip(folded)
        .filter(|(_, folded)| !folded)
        .map(|((p, image), _)| Alert { image, ..p.alert })
        .collect())
}

fn parse_item(r: RawItem) -> Option<Parsed> {
    let title = r.title.map(|t| t.trim().to_owned()).filter(|t| !t.is_empty())?;
    let link = r.link.unwrap_or_default();
    let id = r.guid.and_then(|g| g.text).filter(|g| !g.is_empty()).unwrap_or_else(|| link.clone());
    if id.is_empty() {
        return None;
    }
    let content = r.content.unwrap_or_default();
    Some(Parsed {
        steam_ref: steam_ref(&content),
        alert: Alert {
            id,
            kind: kind_of(&title),
            title,
            link,
            source: r.source.unwrap_or_default(),
            published: r.pub_date.unwrap_or_default(),
            image: first_steam_image(&content),
            summary: summary_of(&content),
            read: false,
        },
    })
}

/// The update type as named by the title: "Minor Update - 09-16-2026" is a "Minor Update". Titles that
/// are only a date ("09-16-2026 Update") are the generic forum changelog.
fn kind_of(title: &str) -> String {
    let base = match title.rsplit_once(" - ") {
        Some((head, tail)) if is_date(tail.trim()) => head.trim(),
        _ => title.trim(),
    };
    let first = base.split_whitespace().next().unwrap_or("");
    if base.is_empty() || is_date(first) {
        "Patch notes".to_owned()
    } else {
        base.to_owned()
    }
}

/// MM-DD-YYYY, the form Valve uses in titles.
fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10 && b.iter().enumerate().all(|(i, c)| if i == 2 || i == 5 { *c == b'-' } else { c.is_ascii_digit() })
}

fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = text.find(start)? + start.len();
    let len = text[from..].find(end)?;
    Some(&text[from..from + len])
}

fn steam_ref(content: &str) -> Option<String> {
    let url = between(content, "data-url=\"", "\"")?;
    url.starts_with(STEAM_NEWS_PREFIX).then(|| url.to_owned())
}

/// Only Steam-hosted art is used: the other images in a post are icons and favicons.
fn first_steam_image(content: &str) -> Option<String> {
    let mut rest = content;
    while let Some(at) = rest.find("<img") {
        let tag_end = rest[at..].find('>').map_or(rest.len(), |e| at + e);
        let tag = &rest[at..tag_end];
        if let Some(src) = between(tag, "src=\"", "\"") {
            let host_ok = src
                .strip_prefix("https://")
                .and_then(|s| s.split('/').next())
                .is_some_and(|h| h.ends_with(".steamstatic.com"));
            if host_ok && !src.contains("favicon") {
                return Some(src.to_owned());
            }
        }
        rest = &rest[tag_end.min(rest.len())..];
    }
    None
}

fn summary_of(content: &str) -> String {
    match between(content, "js-unfurl-desc\">", "</div>") {
        // The forum's link preview arrives already flattened to one line, so bullets are split back out.
        Some(snippet) => truncate_lines(&strip_html(snippet).replace(" - ", "\n- "), SUMMARY_LINES, SUMMARY_CHARS),
        None => truncate_lines(&strip_html(content), SUMMARY_LINES, SUMMARY_CHARS),
    }
}

/// Plain text with one line per block or line break; runs of blank lines and spaces are collapsed.
fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    let mut tag = String::new();
    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                in_tag = false;
                let name = tag.trim_start_matches('/').split(|c: char| !c.is_ascii_alphanumeric()).next().unwrap_or("");
                if matches!(name.to_ascii_lowercase().as_str(), "br" | "p" | "div" | "li") {
                    out.push('\n');
                }
            }
            _ if in_tag => tag.push(c),
            _ => out.push(c),
        }
    }
    let decoded = out
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#039;", "'")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
        .replace("\\[", "[");
    let mut lines: Vec<String> =
        decoded.lines().map(|l| l.split_whitespace().collect::<Vec<_>>().join(" ")).filter(|l| !l.is_empty()).collect();
    if lines.last().is_some_and(|l| l == "Read more") {
        lines.pop();
    }
    if let Some(last) = lines.last_mut() {
        if let Some(head) = last.strip_suffix("Read more") {
            *last = head.trim_end().to_owned();
        }
    }
    lines.retain(|l| !l.is_empty());
    lines.join("\n")
}

/// Keeps whole lines up to the limits, cutting the last one on a word and marking any cut with an ellipsis.
fn truncate_lines(text: &str, max_lines: usize, max_chars: usize) -> String {
    let mut out: Vec<String> = Vec::new();
    let mut used = 0usize;
    let mut cut = false;
    for (i, line) in text.lines().enumerate() {
        let len = line.chars().count();
        if i >= max_lines || used >= max_chars {
            cut = true;
            break;
        }
        if used + len > max_chars {
            let room = max_chars - used;
            let piece: String = line.chars().take(room).collect();
            let piece = piece.rsplit_once(' ').map_or(piece.as_str(), |(head, _)| head).trim_end().to_owned();
            if !piece.is_empty() {
                out.push(piece);
            }
            cut = true;
            break;
        }
        out.push(line.to_owned());
        used += len;
    }
    let mut joined = out.join("\n");
    if cut {
        joined.push('…');
    }
    joined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_comes_from_the_title() {
        assert_eq!(kind_of("Minor Update - 09-16-2026"), "Minor Update");
        assert_eq!(kind_of("Matchmaking Update"), "Matchmaking Update");
        assert_eq!(kind_of("09-16-2026 Update"), "Patch notes");
        assert_eq!(kind_of("Major Update - 1-2-3"), "Major Update - 1-2-3");
        assert_eq!(kind_of("Ranked Update - Season 2 - 10-01-2026"), "Ranked Update - Season 2");
    }

    #[test]
    fn dates_are_strict() {
        assert!(is_date("09-16-2026"));
        assert!(!is_date("9-16-2026"));
        assert!(!is_date("09/16/2026"));
        assert!(!is_date("ab-16-2026"));
    }

    #[test]
    fn strips_tags_and_entities_keeping_line_breaks() {
        assert_eq!(strip_html("<p class=\"a\">One &amp; two</p><p></p><p>it&#039;s</p>"), "One & two\nit's");
        assert_eq!(strip_html("a<br />b<br>c"), "a\nb\nc");
        assert_eq!(strip_html("<b>\\[ General ]</b><br /><br />- x   y"), "[ General ]\n- x y");
        assert_eq!(strip_html("- text<br /><a href=\"x\">Read more</a>"), "- text");
    }

    #[test]
    fn truncation_keeps_whole_lines_and_marks_the_cut() {
        assert_eq!(truncate_lines("a\nb", 5, 100), "a\nb");
        assert_eq!(truncate_lines("a\nb\nc", 2, 100), "a\nb…");
        assert_eq!(truncate_lines("alpha beta gamma", 100, 12), "alpha beta…");
        assert_eq!(truncate_lines("one\ntwo three four", 100, 10), "one\ntwo…");
    }

    #[test]
    fn forum_link_previews_are_split_back_into_bullets() {
        let html = "<div class=\"js-unfurl-desc\">[ General ] - Base HP reduced - Parry is in-line</div>";
        assert_eq!(summary_of(html), "[ General ]\n- Base HP reduced\n- Parry is in-line");
    }

    #[test]
    fn image_must_be_steam_hosted_and_not_an_icon() {
        let html = r#"<img src="https://store.steampowered.com/favicon.ico"/><img src="https://evil.test/a.png"/>
            <img src="https://clan.akamai.steamstatic.com/images/1/a.png" class="x"/>"#;
        assert_eq!(first_steam_image(html).as_deref(), Some("https://clan.akamai.steamstatic.com/images/1/a.png"));
        assert_eq!(first_steam_image(r#"<img src="https://steamstatic.com.evil.test/a.png"/>"#), None);
        assert_eq!(first_steam_image("no images"), None);
        assert_eq!(first_steam_image("<img"), None);
    }

    const STEAM_URL: &str = "https://store.steampowered.com/news/app/1422450/view/1";

    fn feed() -> String {
        format!(
            r#"[
            {{"source":"forum","title":"09-16-2026 Update","pub_date":"2026-09-16T22:41:46Z","link":"https://forums.test/t/1",
              "guid":{{"text":"urn:forum:1"}},
              "content":"<div data-url=\"{STEAM_URL}\"><img src=\"https://clan.akamai.steamstatic.com/images/1/banner.png\"/><div class=\"contentRow-snippet js-unfurl-desc\">forum blurb</div></div>"}},
            {{"source":"steam","title":" Minor Update - 09-16-2026","pub_date":"2026-09-16T20:16:43Z","link":"{STEAM_URL}",
              "guid":{{"text":"{STEAM_URL}"}},"content":"<p class=\"bb_paragraph\">Guardian bounty &amp; more</p>"}},
            {{"source":"forum","title":"04-10-2026 Update","pub_date":"2026-04-10T00:00:00Z","link":"https://forums.test/t/2",
              "guid":{{"text":"urn:forum:2"}},"content":"<b>[ General ]</b><br />- Parrying is now allowed<br /><a href=\"x\">Read more</a>"}}
        ]"#
        )
    }

    #[test]
    fn a_forum_entry_folds_into_its_steam_post_and_lends_its_image() {
        let items = parse_feed(&feed()).unwrap();
        assert_eq!(items.len(), 2);
        let steam = items.iter().find(|a| a.source == "steam").unwrap();
        assert_eq!(steam.title, "Minor Update - 09-16-2026");
        assert_eq!(steam.kind, "Minor Update");
        assert_eq!(steam.image.as_deref(), Some("https://clan.akamai.steamstatic.com/images/1/banner.png"));
        assert_eq!(steam.summary, "Guardian bounty & more");
        assert!(items.iter().all(|a| a.id != "urn:forum:1"));
    }

    #[test]
    fn a_forum_entry_without_a_steam_twin_stays() {
        let items = parse_feed(&feed()).unwrap();
        let forum = items.iter().find(|a| a.id == "urn:forum:2").unwrap();
        assert_eq!(forum.kind, "Patch notes");
        assert_eq!(forum.summary, "[ General ]\n- Parrying is now allowed");
        assert_eq!(forum.image, None);
    }

    #[test]
    fn a_forum_entry_stays_when_its_steam_post_is_not_in_the_feed() {
        let json = feed().replace(&format!("\"link\":\"{STEAM_URL}\""), "\"link\":\"https://other.test/x\"");
        assert_eq!(parse_feed(&json).unwrap().len(), 3);
    }

    #[test]
    fn parses_ids_with_fallbacks_and_drops_unusable_items() {
        let items = parse_feed(r#"[{"title":"t","link":"https://x.test/a"}]"#).unwrap();
        assert_eq!(items[0].id, "https://x.test/a");
        assert!(parse_feed(r#"[{"title":"  "},{"guid":{"text":"g"}},{"title":"no id or link"}]"#).unwrap().is_empty());
        assert!(parse_feed(r#"{"error":"nope"}"#).is_err());
    }
}
