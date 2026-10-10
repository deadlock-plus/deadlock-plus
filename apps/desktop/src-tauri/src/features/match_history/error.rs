use crate::features::error::error_codes;

error_codes! {
    pub enum MatchHistoryError in "match_history" {
        TooLarge = "too_large",
        NotJson = "not_json",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::error::assert_catalogued;

    #[test]
    fn every_match_history_code_is_in_the_english_catalog() {
        assert_catalogued::<MatchHistoryError>();
    }
}
