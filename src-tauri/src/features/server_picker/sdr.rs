use std::collections::HashMap;
use ts_rs::TS;

use serde::{Deserialize, Serialize};

use super::definitions::{GameDefinition, KeywordFilterMode, RoutingNoteDefinition};

const SDR_CONFIG_URL_TEMPLATE: &str = "https://api.steampowered.com/ISteamApps/GetSDRConfig/v1/?appid={app_id}";

#[derive(Debug, Deserialize)]
struct RawSdrResponse {
    revision: serde_json::Value,
    pops: HashMap<String, RawPop>,
}

#[derive(Debug, Deserialize)]
struct RawPop {
    desc: Option<String>,
    relays: Option<Vec<RawRelay>>,
}

#[derive(Debug, Deserialize)]
struct RawRelay {
    ipv4: Option<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct RoutingNoteInfo {
    pub note: String,
    pub related_group_ids: Vec<String>,
    /// Names from the note's `matches` that have no relay group at all in Valve's data,
    /// so they're part of the route but can't be blocked here.
    pub unblockable_matches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ServerGroup {
    pub id: String,
    pub description: String,
    pub is_cluster: bool,
    pub country_code: Option<String>,
    pub relay_ips: Vec<String>,
    /// Ids of the raw pops merged into this cluster; empty for a plain pop.
    pub member_ids: Vec<String>,
    pub routing_note: Option<RoutingNoteInfo>,
}

#[derive(Debug, Clone, Serialize, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ServerData {
    pub revision: String,
    pub unclustered: Vec<ServerGroup>,
    pub clustered: Vec<ServerGroup>,
}

#[derive(thiserror::Error, Debug)]
pub enum SdrError {
    #[error("network request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("relay data unavailable, Valve's SDR config endpoint returned no usable pops")]
    Empty,
}

fn is_accepted(desc: &str, def: &GameDefinition) -> bool {
    match def.keyword_filter_mode {
        KeywordFilterMode::Include => def.keywords.iter().any(|k| desc.contains(k.as_str())),
        KeywordFilterMode::Exclude => !def.keywords.iter().any(|k| desc.contains(k.as_str())),
        KeywordFilterMode::None => true,
    }
}

fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn country_code_for(desc: &str) -> Option<String> {
    const MAP: &[(&str, &str)] = &[
        ("frankfurt", "de"),
        ("germany", "de"),
        ("stockholm", "se"),
        ("sweden", "se"),
        ("helsinki", "fi"),
        ("finland", "fi"),
        ("hong kong", "hk"),
        ("singapore", "sg"),
        ("tokyo", "jp"),
        ("japan", "jp"),
        ("seoul", "kr"),
        ("south korea", "kr"),
        ("mumbai", "in"),
        ("chennai", "in"),
        ("india", "in"),
        ("dubai", "ae"),
        ("emirates", "ae"),
        ("sydney", "au"),
        ("perth", "au"),
        ("australia", "au"),
        ("johannesburg", "za"),
        ("south africa", "za"),
        ("sao paulo", "br"),
        ("brazil", "br"),
        ("santiago", "cl"),
        ("chile", "cl"),
        ("lima", "pe"),
        ("peru", "pe"),
        ("buenos aires", "ar"),
        ("argentina", "ar"),
        ("guam", "gu"),
        ("chicago", "us"),
        ("seattle", "us"),
        ("los angeles", "us"),
        ("virginia", "us"),
        ("atlanta", "us"),
        ("dallas", "us"),
        ("washington", "us"),
        ("moses lake", "us"),
        ("toronto", "ca"),
        ("canada", "ca"),
        ("madrid", "es"),
        ("spain", "es"),
        ("paris", "fr"),
        ("france", "fr"),
        ("london", "gb"),
        ("england", "gb"),
        ("amsterdam", "nl"),
        ("netherlands", "nl"),
        ("warsaw", "pl"),
        ("poland", "pl"),
        ("vienna", "at"),
        ("austria", "at"),
        ("milan", "it"),
        ("italy", "it"),
        ("moscow", "ru"),
        ("china", "cn"),
    ];

    let lower = desc.to_lowercase();
    MAP.iter().find(|(needle, _)| lower.contains(needle)).map(|(_, code)| code.to_string())
}

fn annotate_routing_notes(groups: &mut [ServerGroup], notes: &[RoutingNoteDefinition]) {
    for note in notes {
        let matching_ids: Vec<String> = groups
            .iter()
            .filter(|g| note.matches.iter().any(|m| g.description.contains(m.as_str())))
            .map(|g| g.id.clone())
            .collect();

        if matching_ids.is_empty() {
            continue;
        }

        let unblockable_matches: Vec<String> = note
            .matches
            .iter()
            .filter(|m| !groups.iter().any(|g| g.description.contains(m.as_str())))
            .cloned()
            .collect();

        for g in groups.iter_mut() {
            if matching_ids.contains(&g.id) {
                let related_group_ids = matching_ids.iter().filter(|id| *id != &g.id).cloned().collect();
                g.routing_note = Some(RoutingNoteInfo {
                    note: note.note.clone(),
                    related_group_ids,
                    unblockable_matches: unblockable_matches.clone(),
                });
            }
        }
    }
}

fn build_groups(pops: &HashMap<String, RawPop>, def: &GameDefinition) -> (Vec<ServerGroup>, Vec<ServerGroup>) {
    let mut unclustered: Vec<ServerGroup> = Vec::new();
    let mut clustered: Vec<ServerGroup> = Vec::new();

    for (pop_code, pop) in pops.iter() {
        let Some(relays) = &pop.relays else { continue };
        let Some(desc) = &pop.desc else { continue };

        if !is_accepted(desc, def) {
            continue;
        }

        let relay_ips: Vec<String> = relays.iter().filter_map(|r| r.ipv4.clone()).collect();
        if relay_ips.is_empty() {
            continue;
        }

        let single = ServerGroup {
            id: pop_code.clone(),
            description: desc.clone(),
            is_cluster: false,
            country_code: country_code_for(desc),
            relay_ips: relay_ips.clone(),
            member_ids: Vec::new(),
            routing_note: None,
        };
        unclustered.push(single);

        let cluster_name = def.cluster_keywords.iter().find(|k| desc.contains(k.as_str()));

        match cluster_name {
            Some(name) => {
                let cluster_id = slugify(name);
                if let Some(existing) = clustered.iter_mut().find(|g| g.id == cluster_id) {
                    existing.relay_ips.extend(relay_ips);
                    existing.member_ids.push(pop_code.clone());
                } else {
                    clustered.push(ServerGroup {
                        id: cluster_id,
                        description: name.clone(),
                        is_cluster: true,
                        country_code: country_code_for(name),
                        relay_ips,
                        member_ids: vec![pop_code.clone()],
                        routing_note: None,
                    });
                }
            }
            None => {
                clustered.push(ServerGroup {
                    id: pop_code.clone(),
                    description: desc.clone(),
                    is_cluster: false,
                    country_code: country_code_for(desc),
                    relay_ips,
                    member_ids: Vec::new(),
                    routing_note: None,
                });
            }
        }
    }

    (unclustered, clustered)
}

pub async fn fetch_server_data(client: &reqwest::Client, def: &GameDefinition) -> Result<ServerData, SdrError> {
    let url = SDR_CONFIG_URL_TEMPLATE.replace("{app_id}", &def.app_id.to_string());
    let raw: RawSdrResponse = client.get(url).send().await?.json().await?;

    let (mut unclustered, mut clustered) = build_groups(&raw.pops, def);

    if unclustered.is_empty() {
        return Err(SdrError::Empty);
    }

    annotate_routing_notes(&mut unclustered, &def.routing_notes);
    annotate_routing_notes(&mut clustered, &def.routing_notes);

    unclustered.sort_by(|a, b| a.description.cmp(&b.description));
    clustered.sort_by(|a, b| a.description.cmp(&b.description));

    Ok(ServerData { revision: raw.revision.to_string(), unclustered, clustered })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group(id: &str, description: &str) -> ServerGroup {
        ServerGroup {
            id: id.into(),
            description: description.into(),
            is_cluster: false,
            country_code: None,
            relay_ips: vec!["1.1.1.1".into()],
            member_ids: Vec::new(),
            routing_note: None,
        }
    }

    #[test]
    fn routing_note_flags_unblockable_helsinki() {
        let mut groups = vec![
            group("frankfurt", "Frankfurt"),
            group("stockholm", "Stockholm"),
            group("ams", "Amsterdam (Netherlands)"),
        ];
        let notes = vec![RoutingNoteDefinition {
            matches: vec!["Frankfurt".into(), "Stockholm".into(), "Helsinki".into()],
            note: "n".into(),
        }];

        annotate_routing_notes(&mut groups, &notes);

        let fra = groups[0].routing_note.as_ref().unwrap();
        assert_eq!(fra.related_group_ids, vec!["stockholm"]);
        assert_eq!(fra.unblockable_matches, vec!["Helsinki"]);
        assert!(groups[2].routing_note.is_none());
    }

    #[test]
    fn country_code_matches_city_or_country_case_insensitively() {
        assert_eq!(country_code_for("Frankfurt (Germany)").as_deref(), Some("de"));
        assert_eq!(country_code_for("JOHANNESBURG").as_deref(), Some("za"));
        assert_eq!(country_code_for("London (England)").as_deref(), Some("gb"));
    }

    #[test]
    fn country_code_is_none_for_unknown_places() {
        assert_eq!(country_code_for("Atlantis"), None);
        assert_eq!(country_code_for(""), None);
    }

    #[test]
    fn slugify_normalizes_names() {
        assert_eq!(slugify("Hong Kong"), "hong-kong");
    }

    fn def_with_clusters(clusters: &[&str]) -> GameDefinition {
        serde_json::from_value(serde_json::json!({
            "id": "g", "displayName": "G", "appId": 1, "keywordFilterMode": "none",
            "clusterKeywords": clusters,
        }))
        .unwrap()
    }

    fn pop(desc: &str, ip: &str) -> RawPop {
        RawPop { desc: Some(desc.into()), relays: Some(vec![RawRelay { ipv4: Some(ip.into()) }]) }
    }

    #[test]
    fn cluster_lists_the_pops_it_was_built_from() {
        let pops: HashMap<String, RawPop> = [
            ("fra".to_string(), pop("Frankfurt (Germany)", "1.1.1.1")),
            ("fra2".to_string(), pop("Frankfurt (Germany) 2", "2.2.2.2")),
            ("lhr".to_string(), pop("London (England)", "3.3.3.3")),
        ]
        .into();

        let (unclustered, clustered) = build_groups(&pops, &def_with_clusters(&["Frankfurt"]));

        assert_eq!(unclustered.len(), 3);
        let cluster = clustered.iter().find(|g| g.id == "frankfurt").unwrap();
        let mut members = cluster.member_ids.clone();
        members.sort();
        assert_eq!(members, vec!["fra", "fra2"]);
        assert_eq!(cluster.relay_ips.len(), 2);
        assert!(clustered.iter().find(|g| g.id == "lhr").unwrap().member_ids.is_empty());
    }
}
