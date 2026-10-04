pub const MAX_TEXT_CHARS: usize = 128;
pub const MIN_TEXT_CHARS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PresenceLevel {
    #[default]
    Off,
    Basic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GameFacts {
    pub running: bool,
    pub started_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Presence {
    pub details: Option<String>,
    pub state: Option<String>,
    pub start_timestamp: Option<i64>,
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
}

pub fn map(level: PresenceLevel, facts: &GameFacts) -> Option<Presence> {
    if level == PresenceLevel::Off || !facts.running {
        return None;
    }
    Some(Presence { details: fit("In game"), start_timestamp: facts.started_at, ..Presence::default() })
}

/// Discord rejects text under 2 characters; a zero-width space keeps a 1-char line valid without showing extra text.
fn fit(text: &str) -> Option<String> {
    let text = text.trim();
    let count = text.chars().count();
    if count == 0 {
        return None;
    }
    if count > MAX_TEXT_CHARS {
        let mut out: String = text.chars().take(MAX_TEXT_CHARS - 1).collect();
        out.push('…');
        return Some(out);
    }
    if count < MIN_TEXT_CHARS {
        return Some(format!("{text}\u{200B}"));
    }
    Some(text.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn running(started_at: Option<i64>) -> GameFacts {
        GameFacts { running: true, started_at }
    }

    #[test]
    fn off_maps_to_none() {
        assert_eq!(map(PresenceLevel::Off, &running(Some(1))), None);
    }

    #[test]
    fn not_running_maps_to_none() {
        assert_eq!(map(PresenceLevel::Basic, &GameFacts { running: false, started_at: Some(1) }), None);
    }

    #[test]
    fn basic_running_has_details_and_timestamp() {
        let p = map(PresenceLevel::Basic, &running(Some(1_700_000_000))).unwrap();
        assert_eq!(p.details.as_deref(), Some("In game"));
        assert_eq!(p.start_timestamp, Some(1_700_000_000));
        assert_eq!(p.state, None);
        assert_eq!(p.large_image, None);
    }

    #[test]
    fn unknown_start_time_omits_timestamp() {
        let p = map(PresenceLevel::Basic, &running(None)).unwrap();
        assert_eq!(p.start_timestamp, None);
        assert!(p.details.is_some());
    }

    #[test]
    fn fit_truncates_to_max_chars() {
        let long = "a".repeat(300);
        let out = fit(&long).unwrap();
        assert_eq!(out.chars().count(), MAX_TEXT_CHARS);
        assert!(out.ends_with('…'));
    }

    #[test]
    fn fit_truncates_on_char_boundaries() {
        let long = "é".repeat(200);
        assert_eq!(fit(&long).unwrap().chars().count(), MAX_TEXT_CHARS);
    }

    #[test]
    fn fit_keeps_exact_limit_untouched() {
        let s = "b".repeat(MAX_TEXT_CHARS);
        assert_eq!(fit(&s).unwrap(), s);
    }

    #[test]
    fn fit_pads_short_text() {
        let out = fit("x").unwrap();
        assert_eq!(out.chars().count(), MIN_TEXT_CHARS);
        assert!(out.starts_with('x'));
    }

    #[test]
    fn fit_drops_blank() {
        assert_eq!(fit(""), None);
        assert_eq!(fit("   "), None);
    }

    #[test]
    fn fit_trims() {
        assert_eq!(fit("  hi  ").as_deref(), Some("hi"));
    }
}
