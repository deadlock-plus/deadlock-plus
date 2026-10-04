mod coalescer;
mod map;

pub use coalescer::{Coalescer, Poll, MIN_SEND_INTERVAL_MS};
pub use map::{map, GameFacts, Presence, PresenceLevel, MAX_TEXT_CHARS, MIN_TEXT_CHARS};
