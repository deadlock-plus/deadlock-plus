use crate::map::{Context, GameMode, LiveFacts, MatchMode, Perspective, Phase};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StateId {
    Playing,
    MainMenu,
    Hideout,
    HeroSelect,
    FindingMatch,
    MatchFound,
    PreGame,
    InMatch,
    StreetBrawlRound,
    Paused,
    Spectating,
    PostGame,
    PrivateLobby,
    Practice,
}

impl StateId {
    /// Whether per-hero overrides may apply. Spectating and private lobbies are excluded so a hero override can never
    /// key off an observed player or a custom game.
    pub(crate) fn has_hero_scope(self) -> bool {
        !matches!(self, Self::Playing | Self::MainMenu | Self::Spectating | Self::PrivateLobby)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VariantId {
    Solo,
    Party,
    Unranked,
    Ranked,
    HeroLabs,
    Bots,
    Tutorial,
    Normal,
    StreetBrawl,
    Sandbox,
    ExploreNyc,
    Won,
    Lost,
    Unscored,
}

pub fn classify(live: &LiveFacts) -> (StateId, Option<VariantId>) {
    match live.context {
        Context::Other => return (StateId::MainMenu, None),
        Context::Hideout => return (StateId::Hideout, None),
        Context::Match => {}
    }
    if live.match_mode == Some(MatchMode::PrivateLobby) {
        return (StateId::PrivateLobby, None);
    }
    let spectating = live.perspective == Perspective::Spectating;
    match live.phase {
        Some(Phase::HeroSelection) => (StateId::HeroSelect, mode_variant(live)),
        Some(Phase::MatchIntro | Phase::Loading) => (StateId::MatchFound, mode_variant(live)),
        Some(Phase::PreGame) => (StateId::PreGame, None),
        Some(Phase::PostGame) => {
            let result = match (spectating, live.local_won) {
                (false, Some(true)) => VariantId::Won,
                (false, Some(false)) => VariantId::Lost,
                _ => VariantId::Unscored,
            };
            (StateId::PostGame, Some(result))
        }
        Some(Phase::InProgress) | None if spectating => (StateId::Spectating, None),
        Some(Phase::InProgress) | None if live.paused => (StateId::Paused, mode_variant(live)),
        Some(Phase::InProgress) | None => (StateId::InMatch, mode_variant(live)),
    }
}

fn mode_variant(live: &LiveFacts) -> Option<VariantId> {
    match live.game_mode {
        Some(GameMode::StreetBrawl) => return Some(VariantId::StreetBrawl),
        Some(GameMode::Sandbox) => return Some(VariantId::Sandbox),
        Some(GameMode::ExploreNyc) => return Some(VariantId::ExploreNyc),
        _ => {}
    }
    match live.match_mode {
        Some(MatchMode::Unranked) => Some(VariantId::Unranked),
        Some(MatchMode::Ranked) => Some(VariantId::Ranked),
        Some(MatchMode::CoopBot) => Some(VariantId::Bots),
        Some(MatchMode::HeroLabs) => Some(VariantId::HeroLabs),
        Some(MatchMode::Tutorial) => Some(VariantId::Tutorial),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn live(f: impl FnOnce(&mut LiveFacts)) -> LiveFacts {
        let mut l = LiveFacts { context: Context::Match, phase: Some(Phase::InProgress), ..LiveFacts::default() };
        f(&mut l);
        l
    }

    fn id(l: &LiveFacts) -> (StateId, Option<VariantId>) {
        classify(l)
    }

    #[test]
    fn other_context_is_main_menu() {
        assert_eq!(id(&live(|l| l.context = Context::Other)), (StateId::MainMenu, None));
    }

    #[test]
    fn hideout_has_no_variant_while_party_is_unknown() {
        assert_eq!(id(&live(|l| l.context = Context::Hideout)), (StateId::Hideout, None));
    }

    #[test]
    fn hero_selection_varies_by_match_mode() {
        let l = live(|l| {
            l.phase = Some(Phase::HeroSelection);
            l.match_mode = Some(MatchMode::Ranked);
        });
        assert_eq!(id(&l), (StateId::HeroSelect, Some(VariantId::Ranked)));
        assert_eq!(id(&live(|l| l.phase = Some(Phase::HeroSelection))), (StateId::HeroSelect, None));
    }

    #[test]
    fn intro_and_loading_are_match_found() {
        for phase in [Phase::MatchIntro, Phase::Loading] {
            let l = live(|l| {
                l.phase = Some(phase);
                l.game_mode = Some(GameMode::StreetBrawl);
            });
            assert_eq!(id(&l), (StateId::MatchFound, Some(VariantId::StreetBrawl)));
        }
    }

    #[test]
    fn pre_game_is_its_own_state() {
        assert_eq!(id(&live(|l| l.phase = Some(Phase::PreGame))), (StateId::PreGame, None));
    }

    #[test]
    fn in_match_variant_prefers_game_mode_then_match_mode() {
        let cases = [
            (Some(MatchMode::Unranked), Some(GameMode::StreetBrawl), Some(VariantId::StreetBrawl)),
            (Some(MatchMode::Unranked), Some(GameMode::Normal), Some(VariantId::Unranked)),
            (Some(MatchMode::Ranked), None, Some(VariantId::Ranked)),
            (Some(MatchMode::CoopBot), None, Some(VariantId::Bots)),
            (Some(MatchMode::HeroLabs), None, Some(VariantId::HeroLabs)),
            (Some(MatchMode::Tutorial), None, Some(VariantId::Tutorial)),
            (None, Some(GameMode::Sandbox), Some(VariantId::Sandbox)),
            (None, Some(GameMode::ExploreNyc), Some(VariantId::ExploreNyc)),
            (Some(MatchMode::Other), Some(GameMode::Other), None),
            (None, None, None),
        ];
        for (mm, gm, want) in cases {
            let l = live(|l| {
                l.match_mode = mm;
                l.game_mode = gm;
            });
            assert_eq!(id(&l), (StateId::InMatch, want), "{mm:?} {gm:?}");
        }
    }

    #[test]
    fn in_progress_without_phase_is_in_match() {
        assert_eq!(id(&live(|l| l.phase = None)).0, StateId::InMatch);
    }

    #[test]
    fn paused_in_match_is_paused_and_keeps_the_mode_variant() {
        let l = live(|l| {
            l.paused = true;
            l.match_mode = Some(MatchMode::Ranked);
        });
        assert_eq!(id(&l), (StateId::Paused, Some(VariantId::Ranked)));
    }

    #[test]
    fn pause_outside_the_match_proper_is_ignored() {
        let l = live(|l| {
            l.paused = true;
            l.phase = Some(Phase::PreGame);
        });
        assert_eq!(id(&l).0, StateId::PreGame);
    }

    #[test]
    fn spectating_in_match_is_spectating_even_when_paused() {
        let l = live(|l| {
            l.perspective = Perspective::Spectating;
            l.paused = true;
            l.match_mode = Some(MatchMode::Ranked);
        });
        assert_eq!(id(&l), (StateId::Spectating, None));
    }

    #[test]
    fn spectating_post_game_is_unscored() {
        let l = live(|l| {
            l.perspective = Perspective::Spectating;
            l.phase = Some(Phase::PostGame);
            l.local_won = Some(true);
        });
        assert_eq!(id(&l), (StateId::PostGame, Some(VariantId::Unscored)));
    }

    #[test]
    fn post_game_won_lost_unscored() {
        for (won, want) in [(Some(true), VariantId::Won), (Some(false), VariantId::Lost), (None, VariantId::Unscored)] {
            let l = live(|l| {
                l.phase = Some(Phase::PostGame);
                l.local_won = won;
            });
            assert_eq!(id(&l), (StateId::PostGame, Some(want)));
        }
    }

    #[test]
    fn private_lobby_wins_over_every_phase_and_perspective() {
        for phase in [Phase::HeroSelection, Phase::Loading, Phase::InProgress, Phase::PostGame] {
            for perspective in [Perspective::Playing, Perspective::Spectating] {
                let l = live(|l| {
                    l.phase = Some(phase);
                    l.perspective = perspective;
                    l.match_mode = Some(MatchMode::PrivateLobby);
                });
                assert_eq!(id(&l), (StateId::PrivateLobby, None));
            }
        }
    }

    #[test]
    fn private_lobby_mode_outside_a_match_context_is_ignored() {
        let l = live(|l| {
            l.context = Context::Hideout;
            l.match_mode = Some(MatchMode::PrivateLobby);
        });
        assert_eq!(id(&l).0, StateId::Hideout);
    }

    #[test]
    fn finding_match_street_brawl_round_and_practice_are_never_produced() {
        let mut seen = std::collections::HashSet::new();
        for context in [Context::Other, Context::Hideout, Context::Match] {
            for phase in [
                None,
                Some(Phase::HeroSelection),
                Some(Phase::MatchIntro),
                Some(Phase::Loading),
                Some(Phase::PreGame),
                Some(Phase::InProgress),
                Some(Phase::PostGame),
            ] {
                for perspective in [Perspective::Unknown, Perspective::Playing, Perspective::Spectating] {
                    for mm in [None, Some(MatchMode::PrivateLobby), Some(MatchMode::Tutorial), Some(MatchMode::CoopBot)]
                    {
                        for gm in [None, Some(GameMode::StreetBrawl), Some(GameMode::Sandbox)] {
                            for paused in [false, true] {
                                let l = LiveFacts {
                                    context,
                                    phase,
                                    perspective,
                                    match_mode: mm,
                                    game_mode: gm,
                                    paused,
                                    ..LiveFacts::default()
                                };
                                seen.insert(classify(&l).0);
                            }
                        }
                    }
                }
            }
        }
        for never in [StateId::FindingMatch, StateId::StreetBrawlRound, StateId::Practice, StateId::Playing] {
            assert!(!seen.contains(&never), "{never:?}");
        }
    }
}
