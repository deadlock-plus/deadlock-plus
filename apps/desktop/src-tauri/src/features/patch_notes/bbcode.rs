//! Strips the BBCode `ISteamNews/GetNewsForApp`'s `contents` field uses down to the same
//! plain-line convention `text::strip_html` produces for the RSS-derived feed, so `parse_body`'s
//! section/bullet parsing works unmodified on either source. See `plans/patch-notes-full-text.md`
//! for the tag survey this is built from.
//!
//! `[b]`/`[i]` are kept, not discarded: they become `**bold**`/`_italic_` (the same convention
//! Markdown uses), a lightweight, deterministic signal the frontend renders as real emphasis
//! instead of guessing which lines are headings from their length. A resolved, Steam-hosted
//! `[img]` URL is collected into `strip_bbcode`'s second return value (see `alert::feed::first_steam_image`
//! for the same host trust rule), but the tag also leaves an [`image_marker`] sentinel behind in
//! the text, on its own line, so the position it held in the body survives `parse_body`'s line
//! splitting — the frontend swaps each sentinel for the image it names instead of grouping every
//! image at the top of the post regardless of where it actually sat. The API's `contents` field
//! uses `{STEAM_CLAN_LOC_IMAGE}` as a literal, unresolved template placeholder for its image CDN
//! base, the same CDN the HTML feed's images already come from.

use dp_text::decode_entities;

const CLAN_IMAGE_BASE: &str = "https://clan.akamai.steamstatic.com/images";

/// U+FFFC (OBJECT REPLACEMENT CHARACTER) is built for exactly this: standing in for embedded
/// content a text stream can't carry itself. Vanishingly unlikely to appear in a real patch note,
/// and visually distinct if it ever leaks into a log or an error message.
const MARKER_DELIM: char = '\u{fffc}';

/// The sentinel `strip_bbcode` leaves in place of an `[img]` tag, naming its index into the
/// `images` vec it also returns. `parse_body` carries it through unchanged as an ordinary line's
/// `raw` (see `parse_image_marker`), and the frontend resolves it against `PatchDetail.images`.
pub(crate) fn image_marker(index: usize) -> String {
    format!("{MARKER_DELIM}{index}{MARKER_DELIM}")
}

/// The `images` index an [`image_marker`] line names, if `raw` (already trimmed) is one.
pub(crate) fn parse_image_marker(raw: &str) -> Option<usize> {
    raw.strip_prefix(MARKER_DELIM)?.strip_suffix(MARKER_DELIM)?.parse().ok()
}

/// The Steam Web API never resolves this itself — only Steam's own client does. Both the
/// legacy and localised placeholder names point at the same CDN base in practice.
fn resolve_clan_placeholder(url: &str) -> String {
    url.replace("{STEAM_CLAN_LOC_IMAGE}", CLAN_IMAGE_BASE).replace("{STEAM_CLAN_IMAGE}", CLAN_IMAGE_BASE)
}

/// Only a resolved, Steam-hosted URL is trusted as an image to display, matching
/// `alerts::feed::first_steam_image`'s own guard for the same reason: nothing else in the body is
/// under Valve's control.
fn is_steam_hosted(url: &str) -> bool {
    url.strip_prefix("https://").and_then(|s| s.split('/').next()).is_some_and(|h| h.ends_with(".steamstatic.com"))
}

fn tag_name(inner: &str) -> (bool, String) {
    let is_close = inner.starts_with('/');
    let body = inner.trim_start_matches('/');
    let name = body.split(|c: char| c == '=' || c.is_whitespace()).next().unwrap_or("");
    (is_close, name.to_ascii_lowercase())
}

/// `[img]`/`[video]` content (a raw media URL) is dropped from the text stream along with the tag;
/// for `[img]` specifically, that URL is also returned once it resolves to a real Steam-hosted
/// address. Most instances are a matched pair; a few are a single self-closed tag with the URL only
/// in an attribute, so the closing search is bounded by the next same-name opening tag to avoid
/// swallowing real content when no close ever comes.
fn skip_media(bbcode: &str, end: usize, name: &str, tag_inner: &str) -> (usize, Option<String>) {
    let rest = &bbcode[end + 1..];
    let close = format!("[/{name}]");
    let reopen = format!("[{name}");
    let (new_end, between) = match (rest.find(&close), rest.find(&reopen)) {
        (Some(c), Some(o)) if c < o => (end + 1 + c + close.len(), Some(rest[..c].trim())),
        (Some(c), None) => (end + 1 + c + close.len(), Some(rest[..c].trim())),
        _ => (end + 1, None),
    };
    if name != "img" {
        return (new_end, None);
    }
    let raw_url = between
        .filter(|u| !u.is_empty())
        .or_else(|| tag_inner.split_whitespace().find_map(|p| p.strip_prefix("src=")))
        .map(|u| u.trim_matches('"'));
    let url = raw_url.map(resolve_clan_placeholder).filter(|u| is_steam_hosted(u));
    (new_end, url)
}

pub fn strip_bbcode(bbcode: &str) -> (String, Vec<String>) {
    // Steam's `contents` field HTML-entity-encodes some characters even though it's otherwise
    // BBCode, not HTML — confirmed live: a literal `&#8203;` (zero-width space) survived because
    // nothing in this module decoded entities at all.
    let decoded = decode_entities(bbcode);
    let bbcode = decoded.as_str();
    let mut out = String::with_capacity(bbcode.len());
    let mut images = Vec::new();
    let bytes = bbcode.as_bytes();
    let mut i = 0usize;
    while i < bbcode.len() {
        if bytes[i] == b'\\' && bbcode[i + 1..].starts_with('[') {
            out.push('[');
            i += 2;
            continue;
        }
        if bytes[i] == b'[' {
            if let Some(rel_end) = bbcode[i..].find(']') {
                let end = i + rel_end;
                let tag_inner = &bbcode[i + 1..end];
                let (is_close, name) = tag_name(tag_inner);
                match name.as_str() {
                    "img" | "video" if !is_close => {
                        let (new_i, url) = skip_media(bbcode, end, &name, tag_inner);
                        if let Some(url) = url {
                            out.push('\n');
                            out.push_str(&image_marker(images.len()));
                            out.push('\n');
                            images.push(url);
                        }
                        i = new_i;
                    }
                    "p" | "list" => {
                        out.push('\n');
                        i = end + 1;
                    }
                    "*" if !is_close => {
                        out.push_str("\n- ");
                        i = end + 1;
                    }
                    "h3" => {
                        out.push_str(if is_close { " ]\n" } else { "\n[ " });
                        i = end + 1;
                    }
                    "b" => {
                        out.push_str("**");
                        i = end + 1;
                    }
                    "i" => {
                        out.push('_');
                        i = end + 1;
                    }
                    "u" | "url" | "img" | "video" | "*" => i = end + 1,
                    // Not a tag we know: most likely a literal `[ Section ]` header with no
                    // escaping at all (confirmed live — some posts skip both the backslash and
                    // the `[b]`/`[h3]` wrapper other posts use for the same convention). Keep the
                    // brackets rather than risk silently deleting real section text.
                    _ => {
                        out.push('[');
                        out.push_str(tag_inner);
                        out.push(']');
                        i = end + 1;
                    }
                }
                continue;
            }
        }
        let ch = bbcode[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    let text = out
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    (text, images)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(bbcode: &str) -> String {
        strip_bbcode(bbcode).0
    }

    #[test]
    fn strips_underline_and_url_tags_keeping_inner_text() {
        assert_eq!(text("[u]10%[/u] via [url=https://x.test]a link[/url]"), "10% via a link");
    }

    #[test]
    fn bold_and_italic_become_markdown_style_markers() {
        assert_eq!(text("[b]Guardian[/b] bounty [i]increased[/i]"), "**Guardian** bounty _increased_");
    }

    #[test]
    fn a_bold_wrapped_bracket_header_still_reads_as_a_section() {
        // Real post shape: `[p][b][ General ][/b][/p]` — bold-wrapping doesn't stop the bracket
        // convention `parse_body`'s `section_re` looks for.
        assert_eq!(text("[p][b][ General ][/b][/p]"), "**[ General ]**");
    }

    #[test]
    fn paragraphs_become_line_breaks() {
        assert_eq!(text("[p]One[/p][p]Two[/p]"), "One\nTwo");
    }

    #[test]
    fn headings_become_bracketed_sections() {
        assert_eq!(text("[h3]Heroes[/h3][p]Abrams buffed[/p]"), "[ Heroes ]\nAbrams buffed");
    }

    #[test]
    fn urls_keep_their_text_and_drop_the_link() {
        assert_eq!(text("[url=https://x.test]Read more[/url]"), "Read more");
    }

    #[test]
    fn video_is_dropped_entirely_and_never_collected_as_an_image() {
        assert_eq!(text("[video mp4]https://x.test/a.mp4[/video]text"), "text");
        assert!(strip_bbcode("[video mp4]https://x.test/a.mp4[/video]text").1.is_empty());
    }

    #[test]
    fn a_steam_hosted_image_leaves_a_marker_where_it_sat_and_is_collected_separately() {
        let (text, images) =
            strip_bbcode("[p]Before[/p][img]https://clan.akamai.steamstatic.com/images/1/a.png[/img][p]After[/p]");
        assert_eq!(text, format!("Before\n{}\nAfter", image_marker(0)));
        assert_eq!(images, vec!["https://clan.akamai.steamstatic.com/images/1/a.png"]);
    }

    #[test]
    fn markers_index_images_in_the_order_they_appear() {
        let (text, images) = strip_bbcode(
            "[p]One[/p][img]https://clan.akamai.steamstatic.com/images/1/a.png[/img][p]Two[/p][img]https://clan.akamai.steamstatic.com/images/1/b.png[/img][p]Three[/p]",
        );
        assert_eq!(text, format!("One\n{}\nTwo\n{}\nThree", image_marker(0), image_marker(1)));
        assert_eq!(
            images,
            vec![
                "https://clan.akamai.steamstatic.com/images/1/a.png",
                "https://clan.akamai.steamstatic.com/images/1/b.png"
            ]
        );
    }

    #[test]
    fn a_marker_round_trips_back_to_its_index() {
        assert_eq!(parse_image_marker(&image_marker(0)), Some(0));
        assert_eq!(parse_image_marker(&image_marker(7)), Some(7));
        assert_eq!(parse_image_marker("- Guardian bounty increased by 10%"), None);
    }

    #[test]
    fn a_clan_image_placeholder_is_resolved_to_the_real_cdn_base() {
        let (_, images) = strip_bbcode("[img]{STEAM_CLAN_LOC_IMAGE}/45164767/f6a6d5.png[/img]");
        assert_eq!(images, vec!["https://clan.akamai.steamstatic.com/images/45164767/f6a6d5.png"]);
    }

    #[test]
    fn an_image_from_an_untrusted_host_is_dropped_and_not_collected() {
        let (text, images) = strip_bbcode("before[img]https://evil.test/a.png[/img]after");
        assert_eq!(text, "beforeafter");
        assert!(images.is_empty());
    }

    #[test]
    fn a_self_closed_image_tag_with_no_matching_close_drops_only_itself() {
        assert_eq!(text("[img src=https://x.test/a.png]after text"), "after text");
    }

    #[test]
    fn a_self_closed_image_tags_src_attribute_is_still_collected_when_trusted() {
        let (_, images) = strip_bbcode("[img src=https://clan.akamai.steamstatic.com/images/1/a.png]after text");
        assert_eq!(images, vec!["https://clan.akamai.steamstatic.com/images/1/a.png"]);
    }

    #[test]
    fn list_items_become_dash_prefixed_bullets() {
        assert_eq!(text("[list][*]Item one[*]Item two[/list]"), "- Item one\n- Item two");
    }

    #[test]
    fn a_bare_unescaped_section_header_is_kept_not_deleted() {
        // Confirmed live: some posts write `[ Section ]` with no backslash and no [b]/[h3]
        // wrapper at all, unlike the escaped-and-wrapped convention every other post uses.
        assert_eq!(
            text("[img]https://x.test/a.png[/img]\n\n[ Shop Redesign ]\n\n- Full shop rework"),
            "[ Shop Redesign ]\n- Full shop rework"
        );
    }

    #[test]
    fn a_linked_image_with_no_separate_text_is_dropped_entirely() {
        assert_eq!(text("[url=https://x.test][img]https://x.test/a.png[/img][/url]\n\nafter"), "after");
    }

    #[test]
    fn html_entities_in_the_bbcode_body_are_decoded() {
        // Live bug: "Fleetfoot: Move speed reduced from 3.5&#8203;m/s" rendered the entity
        // literally — `strip_bbcode` never decoded entities at all, unlike `text::strip_html`.
        assert_eq!(text("Move speed reduced from 3.5&#8203;m/s"), "Move speed reduced from 3.5\u{200B}m/s");
        assert_eq!(text("Guardian bounty &amp; more"), "Guardian bounty & more");
    }

    #[test]
    fn escaped_literal_brackets_are_unescaped_like_the_html_feed() {
        assert_eq!(
            text("\\[ General ]\n- Guardian bounty increased by 10%"),
            "[ General ]\n- Guardian bounty increased by 10%"
        );
    }
}
