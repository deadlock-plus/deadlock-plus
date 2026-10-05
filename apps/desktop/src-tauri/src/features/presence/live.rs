use std::collections::HashMap;

use serde::Deserialize;
use ts_rs::TS;

/// What the frontend knows about a hero that the game memory does not: the name and the art URLs.
#[derive(Deserialize, Clone, Default, PartialEq, Eq, Debug, TS)]
#[ts(export, rename = "PresenceHeroArt")]
#[serde(rename_all = "camelCase")]
pub struct HeroArt {
    pub name: String,
    pub portrait: Option<String>,
    pub icon: Option<String>,
    /// The hero's own hideout line, for `{heroPresence}`.
    pub hideout_line: Option<String>,
}

#[derive(Default, Clone, Debug)]
pub struct Lookups {
    pub heroes: HashMap<u32, HeroArt>,
    /// Rank tier to its display name.
    pub ranks: HashMap<u32, String>,
}

pub trait Feed {
    fn poll(&mut self) -> Option<dp_live::LiveFacts>;
}

#[cfg(windows)]
impl Feed for dp_live::LiveFeed {
    fn poll(&mut self) -> Option<dp_live::LiveFacts> {
        dp_live::LiveFeed::poll(self)
    }
}

/// Stand-in where the game cannot be read; Detailed then behaves as Basic.
#[cfg(not(windows))]
#[derive(Default)]
pub struct NoFeed;

#[cfg(not(windows))]
impl Feed for NoFeed {
    fn poll(&mut self) -> Option<dp_live::LiveFacts> {
        None
    }
}

#[cfg(windows)]
pub type PlatformFeed = dp_live::LiveFeed;
#[cfg(not(windows))]
pub type PlatformFeed = NoFeed;

/// Owns the feed only while it is wanted, so game memory is not touched at any other time.
pub struct LiveSource<F> {
    feed: Option<F>,
}

impl<F: Feed> LiveSource<F> {
    pub fn new() -> Self {
        Self { feed: None }
    }

    #[cfg(test)]
    pub fn is_open(&self) -> bool {
        self.feed.is_some()
    }

    pub fn poll(&mut self, wanted: bool, make: impl FnOnce() -> F) -> Option<dp_live::LiveFacts> {
        if !wanted {
            self.feed = None;
            return None;
        }
        self.feed.get_or_insert_with(make).poll()
    }
}

pub fn convert(facts: &dp_live::LiveFacts, lookups: &Lookups) -> dp_presence::LiveFacts {
    let hero = facts.hero_id.and_then(|id| lookups.heroes.get(&id));
    let rank_name = facts.rank.filter(|r| *r > 0).and_then(|r| lookups.ranks.get(&(r / 10)).cloned());
    use dp_presence as p;
    p::LiveFacts {
        context: match facts.context {
            dp_live::Context::Other => p::Context::Other,
            dp_live::Context::Hideout => p::Context::Hideout,
            dp_live::Context::Match => p::Context::Match,
        },
        phase: facts.phase.map(|v| match v {
            dp_live::Phase::HeroSelection => p::Phase::HeroSelection,
            dp_live::Phase::MatchIntro => p::Phase::MatchIntro,
            dp_live::Phase::Loading => p::Phase::Loading,
            dp_live::Phase::PreGame => p::Phase::PreGame,
            dp_live::Phase::InProgress => p::Phase::InProgress,
            dp_live::Phase::PostGame => p::Phase::PostGame,
        }),
        perspective: match facts.perspective {
            dp_live::Perspective::Unknown => p::Perspective::Unknown,
            dp_live::Perspective::Playing => p::Perspective::Playing,
            dp_live::Perspective::Spectating => p::Perspective::Spectating,
        },
        match_mode: facts.match_mode.map(match_mode),
        game_mode: facts.game_mode.map(game_mode),
        hero: hero.map(|h| h.name.clone()),
        hero_id: facts.hero_id,
        hero_portrait: hero.and_then(|h| h.portrait.clone()),
        hero_icon: hero.and_then(|h| h.icon.clone()),
        hero_presence: hero.and_then(|h| h.hideout_line.clone()),
        match_time_secs: facts.match_time_secs,
        paused: facts.paused,
        drift: facts.drift,
        local_won: facts.local_won,
        kills: facts.kills,
        deaths: facts.deaths,
        assists: facts.assists,
        souls: facts.souls,
        rank: facts.rank,
        rank_name,
        street_brawl: facts.street_brawl.map(|b| p::StreetBrawlFacts {
            round: b.round,
            amber: b.amber,
            sapphire: b.sapphire,
        }),
        match_id: facts.match_id,
        party: facts.party.map(|party| p::PartyFacts {
            size: party.size,
            queueing: party.queueing,
            queued_secs: party.queued_secs,
            match_mode: party.match_mode.map(match_mode),
            game_mode: party.game_mode.map(game_mode),
        }),
    }
}

fn match_mode(v: dp_live::MatchMode) -> dp_presence::MatchMode {
    use dp_presence::MatchMode as P;
    match v {
        dp_live::MatchMode::Unranked => P::Unranked,
        dp_live::MatchMode::Ranked => P::Ranked,
        dp_live::MatchMode::PrivateLobby => P::PrivateLobby,
        dp_live::MatchMode::CoopBot => P::CoopBot,
        dp_live::MatchMode::HeroLabs => P::HeroLabs,
        dp_live::MatchMode::Tutorial => P::Tutorial,
        dp_live::MatchMode::Other => P::Other,
    }
}

fn game_mode(v: dp_live::GameMode) -> dp_presence::GameMode {
    use dp_presence::GameMode as P;
    match v {
        dp_live::GameMode::Normal => P::Normal,
        dp_live::GameMode::StreetBrawl => P::StreetBrawl,
        dp_live::GameMode::Sandbox => P::Sandbox,
        dp_live::GameMode::ExploreNyc => P::ExploreNyc,
        dp_live::GameMode::Other => P::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    struct Fake {
        polls: Rc<Cell<u32>>,
        drops: Rc<Cell<u32>>,
        reply: Option<dp_live::LiveFacts>,
    }

    impl Feed for Fake {
        fn poll(&mut self) -> Option<dp_live::LiveFacts> {
            self.polls.set(self.polls.get() + 1);
            self.reply
        }
    }

    impl Drop for Fake {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
        }
    }

    #[test]
    fn converts_every_field_and_resolves_the_hero() {
        let live = dp_live::LiveFacts {
            context: dp_live::Context::Match,
            phase: Some(dp_live::Phase::InProgress),
            perspective: dp_live::Perspective::Spectating,
            match_mode: Some(dp_live::MatchMode::HeroLabs),
            game_mode: Some(dp_live::GameMode::StreetBrawl),
            hero_id: Some(7),
            match_time_secs: Some(12.5),
            paused: true,
            drift: true,
            local_won: Some(false),
            kills: Some(1),
            deaths: Some(2),
            assists: Some(3),
            souls: Some(4000),
            rank: Some(93),
            street_brawl: Some(dp_live::StreetBrawlFacts { round: Some(3), amber: Some(2), sapphire: Some(1) }),
            match_id: Some(111_074_434),
            party: Some(dp_live::PartyFacts {
                size: 2,
                queueing: true,
                queued_secs: Some(30),
                match_mode: Some(dp_live::MatchMode::Ranked),
                game_mode: Some(dp_live::GameMode::StreetBrawl),
            }),
        };
        let heroes = Lookups {
            heroes: HashMap::from([(
                7,
                HeroArt {
                    name: "Seven".into(),
                    portrait: Some("card".into()),
                    icon: Some("icon".into()),
                    hideout_line: Some("Plotting in the Hideout".into()),
                },
            )]),
            ranks: HashMap::from([(9, "Ritualist".to_owned())]),
        };
        assert_eq!(
            convert(&live, &heroes),
            dp_presence::LiveFacts {
                context: dp_presence::Context::Match,
                phase: Some(dp_presence::Phase::InProgress),
                perspective: dp_presence::Perspective::Spectating,
                match_mode: Some(dp_presence::MatchMode::HeroLabs),
                game_mode: Some(dp_presence::GameMode::StreetBrawl),
                hero: Some("Seven".into()),
                hero_id: Some(7),
                hero_portrait: Some("card".into()),
                hero_icon: Some("icon".into()),
                hero_presence: Some("Plotting in the Hideout".into()),
                match_time_secs: Some(12.5),
                paused: true,
                drift: true,
                local_won: Some(false),
                kills: Some(1),
                deaths: Some(2),
                assists: Some(3),
                souls: Some(4000),
                rank: Some(93),
                rank_name: Some("Ritualist".into()),
                street_brawl: Some(dp_presence::StreetBrawlFacts { round: Some(3), amber: Some(2), sapphire: Some(1) }),
                match_id: Some(111_074_434),
                party: Some(dp_presence::PartyFacts {
                    size: 2,
                    queueing: true,
                    queued_secs: Some(30),
                    match_mode: Some(dp_presence::MatchMode::Ranked),
                    game_mode: Some(dp_presence::GameMode::StreetBrawl),
                }),
            }
        );
    }

    #[test]
    fn unknown_or_absent_hero_is_none() {
        let heroes = Lookups::default();
        let mut live = dp_live::LiveFacts { hero_id: Some(9), ..Default::default() };
        assert_eq!(convert(&live, &heroes).hero, None);
        live.hero_id = None;
        let known = Lookups {
            heroes: HashMap::from([(9, HeroArt { name: "Nine".into(), ..HeroArt::default() })]),
            ..Lookups::default()
        };
        assert_eq!(convert(&live, &known).hero, None);
    }

    #[test]
    fn rank_name_is_looked_up_by_tier_and_missing_when_unranked_or_unknown() {
        let lookups = Lookups { ranks: HashMap::from([(5, "Alchemist".to_owned())]), ..Lookups::default() };
        let with = |rank| convert(&dp_live::LiveFacts { rank, ..Default::default() }, &lookups).rank_name;
        assert_eq!(with(Some(52)).as_deref(), Some("Alchemist"));
        assert_eq!(with(Some(0)), None);
        assert_eq!(with(Some(93)), None);
        assert_eq!(with(None), None);
    }

    #[test]
    fn converts_each_enum_variant() {
        use dp_live as l;
        let heroes = Lookups::default();
        for (a, b) in [
            (l::Phase::HeroSelection, dp_presence::Phase::HeroSelection),
            (l::Phase::MatchIntro, dp_presence::Phase::MatchIntro),
            (l::Phase::Loading, dp_presence::Phase::Loading),
            (l::Phase::PreGame, dp_presence::Phase::PreGame),
            (l::Phase::InProgress, dp_presence::Phase::InProgress),
            (l::Phase::PostGame, dp_presence::Phase::PostGame),
        ] {
            let live = l::LiveFacts { phase: Some(a), ..Default::default() };
            assert_eq!(convert(&live, &heroes).phase, Some(b));
        }
        for (a, b) in [
            (l::MatchMode::Unranked, dp_presence::MatchMode::Unranked),
            (l::MatchMode::Ranked, dp_presence::MatchMode::Ranked),
            (l::MatchMode::PrivateLobby, dp_presence::MatchMode::PrivateLobby),
            (l::MatchMode::CoopBot, dp_presence::MatchMode::CoopBot),
            (l::MatchMode::HeroLabs, dp_presence::MatchMode::HeroLabs),
            (l::MatchMode::Tutorial, dp_presence::MatchMode::Tutorial),
            (l::MatchMode::Other, dp_presence::MatchMode::Other),
        ] {
            let live = l::LiveFacts { match_mode: Some(a), ..Default::default() };
            assert_eq!(convert(&live, &heroes).match_mode, Some(b));
        }
        for (a, b) in [
            (l::GameMode::Normal, dp_presence::GameMode::Normal),
            (l::GameMode::StreetBrawl, dp_presence::GameMode::StreetBrawl),
            (l::GameMode::Sandbox, dp_presence::GameMode::Sandbox),
            (l::GameMode::ExploreNyc, dp_presence::GameMode::ExploreNyc),
            (l::GameMode::Other, dp_presence::GameMode::Other),
        ] {
            let live = l::LiveFacts { game_mode: Some(a), ..Default::default() };
            assert_eq!(convert(&live, &heroes).game_mode, Some(b));
        }
        for (a, b) in [
            (l::Context::Other, dp_presence::Context::Other),
            (l::Context::Hideout, dp_presence::Context::Hideout),
            (l::Context::Match, dp_presence::Context::Match),
        ] {
            let live = l::LiveFacts { context: a, ..Default::default() };
            assert_eq!(convert(&live, &heroes).context, b);
        }
        for (a, b) in [
            (l::Perspective::Unknown, dp_presence::Perspective::Unknown),
            (l::Perspective::Playing, dp_presence::Perspective::Playing),
            (l::Perspective::Spectating, dp_presence::Perspective::Spectating),
        ] {
            let live = l::LiveFacts { perspective: a, ..Default::default() };
            assert_eq!(convert(&live, &heroes).perspective, b);
        }
    }

    fn rig() -> (Rc<Cell<u32>>, Rc<Cell<u32>>, impl Fn() -> Fake) {
        let (polls, drops) = (Rc::new(Cell::new(0)), Rc::new(Cell::new(0)));
        let (p, d) = (polls.clone(), drops.clone());
        (polls, drops, move || Fake { polls: p.clone(), drops: d.clone(), reply: Some(dp_live::LiveFacts::default()) })
    }

    #[test]
    fn not_wanted_never_opens_the_feed() {
        let (polls, _, make) = rig();
        let mut src = LiveSource::new();
        assert!(src.poll(false, &make).is_none());
        assert!(!src.is_open());
        assert_eq!(polls.get(), 0);
    }

    #[test]
    fn wanted_opens_once_and_polls_every_tick() {
        let (polls, drops, make) = rig();
        let mut src = LiveSource::new();
        assert!(src.poll(true, &make).is_some());
        assert!(src.poll(true, || panic!("must reuse the open feed")).is_some());
        assert_eq!((polls.get(), drops.get()), (2, 0));
    }

    #[test]
    fn leaving_wanted_drops_the_feed_and_a_return_opens_a_fresh_one() {
        let (polls, drops, make) = rig();
        let mut src = LiveSource::new();
        src.poll(true, &make);
        assert!(src.poll(false, &make).is_none());
        assert!(!src.is_open());
        assert_eq!(drops.get(), 1);
        src.poll(true, &make);
        assert_eq!(polls.get(), 2);
    }

    #[test]
    fn a_feed_returning_none_passes_it_through() {
        let mut src = LiveSource::new();
        let none = || Fake { polls: Rc::default(), drops: Rc::default(), reply: None };
        assert!(src.poll(true, none).is_none());
        assert!(src.is_open());
    }
}
