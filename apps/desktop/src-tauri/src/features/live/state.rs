//! Only the Windows reader feeds this; elsewhere the state stays `Unsupported` and the feeding code is unused.
#![cfg_attr(not(windows), allow(dead_code))]

use std::sync::Mutex;

use dp_live::{Context, LiveFacts, Phase};
use dp_sync::LockExt;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

const EVENT: &str = "live-snapshot";

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum LivePhase {
    Unsupported,
    GameClosed,
    Menus,
    Queuing,
    Pregame,
    InMatch,
    PostMatch,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct LiveState {
    pub phase: LivePhase,
    /// A match exists to show: pregame, in progress or finished.
    pub match_present: bool,
}

impl LiveState {
    pub fn of(phase: LivePhase) -> Self {
        let match_present = matches!(phase, LivePhase::Pregame | LivePhase::InMatch | LivePhase::PostMatch);
        Self { phase, match_present }
    }
}

/// `None` means the game is running but no match or hideout is loaded.
pub fn derive(facts: Option<&LiveFacts>) -> LiveState {
    let Some(facts) = facts else { return LiveState::of(LivePhase::Menus) };
    let phase = match facts.context {
        Context::Match => match facts.phase {
            Some(Phase::InProgress) => LivePhase::InMatch,
            Some(Phase::PostGame) => LivePhase::PostMatch,
            _ => LivePhase::Pregame,
        },
        // The loading screen and hero selection run before the reader sees a match.
        Context::Other
            if matches!(
                facts.phase,
                Some(Phase::HeroSelection | Phase::MatchIntro | Phase::Loading | Phase::PreGame)
            ) =>
        {
            LivePhase::Pregame
        }
        _ if facts.party.is_some_and(|p| p.queueing) => LivePhase::Queuing,
        _ => LivePhase::Menus,
    };
    LiveState::of(phase)
}

struct Inner {
    state: Option<LiveState>,
}

pub struct LiveService {
    inner: Mutex<Inner>,
}

impl Default for LiveService {
    fn default() -> Self {
        Self { inner: Mutex::new(Inner { state: None }) }
    }
}

impl LiveService {
    pub fn current(&self) -> LiveState {
        current(&self.inner.lock_or_recover())
    }

    pub fn report(&self, app: &AppHandle, state: LiveState) {
        let changed = report(&mut self.inner.lock_or_recover(), state);
        emit(app, changed);
    }
}

fn idle() -> LiveState {
    LiveState::of(if cfg!(windows) { LivePhase::GameClosed } else { LivePhase::Unsupported })
}

fn current(inner: &Inner) -> LiveState {
    inner.state.unwrap_or_else(idle)
}

fn report(inner: &mut Inner, state: LiveState) -> Option<LiveState> {
    if inner.state == Some(state) {
        return None;
    }
    inner.state = Some(state);
    Some(state)
}

fn emit(app: &AppHandle, state: Option<LiveState>) {
    if let Some(state) = state {
        if let Err(e) = app.emit(EVENT, state) {
            log::warn!("could not emit {EVENT}: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dp_live::PartyFacts;

    fn facts(context: Context, phase: Option<Phase>) -> LiveFacts {
        LiveFacts { context, phase, ..Default::default() }
    }

    fn queueing() -> Option<PartyFacts> {
        Some(PartyFacts { size: 1, queueing: true, queued_secs: Some(5), ..Default::default() })
    }

    fn phase_of(f: &LiveFacts) -> LivePhase {
        derive(Some(f)).phase
    }

    #[test]
    fn no_snapshot_means_menus() {
        assert_eq!(derive(None), LiveState { phase: LivePhase::Menus, match_present: false });
    }

    #[test]
    fn hideout_is_menus_and_queueing_party_is_queuing() {
        let mut f = facts(Context::Hideout, None);
        assert_eq!(phase_of(&f), LivePhase::Menus);
        f.party = Some(PartyFacts { size: 2, queueing: false, ..Default::default() });
        assert_eq!(phase_of(&f), LivePhase::Menus);
        f.party = queueing();
        assert_eq!(derive(Some(&f)), LiveState { phase: LivePhase::Queuing, match_present: false });
        assert_eq!(phase_of(&facts(Context::Other, None)), LivePhase::Menus);
    }

    #[test]
    fn match_phases_map_and_flag_a_match() {
        for (phase, want) in [
            (Some(Phase::HeroSelection), LivePhase::Pregame),
            (Some(Phase::MatchIntro), LivePhase::Pregame),
            (Some(Phase::Loading), LivePhase::Pregame),
            (Some(Phase::PreGame), LivePhase::Pregame),
            (None, LivePhase::Pregame),
            (Some(Phase::InProgress), LivePhase::InMatch),
            (Some(Phase::PostGame), LivePhase::PostMatch),
        ] {
            assert_eq!(derive(Some(&facts(Context::Match, phase))), LiveState { phase: want, match_present: true });
        }
    }

    #[test]
    fn a_match_wins_over_a_stale_queue_flag() {
        let mut f = facts(Context::Match, Some(Phase::InProgress));
        f.party = queueing();
        assert_eq!(phase_of(&f), LivePhase::InMatch);
    }

    #[test]
    fn loading_into_a_match_from_other_context_is_pregame_but_other_phases_are_menus() {
        assert_eq!(phase_of(&facts(Context::Other, Some(Phase::Loading))), LivePhase::Pregame);
        assert_eq!(phase_of(&facts(Context::Other, Some(Phase::HeroSelection))), LivePhase::Pregame);
        assert_eq!(phase_of(&facts(Context::Other, Some(Phase::InProgress))), LivePhase::Menus);
        assert_eq!(phase_of(&facts(Context::Hideout, Some(Phase::Loading))), LivePhase::Menus);
    }

    #[test]
    fn match_present_only_for_match_phases() {
        for p in [LivePhase::Unsupported, LivePhase::GameClosed, LivePhase::Menus, LivePhase::Queuing] {
            assert!(!LiveState::of(p).match_present, "{p:?}");
        }
        for p in [LivePhase::Pregame, LivePhase::InMatch, LivePhase::PostMatch] {
            assert!(LiveState::of(p).match_present, "{p:?}");
        }
    }

    #[test]
    fn reports_emit_only_on_change() {
        let mut i = Inner { state: None };
        let menus = LiveState::of(LivePhase::Menus);
        assert_eq!(report(&mut i, menus), Some(menus));
        assert_eq!(report(&mut i, menus), None);
        let q = LiveState::of(LivePhase::Queuing);
        assert_eq!(report(&mut i, q), Some(q));
    }

    #[test]
    fn a_fresh_service_state_is_game_closed_or_unsupported() {
        let want = if cfg!(windows) { LivePhase::GameClosed } else { LivePhase::Unsupported };
        assert_eq!(LiveService::default().current().phase, want);
    }
}
