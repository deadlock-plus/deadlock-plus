//! The official Steam Web API, fetched independently of `alerts`' `/v2/patches` ingest. Its
//! `maxlength=0` returns the full, untruncated body going back much further than the RSS-derived
//! feed's ~30-item window — see `plans/patch-notes-full-text.md` for why that feed alone is not
//! enough and `DECISIONS.md` for the confirmed sizes.
use serde::Deserialize;
use time::OffsetDateTime;

use crate::bbcode::strip_bbcode;
use crate::store::{PatchOrigin, PatchSource};

pub const URL: &str = "https://api.steampowered.com/ISteamNews/GetNewsForApp/v2/?appid=1422450&count=200&maxlength=0&feeds=steam_community_announcements";

#[derive(Deserialize)]
struct Response {
    appnews: AppNews,
}

#[derive(Deserialize)]
struct AppNews {
    newsitems: Vec<NewsItem>,
}

#[derive(Deserialize)]
struct NewsItem {
    gid: String,
    title: String,
    url: String,
    contents: String,
    date: i64,
}

/// `gid` has no other meaning here beyond a stable id: there is no documented way to fetch one
/// announcement by `gid` directly (a `gid=` query param is silently ignored), so every poll
/// re-fetches the whole batch and matches against what is already indexed.
fn id_of(gid: &str) -> String {
    format!("steam-news:{gid}")
}

fn published_of(unix_secs: i64) -> String {
    let dt = OffsetDateTime::from_unix_timestamp(unix_secs).unwrap_or(OffsetDateTime::UNIX_EPOCH);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        dt.year(),
        u8::from(dt.month()),
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second()
    )
}

pub fn parse_news_with_text(json: &str) -> Result<Vec<(PatchSource, String, Vec<String>)>, serde_json::Error> {
    let resp: Response = serde_json::from_str(json)?;
    Ok(resp
        .appnews
        .newsitems
        .into_iter()
        .map(|item| {
            let source = PatchSource {
                id: id_of(&item.gid),
                title: item.title.trim().to_owned(),
                published: published_of(item.date),
                link: item.url,
                origin: PatchOrigin::Steam,
            };
            let (text, images) = strip_bbcode(&item.contents);
            (source, text, images)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_newsitems_into_sourced_patches_with_stripped_bodies() {
        let json = r#"{"appnews":{"newsitems":[
            {"gid":"123","title":" Minor Update - 09-16-2026 ","url":"https://store.steampowered.com/news/app/1422450/view/1",
             "contents":"[h3]General[/h3][p]Guardian bounty increased by 10%[/p]","date":1757980800,"feedname":"steam_community_announcements"}
        ]}}"#;
        let items = parse_news_with_text(json).unwrap();
        assert_eq!(items.len(), 1);
        let (source, text, images) = &items[0];
        assert_eq!(source.id, "steam-news:123");
        assert_eq!(source.title, "Minor Update - 09-16-2026");
        assert_eq!(source.link, "https://store.steampowered.com/news/app/1422450/view/1");
        assert_eq!(source.published, "2025-09-16T00:00:00Z");
        assert_eq!(source.origin, PatchOrigin::Steam);
        assert_eq!(text, "[ General ]\nGuardian bounty increased by 10%");
        assert!(images.is_empty());
    }

    #[test]
    fn a_malformed_response_is_an_error() {
        assert!(parse_news_with_text(r#"{"nope": true}"#).is_err());
    }
}
