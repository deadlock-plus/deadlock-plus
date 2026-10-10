use dp_versioned::Migration;
use serde_json::Value;

const MATCH_HISTORY_KEY: &str = "matchHistory";

/// Migrations are per store: the schema version lives in each store's own file, and a store that
/// never had a migration must keep reading as version 1.
pub fn for_store(store: &str) -> &'static [Migration] {
    match store {
        "connection-settings" => &[drop_match_history],
        _ => &[],
    }
}

fn drop_match_history(mut data: Value) -> Value {
    if let Some(map) = data.as_object_mut() {
        map.remove(MATCH_HISTORY_KEY);
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn removes_only_the_match_history_key() {
        let out = drop_match_history(json!({"matchHistory": [{"id": 1}], "other": 2, "nested": {"matchHistory": 3}}));
        assert_eq!(out, json!({"other": 2, "nested": {"matchHistory": 3}}));
    }

    #[test]
    fn removes_a_value_of_any_type() {
        for value in [json!("text"), json!(null), json!(7), json!({"a": 1})] {
            let out = drop_match_history(json!({"matchHistory": value, "keep": true}));
            assert_eq!(out, json!({"keep": true}));
        }
    }

    #[test]
    fn is_a_no_op_without_the_key() {
        let data = json!({"other": 1});
        assert_eq!(drop_match_history(data.clone()), data);
    }

    #[test]
    fn is_idempotent() {
        let once = drop_match_history(json!({"matchHistory": [1], "k": 1}));
        assert_eq!(drop_match_history(once.clone()), once);
    }

    #[test]
    fn leaves_non_object_data_alone() {
        for data in [json!(null), json!([1, 2]), json!("s")] {
            assert_eq!(drop_match_history(data.clone()), data);
        }
    }

    #[test]
    fn only_connection_settings_has_it_registered() {
        assert_eq!(for_store("connection-settings").len(), 1);
        for store in ["app-settings", "presets", "stats-cache", "gc-state"] {
            assert!(for_store(store).is_empty(), "{store}");
        }
    }
}
