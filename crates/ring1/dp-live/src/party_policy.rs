use std::time::Duration;

use crate::{Context, LiveFacts, PartyFacts};

/// Finding the party can cost a search of the whole game heap, so it is read at most this often.
pub const PARTY_READ_INTERVAL: Duration = Duration::from_secs(10);

/// Whether the facts describe a running match, where the party is fixed until the match ends.
pub fn in_match(facts: &LiveFacts) -> bool {
    facts.context == Context::Match || facts.phase.is_some()
}

/// A party is read when none is known yet, or, outside a match, when the last read is old enough. Inside a match
/// the last known party is kept.
pub fn should_read_party(in_match: bool, since_last: Option<Duration>) -> bool {
    match since_last {
        None => true,
        Some(_) if in_match => false,
        Some(elapsed) => elapsed >= PARTY_READ_INTERVAL,
    }
}

/// The last read party carried forward by `elapsed`, so queue time keeps counting between reads.
pub fn aged(party: PartyFacts, elapsed: Duration) -> PartyFacts {
    PartyFacts { queued_secs: party.queued_secs.map(|s| s + elapsed.as_secs()), ..party }
}

/// The last read party as it stands during a match: the queue is over, the roster is what it was.
pub fn matched(party: PartyFacts) -> PartyFacts {
    PartyFacts { queueing: false, queued_secs: None, match_mode: None, game_mode: None, ..party }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GameMode, MatchMode, Phase};

    const SECS: fn(u64) -> Option<Duration> = |s| Some(Duration::from_secs(s));

    #[test]
    fn reads_when_nothing_is_known() {
        assert!(should_read_party(false, None));
        assert!(should_read_party(true, None));
    }

    #[test]
    fn menus_wait_out_the_interval() {
        assert!(!should_read_party(false, SECS(1)));
        assert!(!should_read_party(false, SECS(9)));
        assert!(should_read_party(false, SECS(10)));
        assert!(should_read_party(false, SECS(60)));
    }

    #[test]
    fn a_match_keeps_the_known_party() {
        assert!(!should_read_party(true, SECS(1)));
        assert!(!should_read_party(true, SECS(3600)));
    }

    #[test]
    fn a_match_is_a_match_context_or_any_phase() {
        assert!(!in_match(&LiveFacts::default()));
        assert!(in_match(&LiveFacts { context: Context::Match, ..LiveFacts::default() }));
        assert!(in_match(&LiveFacts { phase: Some(Phase::PostGame), ..LiveFacts::default() }));
        assert!(!in_match(&LiveFacts { context: Context::Hideout, ..LiveFacts::default() }));
    }

    #[test]
    fn queue_time_keeps_counting() {
        let p = PartyFacts { size: 2, queueing: true, queued_secs: Some(5), ..PartyFacts::default() };
        assert_eq!(aged(p, Duration::from_millis(3900)).queued_secs, Some(8));
        assert_eq!(aged(PartyFacts::default(), Duration::from_secs(9)).queued_secs, None);
    }

    #[test]
    fn a_match_ends_the_queue_but_keeps_the_roster() {
        let p = PartyFacts {
            size: 3,
            queueing: true,
            queued_secs: Some(40),
            match_mode: Some(MatchMode::Ranked),
            game_mode: Some(GameMode::Normal),
        };
        assert_eq!(matched(p), PartyFacts { size: 3, ..PartyFacts::default() });
    }
}
