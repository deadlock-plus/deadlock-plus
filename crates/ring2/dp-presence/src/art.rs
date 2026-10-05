use crate::config::{Image, ImageSource};
use crate::map::{LiveFacts, MatchMode, Perspective};
use crate::state::StateId;

const RANK_IMAGE_BASE: &str = "https://api.deadlock-api.com/v1/assets/ranks";
const RANK_TIER_BASE: &str = "https://assets-bucket.deadlock-api.com/assets-api-res/images/ranks";
const MAX_URL_CHARS: usize = 256;

/// Splits a packed badge (`tier * 10 + subrank`) when a rank has been assigned.
pub(crate) fn split_rank(packed: u32) -> Option<(u32, u32)> {
    (packed > 0).then_some((packed / 10, packed % 10))
}

/// The rank the player shown holds, only for their own ranked matches.
pub(crate) fn own_rank(live: &LiveFacts) -> Option<(u32, u32)> {
    if live.perspective == Perspective::Spectating || live.match_mode != Some(MatchMode::Ranked) {
        return None;
    }
    live.rank.and_then(split_rank)
}

fn rank_url((tier, sub): (u32, u32)) -> String {
    if sub == 0 {
        format!("{RANK_TIER_BASE}/rank{tier:02}_lg.png")
    } else {
        format!("{RANK_IMAGE_BASE}/{tier}/{sub}/image")
    }
}

fn custom_url(url: &str) -> Option<String> {
    let ok = url.starts_with("https://")
        && url.len() > "https://".len()
        && url.chars().count() <= MAX_URL_CHARS
        && !url.chars().any(|c| c.is_whitespace() || c.is_control());
    ok.then(|| url.to_owned())
}

/// The URL Discord should fetch for an image slot. Hero art follows the same rule as the `{hero}` placeholder: it is
/// withheld whenever it would describe someone other than the local player.
pub(crate) fn image_url(image: &Image, state: StateId, live: &LiveFacts) -> Option<String> {
    if !image.enabled {
        return None;
    }
    match &image.source {
        ImageSource::HeroPortrait => hero_art(state, live, live.hero_portrait.as_ref().or(live.hero_icon.as_ref())),
        ImageSource::HeroIcon => hero_art(state, live, live.hero_icon.as_ref().or(live.hero_portrait.as_ref())),
        ImageSource::RankBadge => own_rank(live).map(rank_url),
        ImageSource::ModeIcon => None,
        ImageSource::CustomUrl(url) => custom_url(url),
    }
}

fn hero_art(state: StateId, live: &LiveFacts, url: Option<&String>) -> Option<String> {
    crate::map::hero_allowed(state, live).then(|| url.cloned()).flatten()
}
