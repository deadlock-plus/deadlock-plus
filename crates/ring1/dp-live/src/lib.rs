#[cfg(windows)]
mod convert;
#[cfg(windows)]
mod feed;

#[cfg(windows)]
pub use convert::from_snapshot;
#[cfg(windows)]
pub use feed::LiveFeed;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Context {
    #[default]
    Other,
    Hideout,
    Match,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    HeroSelection,
    MatchIntro,
    Loading,
    PreGame,
    InProgress,
    PostGame,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Perspective {
    #[default]
    Unknown,
    Playing,
    Spectating,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchMode {
    Unranked,
    Ranked,
    PrivateLobby,
    CoopBot,
    HeroLabs,
    Tutorial,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameMode {
    Normal,
    StreetBrawl,
    Sandbox,
    ExploreNyc,
    Other,
}

/// Game facts safe to hand to presence code: no names, steam ids, match ids or other players.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LiveFacts {
    pub context: Context,
    pub phase: Option<Phase>,
    pub perspective: Perspective,
    pub match_mode: Option<MatchMode>,
    pub game_mode: Option<GameMode>,
    pub hero_id: Option<u32>,
    pub match_time_secs: Option<f32>,
    pub paused: bool,
    /// The reader noticed the game changed under it; the other fields may be wrong.
    pub drift: bool,
    pub local_won: Option<bool>,
    pub kills: Option<u32>,
    pub deaths: Option<u32>,
    pub assists: Option<u32>,
    pub souls: Option<u32>,
}
