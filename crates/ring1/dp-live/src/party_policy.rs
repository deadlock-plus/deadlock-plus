use crate::{Context, LiveFacts};

/// Whether the facts describe a running match, where the party is fixed until the match ends.
pub fn in_match(facts: &LiveFacts) -> bool {
    match facts.context {
        Context::Match => true,
        // The Hideout reports a game phase too, but it is never a match.
        Context::Hideout => false,
        Context::Other => facts.phase.is_some(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Phase;

    #[test]
    fn a_match_is_a_match_context_or_any_phase() {
        assert!(!in_match(&LiveFacts::default()));
        assert!(in_match(&LiveFacts { context: Context::Match, ..LiveFacts::default() }));
        assert!(in_match(&LiveFacts { phase: Some(Phase::PostGame), ..LiveFacts::default() }));
        assert!(!in_match(&LiveFacts { context: Context::Hideout, ..LiveFacts::default() }));
    }

    #[test]
    fn the_hideout_is_never_a_match_even_though_it_reports_a_phase() {
        let hideout = LiveFacts { context: Context::Hideout, phase: Some(Phase::InProgress), ..LiveFacts::default() };
        assert!(!in_match(&hideout));
    }
}
