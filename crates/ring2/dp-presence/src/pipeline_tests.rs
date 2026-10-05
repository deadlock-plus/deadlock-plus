use crate::config::{Config, ImageSource, PartialImage, PartialSlot, Timer};
use crate::map::{
    map, map_with, Context, GameFacts, GameMode, LiveFacts, MatchMode, PartyFacts, Perspective, Phase, Presence,
    PresenceLevel, StreetBrawlFacts, MAX_TEXT_CHARS, MIN_TEXT_CHARS,
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
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("Playing Deadlock"));
}

#[test]
fn missing_live_data_falls_back_to_playing() {
    let c = cfg(StateId::MainMenu, slot(Some("custom"), None));
    let f = GameFacts { running: true, started_at: Some(5), ..GameFacts::default() };
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("Playing Deadlock"));
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
        l.game_mode = Some(GameMode::Sandbox);
    });
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("Sandbox|vs bots"));
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
fn no_image_is_sent_when_there_is_no_art_to_point_at() {
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

fn scored(l: &mut LiveFacts) {
    l.kills = Some(12);
    l.deaths = Some(3);
    l.assists = Some(8);
    l.souls = Some(24_100);
}

#[test]
fn score_placeholders_render_own_values() {
    let c = cfg(StateId::InMatch, slot(Some("{kills}/{deaths}/{assists}"), Some("{souls} souls")));
    let p = detailed(&facts(scored), &c);
    assert_eq!(p.details.as_deref(), Some("12/3/8"));
    assert_eq!(p.state.as_deref(), Some("24.1k souls"));
}

#[test]
fn souls_are_shown_raw_below_a_thousand() {
    let c = cfg(StateId::InMatch, slot(Some("{souls} souls"), None));
    assert_eq!(detailed(&facts(|l| l.souls = Some(950)), &c).details.as_deref(), Some("950 souls"));
}

#[test]
fn score_is_withheld_while_spectating() {
    let c = cfg(StateId::Spectating, slot(Some("K{kills} D{deaths} {souls}"), Some("Watching")));
    let p = detailed(
        &facts(|l| {
            scored(l);
            l.perspective = Perspective::Spectating;
        }),
        &c,
    );
    assert_eq!(p.details.as_deref(), Some("K D"));
}

#[test]
fn souls_round_to_one_decimal_of_a_thousand() {
    let c = cfg(StateId::InMatch, slot(Some("{souls} souls"), None));
    for (souls, want) in [(0, "0"), (999, "999"), (1000, "1k"), (24_149, "24.1k"), (24_150, "24.2k"), (30_000, "30k")] {
        let got = detailed(&facts(|l| l.souls = Some(souls)), &c).details;
        assert_eq!(got, Some(format!("{want} souls")), "{souls}");
    }
}

fn typed_match_id() -> Config {
    cfg(StateId::InMatch, slot(Some("Match {matchId}"), Some("{hero}")))
}

#[test]
fn match_id_renders_only_where_the_user_typed_it() {
    let f = facts(|l| l.match_id = Some(123_456_789));
    assert_eq!(detailed(&f, &typed_match_id()).details.as_deref(), Some("Match 123456789"));
}

#[test]
fn match_id_keeps_all_digits_of_a_wide_id() {
    let f = facts(|l| l.match_id = Some(u64::from(u32::MAX) + 7));
    assert_eq!(detailed(&f, &typed_match_id()).details.as_deref(), Some("Match 4294967302"));
}

#[test]
fn match_id_renders_while_spectating_when_typed() {
    let c = cfg(StateId::Spectating, slot(Some("Match {matchId}"), Some("Watching")));
    let f = facts(|l| {
        l.match_id = Some(123_456_789);
        l.perspective = Perspective::Spectating;
    });
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("Match 123456789"));
}

#[test]
fn match_id_is_withheld_in_a_private_lobby() {
    let c = cfg(StateId::PrivateLobby, slot(Some("Match {matchId}"), Some("Lobby")));
    let f = facts(|l| {
        l.match_id = Some(123_456_789);
        l.match_mode = Some(MatchMode::PrivateLobby);
    });
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("Match"));
}

#[test]
fn match_id_is_withheld_below_detailed() {
    let f = facts(|l| l.match_id = Some(123_456_789));
    let p = map_with(PresenceLevel::Basic, &f, &typed_match_id()).unwrap();
    assert_eq!(p.details.as_deref(), Some("Playing Deadlock"));
}

#[test]
fn no_built_in_template_uses_a_sensitive_placeholder() {
    use crate::config::builtin_config;
    use crate::template::SENSITIVE_PLACEHOLDERS;
    let c = builtin_config();
    let slots =
        c.states.values().chain(c.variants.values().flat_map(|v| v.values())).chain(
            c.heroes.values().flat_map(|h| h.states.values().chain(h.variants.values().flat_map(|v| v.values()))),
        );
    for s in slots {
        for text in [&s.details, &s.state, &s.large_text, &s.small_text].into_iter().flatten() {
            for name in SENSITIVE_PLACEHOLDERS {
                assert!(!text.contains(&format!("{{{name}}}")), "{text}");
            }
        }
    }
}

#[test]
fn every_state_lists_the_variants_classify_can_produce() {
    use crate::state::classify;
    let phases =
        [Phase::HeroSelection, Phase::MatchIntro, Phase::Loading, Phase::PreGame, Phase::InProgress, Phase::PostGame];
    let modes = [None, Some(GameMode::StreetBrawl), Some(GameMode::Sandbox), Some(GameMode::ExploreNyc)];
    let matches = [
        None,
        Some(MatchMode::Unranked),
        Some(MatchMode::Ranked),
        Some(MatchMode::CoopBot),
        Some(MatchMode::HeroLabs),
        Some(MatchMode::Tutorial),
    ];
    for phase in phases {
        for game_mode in modes {
            for match_mode in matches {
                for (paused, won) in [(false, None), (true, None), (false, Some(true)), (false, Some(false))] {
                    for perspective in [Perspective::Playing, Perspective::Spectating] {
                        let l = LiveFacts {
                            context: Context::Match,
                            phase: Some(phase),
                            game_mode,
                            match_mode,
                            paused,
                            local_won: won,
                            perspective,
                            ..LiveFacts::default()
                        };
                        let (state, variant) = classify(&l);
                        if let Some(v) = variant {
                            assert!(state.variants().contains(&v), "{state:?} lists no {v:?}");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn all_states_are_listed_once() {
    let mut seen = std::collections::HashSet::new();
    assert!(StateId::ALL.iter().all(|s| seen.insert(*s)));
    assert_eq!(StateId::ALL.len(), 14);
}

#[test]
fn builtin_config_resolves_like_the_empty_config() {
    let built = crate::builtin_config();
    for state in StateId::ALL {
        for variant in std::iter::once(None).chain(state.variants().iter().copied().map(Some)) {
            for hero in [None, Some(7)] {
                assert_eq!(
                    crate::resolve_slot(&built, state, variant, hero),
                    crate::resolve_slot(&Config::default(), state, variant, hero),
                    "{state:?} {variant:?}"
                );
            }
        }
    }
    assert!(built.states.contains_key(&StateId::Hideout));
    assert!(built.variants[&StateId::InMatch].contains_key(&VariantId::Ranked));
}

#[test]
fn preview_fills_sample_values() {
    let c = cfg(StateId::InMatch, slot(Some("{hero} {kills}/{deaths}/{assists}"), Some("{souls} {elapsed}")));
    let p = crate::preview(&c, StateId::InMatch, None, None, Some("Haze")).unwrap();
    assert_eq!(p.details.as_deref(), Some("Haze 12/3/8"));
    assert_eq!(p.state.as_deref(), Some("24.1k 12:34"));
    assert!(p.start_timestamp.is_some());
}

#[test]
fn preview_uses_the_variant_and_the_result_sample() {
    let p = crate::preview(&Config::default(), StateId::PostGame, Some(VariantId::Won), None, None).unwrap();
    assert_eq!(p.state.as_deref(), Some("Won"));
    let ranked =
        crate::preview(&Config::default(), StateId::InMatch, Some(VariantId::Ranked), None, Some("Haze")).unwrap();
    assert_eq!(ranked.details.as_deref(), Some("Playing Ranked"));
}

#[test]
fn preview_is_none_for_a_disabled_slot() {
    let c = cfg(StateId::Hideout, PartialSlot { enabled: Some(false), ..PartialSlot::default() });
    assert_eq!(crate::preview(&c, StateId::Hideout, None, None, None), None);
}

#[test]
fn built_in_hero_lines_drop_their_lead_in_without_a_hero() {
    let with = facts(|l| l.hero = Some("Haze".into()));
    let without = facts(|l| l.hero = None);
    assert_eq!(detailed(&with, &Config::default()).state.as_deref(), Some("Playing as Haze"));
    assert_eq!(detailed(&without, &Config::default()).state, None);
}

fn queue_facts(f: impl FnOnce(&mut LiveFacts)) -> GameFacts {
    facts(|l| {
        l.context = Context::Hideout;
        l.phase = None;
        l.party = Some(PartyFacts {
            size: 3,
            queueing: true,
            queued_secs: Some(75),
            match_mode: Some(MatchMode::Ranked),
            game_mode: Some(GameMode::Normal),
        });
        f(l);
    })
}

fn hideout_facts(party: Option<PartyFacts>) -> GameFacts {
    facts(|l| {
        l.context = Context::Hideout;
        l.phase = None;
        l.hero = Some("Haze".into());
        l.party = party;
    })
}

#[test]
fn party_placeholders_show_the_own_party() {
    let c = cfg(StateId::FindingMatch, slot(Some("Party {partySize}/{partyMax}"), Some("Queued {queueTime}")));
    let p = detailed(&queue_facts(|_| {}), &c);
    assert_eq!(p.details.as_deref(), Some("Party 3/6"));
    assert_eq!(p.state.as_deref(), Some("Queued 1:15"));
}

#[test]
fn finding_match_mode_placeholder_names_the_requested_mode() {
    let c = cfg(StateId::FindingMatch, slot(Some("Queue: {mode}"), None));
    assert_eq!(detailed(&queue_facts(|_| {}), &c).details.as_deref(), Some("Queue: Ranked"));
}

#[test]
fn finding_match_ignores_the_modes_of_a_previous_match() {
    let c = cfg(StateId::FindingMatch, slot(Some("Queue: {mode}"), None));
    let f = queue_facts(|l| {
        l.match_mode = Some(MatchMode::HeroLabs);
        l.game_mode = Some(GameMode::Sandbox);
    });
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("Queue: Ranked"));
}

#[test]
fn queue_time_is_empty_when_not_queueing() {
    let c = cfg(StateId::Hideout, slot(Some("In the Hideout {queueTime}"), None));
    let f = hideout_facts(Some(PartyFacts { size: 1, ..PartyFacts::default() }));
    let mut c = c;
    c.variants.clear();
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("In the Hideout"));
}

#[test]
fn party_values_are_empty_while_the_party_is_unknown() {
    let c = cfg(StateId::Hideout, slot(Some("Party {partySize}/{partyMax}"), None));
    assert_eq!(detailed(&hideout_facts(None), &c).details.as_deref(), Some("Party"));
}

#[test]
fn party_values_are_withheld_while_spectating() {
    let c = cfg(StateId::Spectating, slot(Some("Watching {partySize}"), Some("Hi")));
    let f = facts(|l| {
        l.perspective = Perspective::Spectating;
        l.party = Some(PartyFacts { size: 4, ..PartyFacts::default() });
    });
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("Watching"));
}

#[test]
fn party_values_are_withheld_in_a_private_lobby() {
    let c = cfg(StateId::PrivateLobby, slot(Some("Lobby {partySize}"), Some("Hi")));
    let f = facts(|l| {
        l.match_mode = Some(MatchMode::PrivateLobby);
        l.party = Some(PartyFacts { size: 4, ..PartyFacts::default() });
    });
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("Lobby"));
}

#[test]
fn finding_match_timer_counts_from_the_queue_start() {
    let c = cfg(StateId::FindingMatch, PartialSlot { timer: Some(Timer::QueueTime), ..PartialSlot::default() });
    assert_eq!(detailed(&queue_facts(|_| {}), &c).start_timestamp, Some(10_000 - 75));
}

#[test]
fn queue_timer_is_none_without_a_queue() {
    let c = cfg(StateId::Hideout, PartialSlot { timer: Some(Timer::QueueTime), ..PartialSlot::default() });
    let f = hideout_facts(Some(PartyFacts { size: 1, ..PartyFacts::default() }));
    assert_eq!(detailed(&f, &c).start_timestamp, None);
}

#[test]
fn built_in_finding_match_copy_names_the_queue() {
    let p = detailed(&queue_facts(|_| {}), &Config::default());
    assert_eq!(p.details.as_deref(), Some("Looking for a Ranked match"));
    assert_eq!(p.start_timestamp, Some(10_000 - 75));
    let none = queue_facts(|l| l.party.as_mut().unwrap().match_mode = None);
    assert_eq!(detailed(&none, &Config::default()).details.as_deref(), Some("Looking for a match"));
}

#[test]
fn built_in_hideout_lines_follow_the_party() {
    let party = hideout_facts(Some(PartyFacts { size: 3, ..PartyFacts::default() }));
    let p = detailed(&party, &Config::default());
    assert_eq!(p.details.as_deref(), Some("Relaxing in the Hideout"));
    assert_eq!(p.state.as_deref(), Some("In a party of 3 as Haze"));
    let solo = hideout_facts(Some(PartyFacts { size: 1, ..PartyFacts::default() }));
    assert_eq!(detailed(&solo, &Config::default()).state.as_deref(), Some("Hanging out as Haze"));
}

#[test]
fn preview_timer_is_relative_to_the_preview_clock() {
    let p = crate::preview(&Config::default(), StateId::InMatch, None, None, None).unwrap();
    assert_eq!(crate::PREVIEW_NOW_SECS - p.start_timestamp.unwrap(), 754);
}

const PORTRAIT: &str = "https://assets.example/haze_card.png";
const ICON: &str = "https://assets.example/haze_sm.png";

fn with_art(l: &mut LiveFacts) {
    l.hero = Some("Haze".into());
    l.hero_portrait = Some(PORTRAIT.into());
    l.hero_icon = Some(ICON.into());
}

fn image_slot(large: Option<ImageSource>, small: Option<ImageSource>) -> PartialSlot {
    let image = |source: Option<ImageSource>| PartialImage { enabled: Some(source.is_some()), source };
    PartialSlot { large_image: Some(image(large)), small_image: Some(image(small)), ..PartialSlot::default() }
}

#[test]
fn hero_icon_is_the_default_large_image() {
    let p = detailed(&facts(with_art), &Config::default());
    assert_eq!(p.large_image.as_deref(), Some(ICON));
}

#[test]
fn hero_icon_falls_back_to_the_portrait_when_there_is_no_icon() {
    let p = detailed(
        &facts(|l| {
            with_art(l);
            l.hero_icon = None;
        }),
        &Config::default(),
    );
    assert_eq!(p.large_image.as_deref(), Some(PORTRAIT));
}

#[test]
fn hero_portrait_source_uses_the_card() {
    let c = cfg(StateId::InMatch, image_slot(Some(ImageSource::HeroPortrait), None));
    assert_eq!(detailed(&facts(with_art), &c).large_image.as_deref(), Some(PORTRAIT));
}

#[test]
fn large_image_is_on_by_default_only_where_a_hero_is_known() {
    use crate::config::resolve_slot;
    let on = |state| resolve_slot(&Config::default(), state, None, None).large_image.enabled;
    for state in [
        StateId::Hideout,
        StateId::PreGame,
        StateId::InMatch,
        StateId::StreetBrawlRound,
        StateId::Paused,
        StateId::PostGame,
        StateId::Practice,
    ] {
        assert!(on(state), "{state:?}");
    }
    for state in [
        StateId::Playing,
        StateId::MainMenu,
        StateId::HeroSelect,
        StateId::FindingMatch,
        StateId::MatchFound,
        StateId::Spectating,
        StateId::PrivateLobby,
    ] {
        assert!(!on(state), "{state:?}");
    }
}

#[test]
fn small_image_is_on_by_default_only_for_a_ranked_match() {
    use crate::config::resolve_slot;
    let small = |state, variant| resolve_slot(&Config::default(), state, variant, None).small_image;
    for state in [StateId::InMatch, StateId::Paused] {
        let ranked = small(state, Some(VariantId::Ranked));
        assert!(ranked.enabled, "{state:?}");
        assert_eq!(ranked.source, ImageSource::RankBadge);
        assert!(!small(state, Some(VariantId::Unranked)).enabled, "{state:?}");
        assert!(!small(state, None).enabled, "{state:?}");
    }
    for state in StateId::ALL {
        assert!(!small(state, None).enabled, "{state:?}");
    }
}

#[test]
fn no_hero_art_means_no_image() {
    let p = detailed(&facts(|l| l.hero = Some("Haze".into())), &Config::default());
    assert_eq!(p.large_image, None);
}

#[test]
fn spectating_shows_no_hero_art_by_default_and_only_when_opted_in() {
    let f = facts(|l| {
        with_art(l);
        l.perspective = Perspective::Spectating;
    });
    assert_eq!(detailed(&f, &Config::default()).large_image, None);
    let c = cfg(StateId::Spectating, image_slot(Some(ImageSource::HeroPortrait), None));
    assert_eq!(detailed(&f, &c).large_image.as_deref(), Some(PORTRAIT));
}

#[test]
fn spectating_withholds_hero_art_in_states_that_never_opt_in() {
    let f = facts(|l| {
        with_art(l);
        l.perspective = Perspective::Spectating;
        l.phase = Some(Phase::PostGame);
    });
    let c = cfg(StateId::PostGame, image_slot(Some(ImageSource::HeroPortrait), None));
    assert_eq!(detailed(&f, &c).large_image, None);
}

#[test]
fn a_private_lobby_shows_no_art_by_default() {
    let f = facts(|l| {
        with_art(l);
        l.match_mode = Some(MatchMode::PrivateLobby);
    });
    let p = detailed(&f, &Config::default());
    assert_eq!((p.large_image, p.large_text), (None, None));
}

#[test]
fn a_disabled_image_is_not_sent() {
    let c = cfg(StateId::InMatch, image_slot(None, None));
    let p = detailed(&facts(with_art), &c);
    assert_eq!((p.large_image, p.small_image), (None, None));
}

#[test]
fn basic_sends_no_images() {
    let p = map_with(PresenceLevel::Basic, &facts(with_art), &Config::default()).unwrap();
    assert_eq!((p.large_image, p.small_image), (None, None));
}

fn ranked(rank: Option<u32>) -> GameFacts {
    facts(|l| {
        l.match_mode = Some(MatchMode::Ranked);
        l.rank = rank;
    })
}

#[test]
fn rank_badge_is_the_default_small_image_in_a_ranked_match() {
    let p = detailed(&ranked(Some(53)), &Config::default());
    assert_eq!(p.small_image.as_deref(), Some("https://api.deadlock-api.com/v1/assets/ranks/5/3/image"));
}

#[test]
fn a_badge_without_a_subrank_uses_the_tier_image() {
    let p = detailed(&ranked(Some(50)), &Config::default());
    assert_eq!(
        p.small_image.as_deref(),
        Some("https://assets-bucket.deadlock-api.com/assets-api-res/images/ranks/rank05_lg.png")
    );
}

#[test]
fn rank_badge_needs_a_rank_a_ranked_match_and_the_own_player() {
    assert_eq!(detailed(&ranked(None), &Config::default()).small_image, None);
    assert_eq!(detailed(&ranked(Some(0)), &Config::default()).small_image, None);
    let unranked = facts(|l| {
        l.match_mode = Some(MatchMode::Unranked);
        l.rank = Some(53);
    });
    assert_eq!(detailed(&unranked, &Config::default()).small_image, None);
    let spectating = facts(|l| {
        l.match_mode = Some(MatchMode::Ranked);
        l.rank = Some(53);
        l.perspective = Perspective::Spectating;
    });
    assert_eq!(detailed(&spectating, &Config::default()).small_image, None);
}

#[test]
fn custom_url_is_sent_only_when_it_is_a_short_https_address() {
    let sent = |url: &str| {
        let c = cfg(StateId::InMatch, image_slot(Some(ImageSource::CustomUrl(url.into())), None));
        detailed(&facts(|_| {}), &c).large_image
    };
    assert_eq!(sent("https://example.com/a.png").as_deref(), Some("https://example.com/a.png"));
    assert_eq!(sent("http://example.com/a.png"), None);
    assert_eq!(sent("not a url"), None);
    assert_eq!(sent(""), None);
    assert_eq!(sent("https://example.com/a b.png"), None);
    assert_eq!(sent(&format!("https://example.com/{}", "a".repeat(260))), None);
}

#[test]
fn mode_icon_has_no_art_to_send() {
    let c = cfg(StateId::InMatch, image_slot(Some(ImageSource::ModeIcon), None));
    assert_eq!(detailed(&facts(with_art), &c).large_image, None);
}

#[test]
fn rank_placeholder_joins_the_tier_name_and_subrank() {
    let c = cfg(StateId::InMatch, slot(Some("{rank}"), None));
    let f = |name: Option<&str>, rank: u32| {
        facts(|l| {
            l.match_mode = Some(MatchMode::Ranked);
            l.rank = Some(rank);
            l.rank_name = name.map(Into::into);
        })
    };
    assert_eq!(detailed(&f(Some("Acolyte"), 33), &c).details.as_deref(), Some("Acolyte 3"));
    assert_eq!(detailed(&f(Some("Acolyte"), 30), &c).details.as_deref(), Some("Acolyte"));
    assert_eq!(detailed(&f(None, 33), &c).details, None);
}

#[test]
fn built_in_hover_texts_name_the_hero_and_the_rank() {
    let f = facts(|l| {
        with_art(l);
        l.match_mode = Some(MatchMode::Ranked);
        l.rank = Some(33);
        l.rank_name = Some("Acolyte".into());
    });
    let p = detailed(&f, &Config::default());
    assert_eq!(p.large_text.as_deref(), Some("Haze"));
    assert_eq!(p.small_text.as_deref(), Some("Acolyte 3"));
}

#[test]
fn preview_draws_the_sample_art_and_rank() {
    use crate::PreviewSample;
    let sample = PreviewSample {
        hero_name: Some("Haze".into()),
        hero_portrait: Some(PORTRAIT.into()),
        hero_icon: Some(ICON.into()),
        rank_name: Some("Acolyte".into()),
        hero_presence: None,
    };
    let p = crate::preview_with(&Config::default(), StateId::InMatch, Some(VariantId::Ranked), None, &sample).unwrap();
    assert_eq!(p.large_image.as_deref(), Some(ICON));
    assert_eq!(p.small_image.as_deref(), Some("https://api.deadlock-api.com/v1/assets/ranks/5/2/image"));
    assert_eq!((p.large_text.as_deref(), p.small_text.as_deref()), (Some("Haze"), Some("Acolyte 2")));
    let unranked =
        crate::preview_with(&Config::default(), StateId::InMatch, Some(VariantId::Unranked), None, &sample).unwrap();
    assert_eq!(unranked.small_image, None);
}

#[test]
fn preview_without_a_sample_has_no_art() {
    let p = crate::preview(&Config::default(), StateId::InMatch, None, None, None).unwrap();
    assert_eq!((p.large_image, p.small_image), (None, None));
}

fn party_of(size: u32) -> impl FnOnce(&mut LiveFacts) {
    move |l| l.party = Some(PartyFacts { size, ..PartyFacts::default() })
}

#[test]
fn party_size_is_sent_for_a_party_of_two_or_more() {
    let p = detailed(&facts(party_of(3)), &Config::default());
    assert_eq!(p.party, Some((3, crate::PARTY_MAX)));
}

#[test]
fn a_party_of_one_or_no_party_sends_no_party_size() {
    assert_eq!(detailed(&facts(party_of(1)), &Config::default()).party, None);
    assert_eq!(detailed(&facts(|_| {}), &Config::default()).party, None);
}

#[test]
fn party_size_is_withheld_while_spectating_in_a_private_lobby_and_below_detailed() {
    let spectating = facts(|l| {
        party_of(3)(l);
        l.perspective = Perspective::Spectating;
    });
    assert_eq!(detailed(&spectating, &Config::default()).party, None);
    let private = facts(|l| {
        party_of(3)(l);
        l.match_mode = Some(MatchMode::PrivateLobby);
    });
    assert_eq!(detailed(&private, &Config::default()).party, None);
    let basic = map_with(PresenceLevel::Basic, &facts(party_of(3)), &Config::default()).unwrap();
    assert_eq!(basic.party, None);
}

#[test]
fn party_size_can_be_turned_off_per_state() {
    let off = PartialSlot { party_size: Some(false), ..PartialSlot::default() };
    let c = cfg(StateId::InMatch, off);
    assert_eq!(detailed(&facts(party_of(3)), &c).party, None);
}

#[test]
fn preview_shows_a_party_except_for_the_solo_variant() {
    let shown = crate::preview(&Config::default(), StateId::Hideout, Some(VariantId::Party), None, None).unwrap();
    assert_eq!(shown.party, Some((3, crate::PARTY_MAX)));
    let solo = crate::preview(&Config::default(), StateId::Hideout, Some(VariantId::Solo), None, None).unwrap();
    assert_eq!(solo.party, None);
}

#[test]
fn hero_presence_renders_the_heroes_own_hideout_line() {
    let c = cfg(StateId::Hideout, slot(Some("{heroPresence}"), None));
    let f = facts(|l| {
        l.context = Context::Hideout;
        l.hero_presence = Some("Mixing Drinks in the Hideout".into());
    });
    assert_eq!(detailed(&f, &c).details.as_deref(), Some("Mixing Drinks in the Hideout"));
    let none = facts(|l| l.context = Context::Hideout);
    assert_eq!(detailed(&none, &c).details, None);
}

#[test]
fn hero_presence_is_withheld_while_spectating() {
    let c = cfg(StateId::PostGame, slot(Some("{heroPresence}"), Some("x")));
    let f = facts(|l| {
        l.hero_presence = Some("Mixing Drinks in the Hideout".into());
        l.perspective = Perspective::Spectating;
        l.phase = Some(Phase::PostGame);
    });
    assert_eq!(detailed(&f, &c).details, None);
}

#[test]
fn preview_renders_the_sample_hideout_line() {
    use crate::PreviewSample;
    let sample = PreviewSample { hero_presence: Some("Plotting in the Hideout".into()), ..PreviewSample::default() };
    let c = cfg(StateId::Hideout, slot(Some("{heroPresence}"), None));
    let p = crate::preview_with(&c, StateId::Hideout, None, None, &sample).unwrap();
    assert_eq!(p.details.as_deref(), Some("Plotting in the Hideout"));
}

fn brawl(round: Option<u32>, amber: Option<u32>, sapphire: Option<u32>) -> impl FnOnce(&mut LiveFacts) {
    move |l| {
        l.game_mode = Some(GameMode::StreetBrawl);
        l.street_brawl = Some(StreetBrawlFacts { round, amber, sapphire });
    }
}

#[test]
fn street_brawl_round_and_scores_render() {
    let c = cfg(StateId::StreetBrawlRound, slot(Some("Round {round}"), Some("{scoreAmber}:{scoreSapphire}")));
    let p = detailed(&facts(brawl(Some(3), Some(2), Some(1))), &c);
    assert_eq!((p.details.as_deref(), p.state.as_deref()), (Some("Round 3"), Some("2:1")));
}

#[test]
fn a_zero_score_still_renders() {
    let c = cfg(StateId::StreetBrawlRound, slot(Some("{scoreAmber}-{scoreSapphire}"), None));
    let p = detailed(&facts(brawl(Some(1), Some(0), Some(0))), &c);
    assert_eq!(p.details.as_deref(), Some("0-0"));
}

#[test]
fn street_brawl_numbers_are_empty_without_facts_and_while_spectating() {
    let c = cfg(StateId::StreetBrawlRound, slot(Some("Round {round}"), None));
    assert_eq!(detailed(&facts(|l| l.game_mode = Some(GameMode::StreetBrawl)), &c).details.as_deref(), Some("Round"));
    let spectated = facts(|l| {
        brawl(Some(3), Some(2), Some(1))(l);
        l.perspective = Perspective::Spectating;
    });
    let spectating = cfg(StateId::Spectating, slot(Some("Round {round}"), None));
    assert_eq!(detailed(&spectated, &spectating).details.as_deref(), Some("Round"));
}

#[test]
fn the_built_in_round_line_reads_the_round_and_drops_without_one() {
    let with = detailed(&facts(brawl(Some(3), None, None)), &Config::default());
    assert_eq!(with.details.as_deref(), Some("Playing Street Brawl"));
    assert_eq!(with.state.as_deref(), Some("Round 3"));
    let without = detailed(&facts(brawl(None, None, None)), &Config::default());
    assert_eq!(without.state, None);
}

#[test]
fn preview_has_a_sample_round_and_score() {
    let c = cfg(StateId::StreetBrawlRound, slot(Some("{round} {scoreAmber}:{scoreSapphire}"), None));
    let p = crate::preview(&c, StateId::StreetBrawlRound, None, None, None).unwrap();
    assert_eq!(p.details.as_deref(), Some("3 2:1"));
}

#[test]
fn a_street_brawl_party_holds_four() {
    let in_brawl = facts(|l| {
        party_of(3)(l);
        l.game_mode = Some(GameMode::StreetBrawl);
    });
    assert_eq!(detailed(&in_brawl, &Config::default()).party, Some((3, 4)));
    let c = cfg(StateId::StreetBrawlRound, slot(Some("{partySize}/{partyMax}"), None));
    assert_eq!(detailed(&in_brawl, &c).details.as_deref(), Some("3/4"));
}

#[test]
fn a_queue_for_street_brawl_shows_four_and_a_normal_hideout_party_six() {
    let queue = |game_mode| {
        facts(|l| {
            l.party = Some(PartyFacts { size: 2, queueing: true, game_mode, ..PartyFacts::default() });
        })
    };
    let finding = |g| crate::map::map_with(PresenceLevel::Detailed, &hideout_of(queue(g)), &Config::default());
    assert_eq!(finding(Some(GameMode::StreetBrawl)).unwrap().party, Some((2, 4)));
    assert_eq!(finding(Some(GameMode::Normal)).unwrap().party, Some((2, 6)));
    let idle = facts(|l| {
        l.context = Context::Hideout;
        party_of(2)(l);
        l.game_mode = Some(GameMode::StreetBrawl);
    });
    assert_eq!(detailed(&idle, &Config::default()).party, Some((2, 6)));
}

fn hideout_of(mut f: GameFacts) -> GameFacts {
    if let Some(l) = f.live.as_mut() {
        l.context = Context::Hideout;
        l.phase = None;
    }
    f
}

#[test]
fn preview_party_max_follows_the_street_brawl_states() {
    let round = crate::preview(&Config::default(), StateId::StreetBrawlRound, None, None, None).unwrap();
    assert_eq!(round.party, Some((3, 4)));
    let queue =
        crate::preview(&Config::default(), StateId::FindingMatch, Some(VariantId::StreetBrawl), None, None).unwrap();
    assert_eq!(queue.party, Some((3, 4)));
    let ranked = crate::preview(&Config::default(), StateId::InMatch, Some(VariantId::Ranked), None, None).unwrap();
    assert_eq!(ranked.party, Some((3, crate::PARTY_MAX)));
}
