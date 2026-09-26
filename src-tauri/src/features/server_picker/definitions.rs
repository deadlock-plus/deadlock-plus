use serde::{Deserialize, Serialize};
use ts_rs::TS;

const GAMES_JSON: &str = include_str!("../../../resources/games.json");

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum KeywordFilterMode {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "include")]
    Include,
    #[serde(rename = "exclude")]
    Exclude,
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RoutingNoteDefinition {
    pub matches: Vec<String>,
    pub note: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GameDefinition {
    pub id: String,
    pub display_name: String,
    pub app_id: u32,
    pub keyword_filter_mode: KeywordFilterMode,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub cluster_keywords: Vec<String>,
    #[serde(default)]
    pub routing_notes: Vec<RoutingNoteDefinition>,
    /// Cluster names server-picker-x uses for this game; its rules are named
    /// `server_picker_x_<name without spaces>`, so we need them to find its rules.
    #[serde(default)]
    pub server_picker_x_clusters: Vec<String>,
}

pub fn load_definitions() -> Vec<GameDefinition> {
    serde_json::from_str(GAMES_JSON).expect("resources/games.json must be valid and match GameDefinition schema")
}

pub fn find_definition(game_id: &str) -> Option<GameDefinition> {
    load_definitions().into_iter().find(|g| g.id == game_id)
}
