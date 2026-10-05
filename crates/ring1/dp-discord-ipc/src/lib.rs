mod client;
mod kind;
mod paths;
mod proto;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod win;
#[cfg(unix)]
use unix as platform;
#[cfg(windows)]
use win as platform;

pub use client::{Client, Error, Transport};
pub use kind::{classify_bundle, classify_exe_name, classify_path, select_targets, ClientKind, Pipe};
pub use proto::{decode, encode, Activity, Button, Packet, Party, ProtoError, Ready};

#[cfg(any(unix, windows))]
pub type Connection = platform::Connection;

/// Lists the running Discord clients' IPC endpoints, each labelled when its owner can be identified.
#[cfg(any(unix, windows))]
pub fn discover() -> Vec<Pipe> {
    platform::discover()
}

/// Opens the pipe and completes the handshake. Calls block until Discord replies, so run them
/// off the UI thread.
#[cfg(any(unix, windows))]
pub fn connect(pipe: &Pipe, client_id: &str) -> Result<Client<Connection>, Error> {
    Client::handshake(platform::connect(pipe)?, client_id)
}

#[cfg(not(any(unix, windows)))]
pub fn discover() -> Vec<Pipe> {
    Vec::new()
}
