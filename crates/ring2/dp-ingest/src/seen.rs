use std::collections::HashMap;

use crate::salts::Salts;

#[derive(Default)]
pub struct Seen(HashMap<u64, (bool, bool)>);

impl Seen {
    pub fn is_new(&self, s: &Salts) -> bool {
        let (meta, replay) = self.0.get(&s.match_id).copied().unwrap_or_default();
        (s.metadata_salt.is_some() && !meta) || (s.replay_salt.is_some() && !replay)
    }

    pub fn mark(&mut self, s: &Salts) {
        if self.0.len() > 10_000 {
            self.0.clear();
        }
        let entry = self.0.entry(s.match_id).or_default();
        entry.0 |= s.metadata_salt.is_some();
        entry.1 |= s.replay_salt.is_some();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn salts(id: u64, meta: bool) -> Salts {
        Salts {
            match_id: id,
            cluster_id: None,
            metadata_salt: meta.then_some(1),
            replay_salt: (!meta).then_some(1),
            username: None,
        }
    }

    #[test]
    fn metadata_and_replay_salts_are_tracked_separately() {
        let mut seen = Seen::default();
        assert!(seen.is_new(&salts(5, true)));
        seen.mark(&salts(5, true));
        assert!(!seen.is_new(&salts(5, true)));
        assert!(seen.is_new(&salts(5, false)));
    }
}
