use valveprotos::deadlock::CMsgMatchMetaDataContents;

use crate::{Outcome, PostGameMatch};

const OUTCOME_WIN: i32 = 1;
const OUTCOME_LOSS: i32 = 2;

/// `rank_delta` stays `None`: the metadata's `desired_progress_change` is not confirmed to be
/// the same quantity as the API's `ranked_delta`.
pub fn to_match(meta: &CMsgMatchMetaDataContents, account_id: u32) -> Option<PostGameMatch> {
    let info = meta.match_info.as_ref()?;
    let me = info.players.iter().find(|p| p.account_id == Some(account_id))?;
    let rank = me.player_rank_data.as_ref();

    let outcome = match me.player_match_outcome {
        Some(OUTCOME_WIN) => Outcome::Win,
        Some(OUTCOME_LOSS) => Outcome::Loss,
        Some(0) | None => match (info.winning_team, me.team) {
            (Some(winner), Some(team)) if winner == team => Outcome::Win,
            (Some(_), Some(_)) => Outcome::Loss,
            _ => Outcome::Unscored,
        },
        Some(_) => Outcome::Unscored,
    };

    Some(PostGameMatch {
        match_id: info.match_id?,
        hero_id: me.hero_id?,
        start_time: u64::from(info.start_time?),
        match_mode: info.match_mode.and_then(|m| u32::try_from(m).ok()).unwrap_or(0),
        game_mode: info.game_mode.and_then(|m| u32::try_from(m).ok()).unwrap_or(0),
        outcome,
        kills: me.kills.unwrap_or(0),
        deaths: me.deaths.unwrap_or(0),
        assists: me.assists.unwrap_or(0),
        net_worth: me.net_worth.unwrap_or(0),
        duration_s: info.duration_s.unwrap_or(0),
        rank_badge: rank.and_then(|r| r.initial_display_rank).unwrap_or(0),
        rank_delta: None,
        calibration: rank.and_then(|r| r.initial_calibration_games).unwrap_or(0) > 0,
        demotion_protected: rank.and_then(|r| r.consumed_demotion_protection).unwrap_or(false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Outcome;
    use valveprotos::deadlock::c_msg_match_meta_data_contents::{MatchInfo, Players};
    use valveprotos::deadlock::CMsgMatchPlayerRankData;

    const ME: u32 = 395693146;

    fn me() -> Players {
        Players {
            account_id: Some(ME),
            team: Some(1),
            hero_id: Some(65),
            kills: Some(6),
            deaths: Some(8),
            assists: Some(20),
            net_worth: Some(31_000),
            player_match_outcome: Some(1),
            player_rank_data: Some(CMsgMatchPlayerRankData {
                initial_display_rank: Some(63),
                initial_calibration_games: Some(2),
                consumed_demotion_protection: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn other() -> Players {
        Players { account_id: Some(251353065), team: Some(0), hero_id: Some(19), ..Default::default() }
    }

    fn meta(players: Vec<Players>) -> CMsgMatchMetaDataContents {
        CMsgMatchMetaDataContents {
            match_info: Some(MatchInfo {
                match_id: Some(106837748),
                start_time: Some(1_789_947_499),
                duration_s: Some(2070),
                winning_team: Some(1),
                game_mode: Some(1),
                match_mode: Some(4),
                players,
                ..Default::default()
            }),
        }
    }

    #[test]
    fn maps_the_account_s_own_row() {
        let m = to_match(&meta(vec![other(), me()]), ME).unwrap();
        assert_eq!(
            m,
            PostGameMatch {
                match_id: 106837748,
                hero_id: 65,
                start_time: 1_789_947_499,
                match_mode: 4,
                game_mode: 1,
                outcome: Outcome::Win,
                kills: 6,
                deaths: 8,
                assists: 20,
                net_worth: 31_000,
                duration_s: 2070,
                rank_badge: 63,
                rank_delta: None,
                calibration: true,
                demotion_protected: true,
            }
        );
    }

    #[test]
    fn an_absent_account_gives_nothing() {
        assert_eq!(to_match(&meta(vec![other()]), ME), None);
        assert_eq!(to_match(&CMsgMatchMetaDataContents::default(), ME), None);
    }

    #[test]
    fn outcome_falls_back_to_the_winning_team_when_the_code_is_invalid() {
        let mut p = me();
        p.player_match_outcome = Some(0);
        assert_eq!(to_match(&meta(vec![p.clone()]), ME).unwrap().outcome, Outcome::Win);
        p.team = Some(0);
        assert_eq!(to_match(&meta(vec![p]), ME).unwrap().outcome, Outcome::Loss);
    }

    #[test]
    fn penalised_and_unscored_rows_carry_no_result() {
        let mut p = me();
        p.player_match_outcome = Some(5);
        assert_eq!(to_match(&meta(vec![p]), ME).unwrap().outcome, Outcome::Unscored);
    }
}
