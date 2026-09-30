pub fn between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let from = text.find(start)? + start.len();
    let len = text[from..].find(end)?;
    Some(&text[from..from + len])
}

/// Decodes the fixed named entities feed content uses plus any numeric character reference
/// (`&#8203;`, `&#x200B;`). Both the RSS/HTML feed and the Steam BBCode `contents` field can carry
/// numeric entities — confirmed live: Steam's own zero-width-space padding (`&#8203;`) survived
/// un-decoded because the old fixed allowlist only covered a few named entities.
pub fn decode_entities(s: &str) -> String {
    let named = s
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&");
    decode_numeric_entities(&named)
}

fn decode_numeric_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("&#") {
        out.push_str(&rest[..start]);
        let after_hash = &rest[start + 2..];
        let hex = after_hash.starts_with(['x', 'X']);
        let digits = if hex { &after_hash[1..] } else { after_hash };
        let digit_len =
            digits.chars().take_while(|c| if hex { c.is_ascii_hexdigit() } else { c.is_ascii_digit() }).count();
        if digit_len > 0 && digits[digit_len..].starts_with(';') {
            let matched_len = 2 + usize::from(hex) + digit_len + 1;
            match u32::from_str_radix(&digits[..digit_len], if hex { 16 } else { 10 }).ok().and_then(char::from_u32) {
                Some(decoded) => out.push(decoded),
                None => out.push_str(&rest[start..start + matched_len]),
            }
            rest = &rest[start + matched_len..];
        } else {
            out.push_str("&#");
            rest = after_hash;
        }
    }
    out.push_str(rest);
    out
}

/// Plain text with one line per block or line break; runs of blank lines and spaces are collapsed.
pub fn strip_html(html: &str) -> String {
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
    let decoded = decode_entities(&out).replace("\\[", "[");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_tags_and_entities_keeping_line_breaks() {
        assert_eq!(strip_html("<p class=\"a\">One &amp; two</p><p></p><p>it&#039;s</p>"), "One & two\nit's");
        assert_eq!(strip_html("a<br />b<br>c"), "a\nb\nc");
        assert_eq!(strip_html("<b>\\[ General ]</b><br /><br />- x   y"), "[ General ]\n- x y");
        assert_eq!(strip_html("- text<br /><a href=\"x\">Read more</a>"), "- text");
    }

    #[test]
    fn decodes_numeric_character_references() {
        // Live bug: Steam pads text with a literal zero-width space entity that survived
        // un-decoded (`&#8203;`) because the old allowlist only handled a few named entities.
        assert_eq!(decode_entities("3.5&#8203;"), "3.5\u{200B}");
        assert_eq!(decode_entities("&#x200B;"), "\u{200B}");
        assert_eq!(decode_entities("&#039;"), "'");
        assert_eq!(decode_entities("&#39;"), "'");
    }

    #[test]
    fn an_incomplete_or_invalid_numeric_reference_is_left_alone() {
        assert_eq!(decode_entities("a &# b"), "a &# b");
        assert_eq!(decode_entities("a &#xyz; b"), "a &#xyz; b");
        assert_eq!(decode_entities("a &#99999999999; b"), "a &#99999999999; b");
        assert_eq!(decode_entities("no entities here"), "no entities here");
    }

    #[test]
    fn between_finds_the_first_bracketed_span() {
        assert_eq!(between("a[b]c", "[", "]"), Some("b"));
        assert_eq!(between("no markers", "[", "]"), None);
    }
}
