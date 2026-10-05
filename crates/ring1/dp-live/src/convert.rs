use deadlock_reader::snapshot::{Context as RContext, LiveSnapshot, Perspective as RPerspective};
use deadlock_reader::{GameMode as RGameMode, GameState, MatchMode as RMatchMode};

use crate::{Context, GameMode, LiveFacts, MatchMode, Perspective, Phase};

pub fn from_snapshot(snap: &LiveSnapshot) -> LiveFacts {
    let perspective = match snap.perspective {
        RPerspective::Unknown => Perspective::Unknown,
        RPerspective::Playing => Perspective::Playing,
        RPerspective::Spectating => Perspective::Spectating,
    };
    let phase = snap.game_state.and_then(phase);
    let me = match perspective {
        Perspective::Playing => snap.local_player(),
        Perspective::Spectating => snap.observed_player(),
        Perspective::Unknown => None,
    };
    let local_won = match (phase, perspective, snap.winning_team, me.and_then(|p| p.team)) {
        (Some(Phase::PostGame), Perspective::Playing, Some(winner), Some(mine)) => Some(winner == mine),
        _ => None,
    };
    LiveFacts {
        context: match snap.context {
            RContext::Other => Context::Other,
            // Hideout entities stay loaded while spectating a match from the hideout, so the
            // reader's context says Hideout. A match id is only set inside a real match.
            RContext::Hideout if snap.match_id.is_some() => Context::Match,
            RContext::Hideout => Context::Hideout,
            RContext::Match => Context::Match,
        },
        phase,
        perspective,
        match_mode: snap.match_mode.and_then(match_mode),
        game_mode: snap.game_mode.and_then(game_mode),
        hero_id: me.and_then(|p| p.hero_id).filter(|h| h.is_some()).map(|h| h.get()),
        match_time_secs: snap.timers.match_time,
        paused: snap.paused.unwrap_or(false),
        drift: !snap.drift.is_empty(),
        local_won,
        kills: me.and_then(|p| p.kills),
        deaths: me.and_then(|p| p.deaths),
        assists: me.and_then(|p| p.assists),
        souls: me.and_then(|p| p.net_worth),
        match_id: snap.match_id,
    }
}

fn phase(state: GameState) -> Option<Phase> {
    match state {
        GameState::HeroSelection => Some(Phase::HeroSelection),
        GameState::MatchIntro => Some(Phase::MatchIntro),
        GameState::WaitForMapToLoad => Some(Phase::Loading),
        GameState::PreGameWait => Some(Phase::PreGame),
        GameState::GameInProgress => Some(Phase::InProgress),
        s if s.is_over() => Some(Phase::PostGame),
        _ => None,
    }
}

fn match_mode(mode: RMatchMode) -> Option<MatchMode> {
    Some(match mode {
        RMatchMode::Invalid => return None,
        RMatchMode::Unranked => MatchMode::Unranked,
        RMatchMode::Ranked => MatchMode::Ranked,
        RMatchMode::PrivateLobby => MatchMode::PrivateLobby,
        RMatchMode::CoopBot => MatchMode::CoopBot,
        RMatchMode::HeroLabs => MatchMode::HeroLabs,
        RMatchMode::Tutorial => MatchMode::Tutorial,
        _ => MatchMode::Other,
    })
}

fn game_mode(mode: RGameMode) -> Option<GameMode> {
    Some(match mode {
        RGameMode::Invalid => return None,
        RGameMode::Normal => GameMode::Normal,
        RGameMode::StreetBrawl => GameMode::StreetBrawl,
        RGameMode::Sandbox => GameMode::Sandbox,
        RGameMode::ExploreNyc => GameMode::ExploreNyc,
        _ => GameMode::Other,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, GameMode, MatchMode, Perspective, Phase};
    use deadlock_reader::snapshot::PlayerRow;
    use deadlock_reader::timers::Timers;
    use deadlock_reader::{Drift, HeroId, Team};

    fn row(team: Team, hero: u32, local: bool, observed: bool) -> PlayerRow {
        PlayerRow {
            team: Some(team),
            is_spectator: team == Team::SPECTATOR,
            is_local: Some(local),
            is_observed: observed,
            hero_id: Some(HeroId(hero)),
            ..Default::default()
        }
    }

    fn playing() -> LiveSnapshot {
        LiveSnapshot {
            context: RContext::Match,
            perspective: RPerspective::Playing,
            game_state: Some(GameState::GameInProgress),
            players: vec![row(Team::SAPPHIRE, 9, false, false), row(Team::AMBER, 15, true, false)],
            ..Default::default()
        }
    }

    #[test]
    fn empty_snapshot_maps_to_defaults() {
        let f = from_snapshot(&LiveSnapshot::default());
        assert_eq!(f, LiveFacts::default());
        assert_eq!(f.context, Context::Other);
        assert_eq!(f.perspective, Perspective::Unknown);
    }

    #[test]
    fn context_and_perspective_map() {
        let mut s = playing();
        let f = from_snapshot(&s);
        assert_eq!((f.context, f.perspective), (Context::Match, Perspective::Playing));
        s.context = RContext::Hideout;
        s.perspective = RPerspective::Spectating;
        let f = from_snapshot(&s);
        assert_eq!((f.context, f.perspective), (Context::Hideout, Perspective::Spectating));
    }

    #[test]
    fn hideout_entities_with_a_match_id_still_mean_a_match() {
        let mut s = playing();
        s.context = RContext::Hideout;
        s.perspective = RPerspective::Spectating;
        s.match_id = Some(111_074_434);
        assert_eq!(from_snapshot(&s).context, Context::Match);
        s.match_id = None;
        assert_eq!(from_snapshot(&s).context, Context::Hideout);
    }

    #[test]
    fn game_states_map_to_phases() {
        for (state, want) in [
            (GameState::HeroSelection, Some(Phase::HeroSelection)),
            (GameState::MatchIntro, Some(Phase::MatchIntro)),
            (GameState::WaitForMapToLoad, Some(Phase::Loading)),
            (GameState::PreGameWait, Some(Phase::PreGame)),
            (GameState::GameInProgress, Some(Phase::InProgress)),
            (GameState::PostGame, Some(Phase::PostGame)),
            (GameState::PostGamePlayOfTheGame, Some(Phase::PostGame)),
            (GameState::Abandoned, Some(Phase::PostGame)),
            (GameState::End, Some(Phase::PostGame)),
            (GameState::Invalid, None),
            (GameState::Init, None),
            (GameState::WaitingForPlayersToJoin, None),
            (GameState::Unknown(99), None),
        ] {
            let s = LiveSnapshot { game_state: Some(state), ..Default::default() };
            assert_eq!(from_snapshot(&s).phase, want, "{state:?}");
        }
        assert_eq!(from_snapshot(&LiveSnapshot::default()).phase, None);
    }

    #[test]
    fn match_modes_map() {
        for (mode, want) in [
            (RMatchMode::Invalid, None),
            (RMatchMode::Unranked, Some(MatchMode::Unranked)),
            (RMatchMode::Ranked, Some(MatchMode::Ranked)),
            (RMatchMode::PrivateLobby, Some(MatchMode::PrivateLobby)),
            (RMatchMode::CoopBot, Some(MatchMode::CoopBot)),
            (RMatchMode::HeroLabs, Some(MatchMode::HeroLabs)),
            (RMatchMode::Tutorial, Some(MatchMode::Tutorial)),
            (RMatchMode::ServerTest, Some(MatchMode::Other)),
            (RMatchMode::NewPlayerPlacement, Some(MatchMode::Other)),
            (RMatchMode::Unknown(77), Some(MatchMode::Other)),
        ] {
            let s = LiveSnapshot { match_mode: Some(mode), ..Default::default() };
            assert_eq!(from_snapshot(&s).match_mode, want, "{mode:?}");
        }
    }

    #[test]
    fn game_modes_map() {
        for (mode, want) in [
            (RGameMode::Invalid, None),
            (RGameMode::Normal, Some(GameMode::Normal)),
            (RGameMode::StreetBrawl, Some(GameMode::StreetBrawl)),
            (RGameMode::Sandbox, Some(GameMode::Sandbox)),
            (RGameMode::ExploreNyc, Some(GameMode::ExploreNyc)),
            (RGameMode::OneVsOneTest, Some(GameMode::Other)),
            (RGameMode::Internal, Some(GameMode::Other)),
            (RGameMode::Unknown(42), Some(GameMode::Other)),
        ] {
            let s = LiveSnapshot { game_mode: Some(mode), ..Default::default() };
            assert_eq!(from_snapshot(&s).game_mode, want, "{mode:?}");
        }
    }

    #[test]
    fn hero_comes_from_the_local_row_when_playing() {
        assert_eq!(from_snapshot(&playing()).hero_id, Some(15));
    }

    #[test]
    fn hero_comes_from_the_observed_row_when_spectating() {
        let s = LiveSnapshot {
            perspective: RPerspective::Spectating,
            players: vec![
                row(Team::SPECTATOR, 0, true, false),
                row(Team::AMBER, 4, false, true),
                row(Team::SAPPHIRE, 9, false, false),
            ],
            ..Default::default()
        };
        assert_eq!(from_snapshot(&s).hero_id, Some(4));
    }

    #[test]
    fn spectating_never_reports_the_local_controller_hero() {
        let s = LiveSnapshot {
            perspective: RPerspective::Spectating,
            players: vec![row(Team::SPECTATOR, 7, true, false)],
            ..Default::default()
        };
        assert_eq!(from_snapshot(&s).hero_id, None);
    }

    #[test]
    fn an_unpicked_or_unidentified_hero_is_none() {
        let mut s = playing();
        s.players[1].hero_id = Some(HeroId::NONE);
        assert_eq!(from_snapshot(&s).hero_id, None);
        s.players[1].hero_id = None;
        assert_eq!(from_snapshot(&s).hero_id, None);
        s.players.clear();
        assert_eq!(from_snapshot(&s).hero_id, None);
        s.perspective = RPerspective::Unknown;
        assert_eq!(from_snapshot(&s).hero_id, None);
    }

    #[test]
    fn match_time_and_pause_map() {
        let s = LiveSnapshot {
            timers: Timers { match_time: Some(312.5), ..Default::default() },
            paused: Some(true),
            ..Default::default()
        };
        let f = from_snapshot(&s);
        assert_eq!(f.match_time_secs, Some(312.5));
        assert!(f.paused);
        let f = from_snapshot(&LiveSnapshot::default());
        assert_eq!(f.match_time_secs, None);
        assert!(!f.paused);
    }

    #[test]
    fn drift_is_set_only_when_the_list_is_non_empty() {
        let mut s = playing();
        assert!(!from_snapshot(&s).drift);
        s.drift.push(Drift::EnumMismatch {
            class: "C".into(),
            field: "f".into(),
            raw: 1,
            schema: "a".into(),
            baked: "b".into(),
        });
        assert!(from_snapshot(&s).drift);
    }

    fn finished(local: Team, winner: Option<Team>, state: GameState) -> LiveSnapshot {
        let other = if local == Team::AMBER { Team::SAPPHIRE } else { Team::AMBER };
        LiveSnapshot {
            perspective: RPerspective::Playing,
            game_state: Some(state),
            winning_team: winner,
            players: vec![row(other, 9, false, false), row(local, 15, true, false)],
            ..Default::default()
        }
    }

    #[test]
    fn local_won_compares_the_winner_with_the_local_team_after_the_match() {
        let won = from_snapshot(&finished(Team::AMBER, Some(Team::AMBER), GameState::PostGame));
        assert_eq!(won.local_won, Some(true));
        let lost = from_snapshot(&finished(Team::AMBER, Some(Team::SAPPHIRE), GameState::PostGame));
        assert_eq!(lost.local_won, Some(false));
        let sapphire = from_snapshot(&finished(Team::SAPPHIRE, Some(Team::SAPPHIRE), GameState::PostGame));
        assert_eq!(sapphire.local_won, Some(true));
    }

    #[test]
    fn local_won_is_none_before_post_game_or_without_a_winner() {
        let early = finished(Team::AMBER, Some(Team::AMBER), GameState::GameInProgress);
        assert_eq!(from_snapshot(&early).local_won, None);
        let undecided = finished(Team::AMBER, None, GameState::PostGame);
        assert_eq!(from_snapshot(&undecided).local_won, None);
    }

    #[test]
    fn local_won_is_none_when_spectating() {
        let mut s = finished(Team::AMBER, Some(Team::AMBER), GameState::PostGame);
        s.perspective = RPerspective::Spectating;
        assert_eq!(from_snapshot(&s).local_won, None);
    }

    #[test]
    fn score_comes_from_the_local_row() {
        let mut s = playing();
        s.players[1].kills = Some(12);
        s.players[1].deaths = Some(3);
        s.players[1].assists = Some(8);
        s.players[1].net_worth = Some(24_100);
        s.players[0].kills = Some(99);
        let f = from_snapshot(&s);
        assert_eq!((f.kills, f.deaths, f.assists, f.souls), (Some(12), Some(3), Some(8), Some(24_100)));
    }

    #[test]
    fn match_id_is_carried_through_in_a_match_and_from_a_hideout_spectate() {
        let mut s = playing();
        s.match_id = Some(111_074_434);
        assert_eq!(from_snapshot(&s).match_id, Some(111_074_434));
        s.context = RContext::Hideout;
        s.perspective = RPerspective::Spectating;
        assert_eq!(from_snapshot(&s).match_id, Some(111_074_434));
        s.match_id = None;
        assert_eq!(from_snapshot(&s).match_id, None);
    }

    #[test]
    fn match_id_keeps_the_full_u64_width() {
        let s = LiveSnapshot { match_id: Some(u64::from(u32::MAX) + 5), ..Default::default() };
        assert_eq!(from_snapshot(&s).match_id, Some(u64::from(u32::MAX) + 5));
    }

    #[test]
    fn score_is_none_without_a_player_row() {
        let f = from_snapshot(&LiveSnapshot::default());
        assert_eq!((f.kills, f.deaths, f.assists, f.souls), (None, None, None, None));
    }
}
