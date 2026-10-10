use std::panic::{catch_unwind, AssertUnwindSafe};

use deadlock_reader::netchan::NetChanStats;
use deadlock_reader::Reader;

/// The user's own connection to the game server, as the game's net channel reports it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnginePing {
    pub ping_ms: f32,
    /// Fraction of packets lost inbound, 0.0..=1.0.
    pub loss_down: f32,
    /// Fraction of packets lost outbound, 0.0..=1.0.
    pub loss_up: f32,
    /// The worse of the inbound and outbound jitter.
    pub jitter_ms: f32,
}

impl EnginePing {
    pub(crate) fn from_stats(stats: &NetChanStats) -> Self {
        Self {
            ping_ms: stats.ping_ms as f32,
            loss_down: stats.loss_down,
            loss_up: stats.loss_up,
            jitter_ms: stats.jitter_in_ms.max(stats.jitter_out_ms),
        }
    }
}

/// `None` when the game is not connected to a server or the channel cannot be read.
pub(crate) fn read(reader: &Reader) -> Option<EnginePing> {
    // The reader walks foreign process memory; a bug in that walk must not take the app down.
    match catch_unwind(AssertUnwindSafe(|| reader.net_chan_stats())) {
        Ok(Ok(stats)) => Some(EnginePing::from_stats(&stats)),
        Ok(Err(e)) => {
            log::debug!("engine ping not readable: {e}");
            None
        }
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadlock_reader::netchan::NetChanStats;

    #[test]
    fn jitter_is_the_worse_direction_and_ping_is_widened() {
        let stats = NetChanStats { ping_ms: 42, loss_down: 0.25, loss_up: 0.5, jitter_in_ms: 2.0, jitter_out_ms: 7.5 };
        let got = EnginePing::from_stats(&stats);
        assert_eq!(got, EnginePing { ping_ms: 42.0, loss_down: 0.25, loss_up: 0.5, jitter_ms: 7.5 });
    }
}
