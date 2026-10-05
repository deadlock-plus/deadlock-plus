use crate::map::fit;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Values {
    pub hero: Option<String>,
    pub mode: Option<String>,
    pub game_mode: Option<String>,
    pub result: Option<String>,
    pub elapsed: Option<String>,
    pub kills: Option<String>,
    pub deaths: Option<String>,
    pub assists: Option<String>,
    pub souls: Option<String>,
}

/// Every placeholder a template may use. Names without a value source yet render empty.
pub const PLACEHOLDERS: [&str; 16] = [
    "hero",
    "mode",
    "gameMode",
    "rank",
    "kills",
    "deaths",
    "assists",
    "souls",
    "partySize",
    "partyMax",
    "round",
    "scoreAmber",
    "scoreSapphire",
    "elapsed",
    "queueTime",
    "result",
];

#[derive(Debug)]
enum Item {
    Word(String),
    Sep(String),
    /// A placeholder with no value. `inner` holds separators swallowed between adjacent empty placeholders.
    Gap {
        inner: String,
    },
}

fn is_sep(c: char) -> bool {
    c.is_whitespace() || "-\u{2013}\u{2014},/:;|&\u{b7}\u{2022}()[]".contains(c)
}

fn is_open(c: char) -> bool {
    c == '(' || c == '['
}

fn is_close(c: char) -> bool {
    c == ')' || c == ']'
}

fn value_of<'a>(name: &str, v: &'a Values) -> Option<Option<&'a str>> {
    let own = match name {
        "hero" => &v.hero,
        "mode" => &v.mode,
        "gameMode" => &v.game_mode,
        "result" => &v.result,
        "elapsed" => &v.elapsed,
        "kills" => &v.kills,
        "deaths" => &v.deaths,
        "assists" => &v.assists,
        "souls" => &v.souls,
        other if PLACEHOLDERS.contains(&other) => return Some(None),
        _ => return None,
    };
    Some(own.as_deref().filter(|s| !s.is_empty()))
}

fn push_literal(items: &mut Vec<Item>, text: &str) {
    let mut run = String::new();
    let mut run_sep = false;
    for c in text.chars() {
        if !run.is_empty() && is_sep(c) != run_sep {
            push_run(items, std::mem::take(&mut run), run_sep);
        }
        run_sep = is_sep(c);
        run.push(c);
    }
    if !run.is_empty() {
        push_run(items, run, run_sep);
    }
}

fn push_run(items: &mut Vec<Item>, run: String, sep: bool) {
    match (items.last_mut(), sep) {
        (Some(Item::Sep(s)), true) | (Some(Item::Word(s)), false) => s.push_str(&run),
        (_, true) => items.push(Item::Sep(run)),
        (_, false) => items.push(Item::Word(run)),
    }
}

fn push_gap(items: &mut Vec<Item>) {
    if matches!(items.last(), Some(Item::Gap { .. })) {
        return;
    }
    let after_gap = items.len() >= 2 && matches!(items[items.len() - 2], Item::Gap { .. });
    if after_gap && matches!(items.last(), Some(Item::Sep(_))) {
        let Some(Item::Sep(sep)) = items.pop() else { return };
        if let Some(Item::Gap { inner }) = items.last_mut() {
            inner.push_str(&sep);
        }
        return;
    }
    items.push(Item::Gap { inner: String::new() });
}

fn tokenize(template: &str, values: &Values) -> Vec<Item> {
    let mut items = Vec::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else { break };
        match value_of(&after[..end], values) {
            Some(value) => {
                push_literal(&mut items, &rest[..start]);
                match value {
                    Some(text) => match items.last_mut() {
                        Some(Item::Word(w)) => w.push_str(text),
                        _ => items.push(Item::Word(text.to_owned())),
                    },
                    None => push_gap(&mut items),
                }
                rest = &after[end + 1..];
            }
            None => {
                push_literal(&mut items, &rest[..=start]);
                rest = after;
            }
        }
    }
    push_literal(&mut items, rest);
    items
}

fn sep_mut(items: &mut [Item], i: Option<usize>) -> Option<&mut String> {
    match items.get_mut(i?) {
        Some(Item::Sep(s)) => Some(s),
        _ => None,
    }
}

/// Removes the separators and brackets that only existed to join an empty placeholder to its neighbours.
fn resolve_gaps(items: &mut [Item]) {
    for i in 0..items.len() {
        let Item::Gap { inner } = &items[i] else { continue };
        let mut opens = inner.chars().filter(|c| is_open(*c)).count();
        let has_word_before = items[..i].iter().any(|it| matches!(it, Item::Word(_)));
        let has_word_after = items[i + 1..].iter().any(|it| matches!(it, Item::Word(_)));
        let left = i.checked_sub(1);
        let right = Some(i + 1);

        if let Some(s) = sep_mut(items, left) {
            opens += s.chars().filter(|c| is_open(*c)).count();
            s.retain(|c| !is_open(c));
        }
        if let Some(s) = sep_mut(items, right) {
            let mut to_strip = opens;
            s.retain(|c| {
                if to_strip > 0 && is_close(c) {
                    to_strip -= 1;
                    return false;
                }
                true
            });
        }

        let has_left = left.is_some_and(|l| matches!(items[l], Item::Sep(_)));
        if !has_word_before || !has_word_after {
            if let Some(s) = sep_mut(items, left) {
                s.clear();
            }
            if let Some(s) = sep_mut(items, right) {
                s.clear();
            }
        } else if has_left {
            if let Some(s) = sep_mut(items, right) {
                s.clear();
            }
        }
    }
}

/// Resolves `[[a||b]]` groups: the first alternative whose known placeholders all have values is kept, or nothing if
/// none qualifies. An unclosed `[[` stays literal.
fn expand_groups(template: &str, values: &Values) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find("[[") {
        let Some(len) = rest[start + 2..].find("]]") else { break };
        let inner = &rest[start + 2..start + 2 + len];
        out.push_str(&rest[..start]);
        if let Some(alt) = inner.split("||").find(|alt| names_in(alt).all(|n| value_of(n, values) != Some(None))) {
            out.push_str(alt);
        }
        rest = &rest[start + len + 4..];
    }
    out.push_str(rest);
    out
}

fn names_in(text: &str) -> impl Iterator<Item = &str> {
    text.split('{').skip(1).filter_map(|part| part.split_once('}').map(|(name, _)| name))
}

pub fn render(template: &str, values: &Values) -> Option<String> {
    let template = expand_groups(template, values);
    let mut items = tokenize(&template, values);
    resolve_gaps(&mut items);
    let text: String = items
        .iter()
        .map(|item| match item {
            Item::Word(s) | Item::Sep(s) => s.as_str(),
            Item::Gap { .. } => "",
        })
        .collect();
    fit(&text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(t: &str, v: &Values) -> Option<String> {
        render(t, v)
    }

    fn hero(h: &str) -> Values {
        Values { hero: Some(h.into()), ..Values::default() }
    }

    #[test]
    fn optional_group_renders_when_its_placeholders_have_values() {
        assert_eq!(r("[[Playing as {hero}]]", &hero("Haze")).as_deref(), Some("Playing as Haze"));
    }

    #[test]
    fn optional_group_is_dropped_when_a_placeholder_is_empty() {
        assert_eq!(r("[[Playing as {hero}]]", &Values::default()), None);
        assert_eq!(r("In a match [[as {hero}]]", &Values::default()).as_deref(), Some("In a match"));
        assert_eq!(r("In a match [[as {hero}]]", &hero("Haze")).as_deref(), Some("In a match as Haze"));
    }

    #[test]
    fn optional_group_needs_every_placeholder() {
        let v = Values { hero: Some("Haze".into()), mode: None, ..Values::default() };
        assert_eq!(r("[[{hero} in {mode}]]", &v), None);
    }

    #[test]
    fn group_falls_back_to_the_next_alternative() {
        let t = "[[Hanging out as {hero}||Hanging out]]";
        assert_eq!(r(t, &hero("Haze")).as_deref(), Some("Hanging out as Haze"));
        assert_eq!(r(t, &Values::default()).as_deref(), Some("Hanging out"));
    }

    #[test]
    fn group_tries_alternatives_in_order() {
        let t = "[[{hero} in {mode}||{hero}||nobody]]";
        let both = Values { hero: Some("Haze".into()), mode: Some("Ranked".into()), ..Values::default() };
        assert_eq!(r(t, &both).as_deref(), Some("Haze in Ranked"));
        assert_eq!(r(t, &hero("Haze")).as_deref(), Some("Haze"));
        assert_eq!(r(t, &Values::default()).as_deref(), Some("nobody"));
    }

    #[test]
    fn group_with_no_usable_alternative_is_dropped() {
        assert_eq!(r("[[{hero}||{mode}]]", &Values::default()), None);
    }

    #[test]
    fn unclosed_optional_group_is_literal() {
        assert_eq!(r("[[Hello", &Values::default()).as_deref(), Some("[[Hello"));
    }

    #[test]
    fn plain_text_passes_through() {
        assert_eq!(r("In the Hideout", &Values::default()).as_deref(), Some("In the Hideout"));
    }

    #[test]
    fn each_available_placeholder_renders() {
        let v = Values {
            hero: Some("Haze".into()),
            mode: Some("Ranked".into()),
            game_mode: Some("Street Brawl".into()),
            result: Some("Won".into()),
            elapsed: Some("12:03".into()),
            ..Values::default()
        };
        assert_eq!(r("{hero}", &v).as_deref(), Some("Haze"));
        assert_eq!(r("{mode}", &v).as_deref(), Some("Ranked"));
        assert_eq!(r("{gameMode}", &v).as_deref(), Some("Street Brawl"));
        assert_eq!(r("{result}", &v).as_deref(), Some("Won"));
        assert_eq!(r("{elapsed}", &v).as_deref(), Some("12:03"));
    }

    #[test]
    fn planned_placeholders_are_recognised_and_render_empty() {
        for name in PLACEHOLDERS {
            let t = format!("a {{{name}}} b");
            let known = ["hero", "mode", "gameMode", "elapsed", "result"].contains(&name);
            if !known {
                assert_eq!(r(&t, &Values::default()).as_deref(), Some("a b"), "{name}");
            }
        }
    }

    #[test]
    fn unknown_placeholder_stays_literal() {
        assert_eq!(r("Hi {foo} there", &Values::default()).as_deref(), Some("Hi {foo} there"));
    }

    #[test]
    fn placeholder_names_are_case_sensitive() {
        assert_eq!(r("{Hero}", &hero("Haze")).as_deref(), Some("{Hero}"));
    }

    #[test]
    fn unclosed_brace_stays_literal() {
        assert_eq!(r("a {hero", &hero("Haze")).as_deref(), Some("a {hero"));
    }

    #[test]
    fn empty_value_in_a_lone_placeholder_omits_the_line() {
        assert_eq!(r("{hero}", &Values::default()), None);
    }

    #[test]
    fn empty_string_value_counts_as_empty() {
        let v = Values { hero: Some(String::new()), ..Values::default() };
        assert_eq!(r("Hero: {hero}", &v).as_deref(), Some("Hero"));
    }

    #[test]
    fn trailing_separator_is_dropped_with_an_empty_value() {
        let v = Values { game_mode: Some("Street Brawl".into()), ..Values::default() };
        assert_eq!(r("Playing {gameMode} - {mode}", &v).as_deref(), Some("Playing Street Brawl"));
    }

    #[test]
    fn leading_separator_is_dropped_with_an_empty_value() {
        let v = Values { mode: Some("Ranked".into()), ..Values::default() };
        assert_eq!(r("Playing {gameMode} - {mode}", &v).as_deref(), Some("Playing Ranked"));
    }

    #[test]
    fn both_values_keep_the_separator() {
        let v = Values { game_mode: Some("Street Brawl".into()), mode: Some("Ranked".into()), ..Values::default() };
        assert_eq!(r("Playing {gameMode} - {mode}", &v).as_deref(), Some("Playing Street Brawl - Ranked"));
    }

    #[test]
    fn middle_empty_value_keeps_one_separator() {
        let v = Values { hero: Some("A".into()), elapsed: Some("B".into()), ..Values::default() };
        assert_eq!(r("{hero} - {mode} - {elapsed}", &v).as_deref(), Some("A - B"));
    }

    #[test]
    fn leading_empty_value_drops_the_comma() {
        let v = Values { mode: Some("Ranked".into()), ..Values::default() };
        assert_eq!(r("{hero}, {mode}", &v).as_deref(), Some("Ranked"));
    }

    #[test]
    fn trailing_empty_value_drops_the_comma() {
        assert_eq!(r("{hero}, {mode}", &hero("Haze")).as_deref(), Some("Haze"));
    }

    #[test]
    fn slash_between_two_empty_values_is_dropped() {
        assert_eq!(r("Score {kills}/{deaths}", &Values::default()).as_deref(), Some("Score"));
    }

    #[test]
    fn party_line_degrades_to_the_hero_when_party_is_unknown() {
        assert_eq!(
            r("In party as {hero} ({partySize}/{partyMax})", &hero("Haze")).as_deref(),
            Some("In party as Haze")
        );
    }

    #[test]
    fn party_line_keeps_parentheses_when_a_party_exists() {
        // No party fact exists yet, so exercise the shape with a value that is present.
        let v = Values {
            hero: Some("Haze".into()),
            mode: Some("3".into()),
            game_mode: Some("6".into()),
            ..Values::default()
        };
        assert_eq!(r("In party as {hero} ({mode}/{gameMode})", &v).as_deref(), Some("In party as Haze (3/6)"));
    }

    #[test]
    fn party_line_without_hero_or_party_drops_everything_after_the_label() {
        assert_eq!(
            r("In party as {hero} ({partySize}/{partyMax})", &Values::default()).as_deref(),
            Some("In party as")
        );
    }

    #[test]
    fn parentheses_around_an_empty_value_are_dropped() {
        assert_eq!(r("{hero} ({mode})", &hero("Haze")).as_deref(), Some("Haze"));
    }

    #[test]
    fn square_brackets_around_an_empty_value_are_dropped() {
        assert_eq!(r("{hero} [{mode}]", &hero("Haze")).as_deref(), Some("Haze"));
    }

    #[test]
    fn values_are_not_split_on_inner_separators() {
        let v = Values { mode: Some("a - b (c)".into()), ..Values::default() };
        assert_eq!(r("{hero} {mode}", &v).as_deref(), Some("a - b (c)"));
    }

    #[test]
    fn long_output_is_truncated_through_fit() {
        let out = r("{hero}", &hero(&"h".repeat(300))).unwrap();
        assert_eq!(out.chars().count(), crate::MAX_TEXT_CHARS);
        assert!(out.ends_with('…'));
    }

    #[test]
    fn short_output_is_padded_through_fit() {
        assert_eq!(r("{hero}", &hero("H")).unwrap().chars().count(), crate::MIN_TEXT_CHARS);
    }

    #[test]
    fn blank_template_omits_the_line() {
        assert_eq!(r("   ", &Values::default()), None);
        assert_eq!(r("", &Values::default()), None);
    }
}
