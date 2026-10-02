use std::collections::HashSet;

/// Newest first, without duplicates or ids already handled, capped at `take`.
pub fn fresh_newest_first(ids: Vec<u64>, processed: &HashSet<u64>, take: usize) -> Vec<u64> {
    let mut ids: Vec<u64> = ids.into_iter().filter(|id| !processed.contains(id)).collect();
    ids.sort_unstable_by(|a, b| b.cmp(a));
    ids.dedup();
    ids.truncate(take);
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_newest_first_and_drops_duplicates() {
        assert_eq!(fresh_newest_first(vec![3, 9, 3, 5], &HashSet::new(), 10), vec![9, 5, 3]);
    }

    #[test]
    fn skips_processed_ids() {
        let processed = HashSet::from([9]);
        assert_eq!(fresh_newest_first(vec![3, 9, 5], &processed, 10), vec![5, 3]);
    }

    #[test]
    fn caps_after_filtering() {
        let processed = HashSet::from([9]);
        assert_eq!(fresh_newest_first(vec![9, 8, 7, 6], &processed, 2), vec![8, 7]);
    }

    #[test]
    fn zero_take_is_empty() {
        assert!(fresh_newest_first(vec![1], &HashSet::new(), 0).is_empty());
    }
}
