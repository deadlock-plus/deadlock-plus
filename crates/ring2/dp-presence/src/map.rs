pub const MAX_TEXT_CHARS: usize = 128;
pub const MIN_TEXT_CHARS: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PresenceLevel {
    #[default]
    Off,
    Basic,
    Detailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Context {
    #[default]
    Other,
    Hideout,
    Match,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    HeroSelection,
    MatchIntro,
    Loading,
    PreGame,
    InProgress,
    PostGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Perspective {
    #[default]
    Unknown,
    Playing,
    Spectating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchMode {
    Unranked,
    Ranked,
    PrivateLobby,
    CoopBot,
    HeroLabs,
    Tutorial,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Normal,
    StreetBrawl,
    Sandbox,
    ExploreNyc,
    Other,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LiveFacts {
    pub context: Context,
    pub phase: Option<Phase>,
    pub perspective: Perspective,
    pub match_mode: Option<MatchMode>,
    pub game_mode: Option<GameMode>,
    pub hero: Option<String>,
    pub match_time_secs: Option<f32>,
    pub paused: bool,
    pub drift: bool,
    pub local_won: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GameFacts {
    pub running: bool,
    pub started_at: Option<i64>,
    /// Current unix time in seconds; anchors the match timer.
    pub now_secs: i64,
    pub live: Option<LiveFacts>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Button {
    pub label: String,
    pub url: String,
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
    pub buttons: Vec<Button>,
}

pub fn map(level: PresenceLevel, facts: &GameFacts) -> Option<Presence> {
    if level == PresenceLevel::Off || !facts.running {
        return None;
    }
    if level == PresenceLevel::Detailed {
        if let Some(live) = facts.live.as_ref().filter(|l| !l.drift) {
            return Some(detailed(facts, live));
        }
    }
    Some(basic(facts))
}

fn basic(facts: &GameFacts) -> Presence {
    Presence { details: fit("In game"), start_timestamp: facts.started_at, ..Presence::default() }
}

fn detailed(facts: &GameFacts, live: &LiveFacts) -> Presence {
    let spectating = live.perspective == Perspective::Spectating;
    let session = facts.started_at;
    let line = |details: &str, state: Option<&str>, start: Option<i64>| Presence {
        details: fit(details),
        state: state.and_then(fit),
        start_timestamp: start,
        ..Presence::default()
    };

    if live.context == Context::Match && live.match_mode == Some(MatchMode::PrivateLobby) {
        return line("In a private match", None, session);
    }

    let hero = if spectating { None } else { live.hero.as_deref() };
    match live.context {
        Context::Other => line("In the main menu", None, session),
        Context::Hideout => line("In the Hideout", hero, session),
        Context::Match => match live.phase {
            Some(Phase::HeroSelection) => line("Choosing a hero", None, session),
            Some(Phase::MatchIntro | Phase::Loading) => line("Loading into a match", None, session),
            Some(Phase::PreGame) => line("Waiting for the match to start", hero, session),
            Some(Phase::PostGame) => {
                let result = match (spectating, live.local_won) {
                    (false, Some(true)) => Some("Won"),
                    (false, Some(false)) => Some("Lost"),
                    _ => None,
                };
                line("Match finished", result, session)
            }
            Some(Phase::InProgress) | None => {
                let details = if spectating {
                    "Spectating a match".to_owned()
                } else {
                    mode_label(live).map_or_else(|| "In a match".to_owned(), |label| format!("Playing {label}"))
                };
                let start = match live.match_time_secs {
                    Some(t) if !live.paused && t.is_finite() && t >= 0.0 => Some(facts.now_secs - t as i64),
                    _ => None,
                };
                line(&details, hero, start)
            }
        },
    }
}

fn mode_label(live: &LiveFacts) -> Option<String> {
    let game = match live.game_mode {
        Some(GameMode::StreetBrawl) => Some("Street Brawl"),
        Some(GameMode::Sandbox) => Some("Sandbox"),
        Some(GameMode::ExploreNyc) => Some("Explore NYC"),
        _ => None,
    };
    let queue = match live.match_mode {
        Some(MatchMode::Unranked) => Some("Unranked"),
        Some(MatchMode::Ranked) => Some("Ranked"),
        Some(MatchMode::CoopBot) => Some("vs bots"),
        Some(MatchMode::HeroLabs) => Some("Hero Labs"),
        Some(MatchMode::Tutorial) => Some("Tutorial"),
        _ => None,
    };
    match (game, queue) {
        (Some(g), Some(q)) => Some(format!("{g} - {q}")),
        (Some(x), None) | (None, Some(x)) => Some(x.to_owned()),
        (None, None) => None,
    }
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
        GameFacts { running: true, started_at, ..GameFacts::default() }
    }

    #[test]
    fn off_maps_to_none() {
        assert_eq!(map(PresenceLevel::Off, &running(Some(1))), None);
    }

    #[test]
    fn not_running_maps_to_none() {
        assert_eq!(
            map(PresenceLevel::Basic, &GameFacts { running: false, started_at: Some(1), ..GameFacts::default() }),
            None
        );
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
    fn basic_sends_no_buttons() {
        assert!(map(PresenceLevel::Basic, &running(None)).unwrap().buttons.is_empty());
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

    fn live(f: impl FnOnce(&mut LiveFacts)) -> GameFacts {
        let mut l = LiveFacts { context: Context::Match, phase: Some(Phase::InProgress), ..LiveFacts::default() };
        f(&mut l);
        GameFacts { running: true, started_at: Some(100), now_secs: 10_000, live: Some(l) }
    }

    fn detailed_of(facts: &GameFacts) -> Presence {
        map(PresenceLevel::Detailed, facts).unwrap()
    }

    #[test]
    fn detailed_without_live_matches_basic() {
        let f = running(Some(5));
        assert_eq!(map(PresenceLevel::Detailed, &f), map(PresenceLevel::Basic, &f));
    }

    #[test]
    fn detailed_with_drift_matches_basic() {
        let f = live(|l| {
            l.drift = true;
            l.hero = Some("Haze".into());
        });
        assert_eq!(map(PresenceLevel::Detailed, &f), map(PresenceLevel::Basic, &f));
    }

    #[test]
    fn detailed_not_running_is_none() {
        let mut f = live(|_| {});
        f.running = false;
        assert_eq!(map(PresenceLevel::Detailed, &f), None);
    }

    #[test]
    fn basic_ignores_live() {
        let f = live(|l| l.hero = Some("Haze".into()));
        assert_eq!(map(PresenceLevel::Basic, &f).unwrap().details.as_deref(), Some("In game"));
    }

    #[test]
    fn main_menu() {
        let p = detailed_of(&live(|l| {
            l.context = Context::Other;
            l.phase = None;
        }));
        assert_eq!(p.details.as_deref(), Some("In the main menu"));
        assert_eq!(p.start_timestamp, Some(100));
    }

    #[test]
    fn hideout() {
        let p = detailed_of(&live(|l| {
            l.context = Context::Hideout;
            l.phase = Some(Phase::InProgress);
        }));
        assert_eq!(p.details.as_deref(), Some("In the Hideout"));
        assert_eq!(p.state, None);
    }

    #[test]
    fn hero_selection() {
        let p = detailed_of(&live(|l| l.phase = Some(Phase::HeroSelection)));
        assert_eq!(p.details.as_deref(), Some("Choosing a hero"));
    }

    #[test]
    fn loading_phases() {
        for phase in [Phase::MatchIntro, Phase::Loading] {
            let p = detailed_of(&live(|l| l.phase = Some(phase)));
            assert_eq!(p.details.as_deref(), Some("Loading into a match"));
        }
    }

    #[test]
    fn pre_game() {
        let p = detailed_of(&live(|l| {
            l.phase = Some(Phase::PreGame);
            l.hero = Some("Haze".into());
        }));
        assert_eq!(p.details.as_deref(), Some("Waiting for the match to start"));
        assert_eq!(p.state.as_deref(), Some("Haze"));
    }

    #[test]
    fn in_match_mode_labels() {
        let cases = [
            (Some(MatchMode::Ranked), Some(GameMode::Normal), "Playing Ranked"),
            (Some(MatchMode::Unranked), None, "Playing Unranked"),
            (Some(MatchMode::CoopBot), Some(GameMode::Normal), "Playing vs bots"),
            (Some(MatchMode::HeroLabs), None, "Playing Hero Labs"),
            (Some(MatchMode::Tutorial), None, "Playing Tutorial"),
            (Some(MatchMode::Unranked), Some(GameMode::StreetBrawl), "Playing Street Brawl - Unranked"),
            (None, Some(GameMode::Sandbox), "Playing Sandbox"),
            (None, Some(GameMode::ExploreNyc), "Playing Explore NYC"),
            (Some(MatchMode::Other), Some(GameMode::Other), "In a match"),
            (None, None, "In a match"),
        ];
        for (mm, gm, want) in cases {
            let p = detailed_of(&live(|l| {
                l.match_mode = mm;
                l.game_mode = gm;
            }));
            assert_eq!(p.details.as_deref(), Some(want), "{mm:?} {gm:?}");
        }
    }

    #[test]
    fn in_match_hero_is_bottom_line() {
        let p = detailed_of(&live(|l| l.hero = Some("Haze".into())));
        assert_eq!(p.state.as_deref(), Some("Haze"));
    }

    #[test]
    fn in_match_without_hero_has_no_state() {
        assert_eq!(detailed_of(&live(|_| {})).state, None);
    }

    #[test]
    fn match_timer_is_now_minus_match_time() {
        let p = detailed_of(&live(|l| l.match_time_secs = Some(600.9)));
        assert_eq!(p.start_timestamp, Some(10_000 - 600));
    }

    #[test]
    fn paused_drops_timer() {
        let p = detailed_of(&live(|l| {
            l.match_time_secs = Some(600.0);
            l.paused = true;
        }));
        assert_eq!(p.start_timestamp, None);
    }

    #[test]
    fn unknown_match_time_drops_timer() {
        assert_eq!(detailed_of(&live(|_| {})).start_timestamp, None);
    }

    #[test]
    fn post_game_won_lost_unscored() {
        for (won, want) in [(Some(true), Some("Won")), (Some(false), Some("Lost")), (None, None)] {
            let p = detailed_of(&live(|l| {
                l.phase = Some(Phase::PostGame);
                l.local_won = won;
            }));
            assert_eq!(p.details.as_deref(), Some("Match finished"));
            assert_eq!(p.state.as_deref(), want);
        }
    }

    #[test]
    fn spectating_hides_hero() {
        let p = detailed_of(&live(|l| {
            l.perspective = Perspective::Spectating;
            l.hero = Some("Haze".into());
            l.match_mode = Some(MatchMode::Ranked);
            l.match_time_secs = Some(60.0);
        }));
        assert_eq!(p.details.as_deref(), Some("Spectating a match"));
        assert_eq!(p.state, None);
        assert_eq!(p.start_timestamp, Some(10_000 - 60));
    }

    #[test]
    fn spectating_hides_post_game_result() {
        let p = detailed_of(&live(|l| {
            l.perspective = Perspective::Spectating;
            l.phase = Some(Phase::PostGame);
            l.local_won = Some(true);
        }));
        assert_eq!(p.state, None);
    }

    #[test]
    fn private_lobby_shows_only_private_match() {
        for phase in [Phase::HeroSelection, Phase::Loading, Phase::InProgress, Phase::PostGame] {
            let p = detailed_of(&live(|l| {
                l.phase = Some(phase);
                l.match_mode = Some(MatchMode::PrivateLobby);
                l.hero = Some("Haze".into());
                l.match_time_secs = Some(60.0);
                l.local_won = Some(true);
            }));
            assert_eq!(p.details.as_deref(), Some("In a private match"));
            assert_eq!(p.state, None);
            assert_eq!(p.start_timestamp, Some(100));
        }
    }

    #[test]
    fn long_hero_name_is_truncated() {
        let p = detailed_of(&live(|l| l.hero = Some("h".repeat(300))));
        assert_eq!(p.state.unwrap().chars().count(), MAX_TEXT_CHARS);
    }

    #[test]
    fn one_char_hero_name_is_padded() {
        let p = detailed_of(&live(|l| l.hero = Some("H".into())));
        assert_eq!(p.state.unwrap().chars().count(), MIN_TEXT_CHARS);
    }
}
