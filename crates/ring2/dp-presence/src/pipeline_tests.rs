use crate::config::{Config, PartialSlot, Timer};
use crate::map::{
    map, map_with, Context, GameFacts, GameMode, LiveFacts, MatchMode, Perspective, Phase, Presence, PresenceLevel,
    MAX_TEXT_CHARS, MIN_TEXT_CHARS,
};
use crate::state::{StateId, VariantId};

fn facts(f: impl FnOnce(&mut LiveFacts)) -> GameFacts {
    let mut l = LiveFacts { context: Context::Match, phase: Some(Phase::InProgress), ..LiveFacts::default() };
    f(&mut l);
    GameFacts { running: true, started_at: Some(100), now_secs: 10_000, live: Some(l) }
}

fn slot(details: Option<&str>, state: Option<&str>) -> PartialSlot {
    PartialSlot { details: details.map(Into::into), state: state.map(Into::into), ..PartialSlot::default() }
}

fn cfg(state: StateId, p: PartialSlot) -> Config {
    let mut c = Config::default();
    c.states.insert(state, p);
    c
}

fn detailed(f: &GameFacts, c: &Config) -> Presence {
    map_with(PresenceLevel::Detailed, f, c).unwrap()
}

#[test]
fn map_is_map_with_the_default_config() {
    let f = facts(|l| l.hero = Some("Haze".into()));
    assert_eq!(map(PresenceLevel::Detailed, &f), map_with(PresenceLevel::Detailed, &f, &Config::default()));
    assert_eq!(detailed(&f, &Config::default()).details.as_deref(), Some("In a match"));
}

#[test]
fn off_and_not_running_ignore_the_config() {
    let f = facts(|_| {});
    assert_eq!(map_with(PresenceLevel::Off, &f, &Config::default()), None);
    let mut stopped = f.clone();
    stopped.running = false;
    assert_eq!(map_with(PresenceLevel::Detailed, &stopped, &Config::default()), None);
}

#[test]
fn state_config_changes_the_lines() {
    let c = cfg(StateId::InMatch, slot(Some("Fighting as {hero}"), Some("{mode}")));
    let f = facts(|l| {
        l.hero = Some("Haze".into());
        l.match_mode = Some(MatchMode::Ranked);
    });
    let p = detailed(&f, &c);
    assert_eq!(p.details.as_deref(), Some("Fighting as Haze"));
    assert_eq!(p.state.as_deref(), Some("Ranked"));
}

#[test]
fn variant_config_applies_only_to_its_variant() {
    let mut c = Config::default();
    c.variants.entry(StateId::InMatch).or_default().insert(VariantId::Ranked, slot(Some("Grinding"), None));
    let ranked = facts(|l| l.match_mode = Some(MatchMode::Ranked));
    let unranked = facts(|l| l.match_mode = Some(MatchMode::Unranked));
    assert_eq!(detailed(&ranked, &c).details.as_deref(), Some("Grinding"));
    assert_eq!(detailed(&unranked, &c).details.as_deref(), Some("Playing Unranked"));
}

#[test]
fn hero_override_is_addressed_by_hero_id() {
    let mut c = Config::default();
    c.heroes.entry(7).or_default().states.insert(StateId::InMatch, slot(Some("Seven mode"), None));
    let seven = facts(|l| {
        l.hero_id = Some(7);
        l.hero = Some("Seven".into());
    });
    let other = facts(|l| l.hero_id = Some(8));
    let no_id = facts(|l| l.hero = Some("Seven".into()));
    assert_eq!(detailed(&seven, &c).details.as_deref(), Some("Seven mode"));
    assert_eq!(detailed(&other, &c).details.as_deref(), Some("In a match"));
    assert_eq!(detailed(&no_id, &c).details.as_deref(), Some("In a match"));
}

#[test]
fn disabled_state_sends_no_activity() {
    let c = cfg(StateId::InMatch, PartialSlot { enabled: Some(false), ..PartialSlot::default() });
    assert_eq!(map_with(PresenceLevel::Detailed, &facts(|_| {}), &c), None);
    let menu = facts(|l| l.context = Context::Other);
    assert!(map_with(PresenceLevel::Detailed, &menu, &c).is_some());
}

#[test]
fn disabled_playing_state_sends_nothing_at_basic() {
    let c = cfg(StateId::Playing, PartialSlot { enabled: Some(false), ..PartialSlot::default() });
    assert_eq!(map_with(PresenceLevel::Basic, &facts(|_| {}), &c), None);
}

#[test]
fn playing_config_shapes_basic() {
    let c = cfg(StateId::Playing, slot(Some("Deadlocking"), Some("with Deadlock+")));
    let p = map_with(PresenceLevel::Basic, &facts(|_| {}), &c).unwrap();
    assert_eq!(p.details.as_deref(), Some("Deadlocking"));
    assert_eq!(p.state.as_deref(), Some("with Deadlock+"));
    assert_eq!(p.start_timestamp, Some(100));
}

#[test]
fn drift_falls_back_to_playing_and_ignores_other_state_config() {
    let mut c = cfg(StateId::InMatch, slot(Some("custom"), None));
    c.heroes.entry(7).or_default().states.insert(StateId::InMatch, slot(Some("hero"), None));
    let f = facts(|l| {
        l.drift = true;
        l.hero_id = Some(7);
    });
    assert_eq!(map_with(PresenceLevel::Detailed, &f, &c), map(PresenceLevel::Basic, &f));
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("In game"));
}

#[test]
fn missing_live_data_falls_back_to_playing() {
    let c = cfg(StateId::MainMenu, slot(Some("custom"), None));
    let f = GameFacts { running: true, started_at: Some(5), ..GameFacts::default() };
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("In game"));
}

#[test]
fn spectating_defaults_hide_the_hero() {
    let f = facts(|l| {
        l.perspective = Perspective::Spectating;
        l.hero = Some("Haze".into());
        l.hero_id = Some(7);
    });
    assert_eq!(detailed(&f, &Config::default()).state, None);
}

#[test]
fn spectating_hero_is_an_explicit_opt_in() {
    let c = cfg(StateId::Spectating, slot(None, Some("{hero}")));
    let f = facts(|l| {
        l.perspective = Perspective::Spectating;
        l.hero = Some("Haze".into());
    });
    assert_eq!(detailed(&f, &c).state.as_deref(), Some("Haze"));
}

#[test]
fn spectating_ignores_hero_overrides() {
    let mut c = Config::default();
    c.heroes.entry(7).or_default().states.insert(StateId::Spectating, slot(Some("watching Seven"), None));
    let f = facts(|l| {
        l.perspective = Perspective::Spectating;
        l.hero_id = Some(7);
    });
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("Spectating a match"));
}

#[test]
fn spectating_hero_and_result_stay_hidden_in_other_states() {
    let mut c = Config::default();
    c.states.insert(StateId::PostGame, slot(Some("{hero}"), Some("{result}")));
    c.states.insert(StateId::HeroSelect, slot(Some("Picking {hero}"), None));
    for phase in [Phase::PostGame, Phase::HeroSelection] {
        let f = facts(|l| {
            l.perspective = Perspective::Spectating;
            l.phase = Some(phase);
            l.hero = Some("Haze".into());
            l.local_won = Some(true);
        });
        let p = detailed(&f, &c);
        let text = format!("{:?}{:?}", p.details, p.state);
        assert!(!text.contains("Haze") && !text.contains("Won"), "{phase:?} {text}");
    }
}

#[test]
fn private_lobby_defaults_hide_hero_and_score() {
    let f = facts(|l| {
        l.match_mode = Some(MatchMode::PrivateLobby);
        l.hero = Some("Haze".into());
        l.local_won = Some(true);
    });
    let p = detailed(&f, &Config::default());
    assert_eq!(p.details.as_deref(), Some("In a private match"));
    assert_eq!(p.state, None);
}

#[test]
fn private_lobby_hero_is_an_explicit_opt_in() {
    let c = cfg(StateId::PrivateLobby, slot(None, Some("{hero}")));
    let f = facts(|l| {
        l.match_mode = Some(MatchMode::PrivateLobby);
        l.hero = Some("Haze".into());
    });
    assert_eq!(detailed(&f, &c).state.as_deref(), Some("Haze"));
}

#[test]
fn private_lobby_ignores_hero_overrides() {
    let mut c = Config::default();
    c.heroes.entry(7).or_default().states.insert(StateId::PrivateLobby, slot(Some("custom"), None));
    let f = facts(|l| {
        l.match_mode = Some(MatchMode::PrivateLobby);
        l.hero_id = Some(7);
    });
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("In a private match"));
}

#[test]
fn post_game_result_placeholder() {
    let won = facts(|l| {
        l.phase = Some(Phase::PostGame);
        l.local_won = Some(true);
    });
    let c = cfg(StateId::PostGame, slot(Some("Game over: {result}"), None));
    assert_eq!(detailed(&won, &c).details.as_deref(), Some("Game over: Won"));
    let unscored = facts(|l| l.phase = Some(Phase::PostGame));
    assert_eq!(detailed(&unscored, &c).details.as_deref(), Some("Game over"));
}

#[test]
fn elapsed_placeholder_formats_match_time() {
    let c = cfg(StateId::InMatch, slot(Some("{elapsed}"), None));
    for (secs, want) in [(65.9, "1:05"), (0.0, "0:00"), (3_725.0, "1:02:05")] {
        let f = facts(|l| l.match_time_secs = Some(secs));
        assert_eq!(detailed(&f, &c).details.as_deref(), Some(want));
    }
}

#[test]
fn elapsed_placeholder_is_empty_without_a_valid_match_time() {
    let c = cfg(StateId::InMatch, slot(Some("Time {elapsed}"), None));
    for t in [None, Some(f32::NAN), Some(-1.0)] {
        let f = facts(|l| l.match_time_secs = t);
        assert_eq!(detailed(&f, &c).details.as_deref(), Some("Time"));
    }
}

#[test]
fn mode_placeholders_render() {
    let c = cfg(StateId::InMatch, slot(Some("{gameMode}|{mode}"), None));
    let f = facts(|l| {
        l.match_mode = Some(MatchMode::CoopBot);
        l.game_mode = Some(GameMode::StreetBrawl);
    });
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("Street Brawl|vs bots"));
}

#[test]
fn unavailable_placeholders_render_empty() {
    let c = cfg(StateId::InMatch, slot(Some("K{kills} D{deaths} {rank} {queueTime}"), Some("{partySize}/{partyMax}")));
    let p = detailed(&facts(|_| {}), &c);
    assert_eq!(p.details.as_deref(), Some("K D"));
    assert_eq!(p.state, None);
}

#[test]
fn timer_none_sends_no_timestamp() {
    let c = cfg(StateId::InMatch, PartialSlot { timer: Some(Timer::None), ..PartialSlot::default() });
    assert_eq!(detailed(&facts(|l| l.match_time_secs = Some(60.0)), &c).start_timestamp, None);
}

#[test]
fn timer_elapsed_in_state_uses_the_session_start() {
    let c = cfg(StateId::InMatch, PartialSlot { timer: Some(Timer::ElapsedInState), ..PartialSlot::default() });
    assert_eq!(detailed(&facts(|l| l.match_time_secs = Some(60.0)), &c).start_timestamp, Some(100));
}

#[test]
fn timer_match_time_needs_a_running_clock() {
    let c = cfg(StateId::Hideout, PartialSlot { timer: Some(Timer::MatchTime), ..PartialSlot::default() });
    let running = facts(|l| {
        l.context = Context::Hideout;
        l.match_time_secs = Some(60.0);
    });
    assert_eq!(detailed(&running, &c).start_timestamp, Some(10_000 - 60));
    let paused = facts(|l| {
        l.context = Context::Hideout;
        l.match_time_secs = Some(60.0);
        l.paused = true;
    });
    assert_eq!(detailed(&paused, &c).start_timestamp, None);
}

#[test]
fn hover_texts_are_rendered() {
    let c = cfg(
        StateId::InMatch,
        PartialSlot {
            large_text: Some("Playing {mode}".into()),
            small_text: Some("{hero}".into()),
            ..PartialSlot::default()
        },
    );
    let f = facts(|l| {
        l.match_mode = Some(MatchMode::Ranked);
        l.hero = Some("Haze".into());
    });
    let p = detailed(&f, &c);
    assert_eq!(p.large_text.as_deref(), Some("Playing Ranked"));
    assert_eq!(p.small_text.as_deref(), Some("Haze"));
}

#[test]
fn user_text_is_truncated_and_padded() {
    let c = cfg(StateId::InMatch, slot(Some(&"x".repeat(300)), Some("y")));
    let p = detailed(&facts(|_| {}), &c);
    assert_eq!(p.details.unwrap().chars().count(), MAX_TEXT_CHARS);
    assert_eq!(p.state.unwrap().chars().count(), MIN_TEXT_CHARS);
}

#[test]
fn images_are_carried_but_not_resolved_to_urls() {
    let p = detailed(&facts(|_| {}), &Config::default());
    assert_eq!((p.large_image, p.small_image), (None, None));
}

#[test]
fn paused_match_uses_the_paused_slot_and_keeps_the_mode_label() {
    let f = facts(|l| {
        l.paused = true;
        l.match_mode = Some(MatchMode::Ranked);
        l.hero = Some("Haze".into());
    });
    let c = cfg(StateId::Paused, slot(Some("Paused ({mode})"), None));
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("Paused (Ranked)"));
    assert_eq!(detailed(&f, &Config::default()).details.as_deref(), Some("Playing Ranked"));
}
