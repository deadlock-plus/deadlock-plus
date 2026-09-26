pub mod commands;
#[cfg(windows)]
mod icmp;
#[cfg(windows)]
pub(crate) use icmp::ping as icmp_ping;
#[cfg(windows)]
mod history_store;
#[cfg(windows)]
mod monitor;
mod types;

#[cfg(windows)]
pub use monitor::NetworkMonitor;

#[cfg(not(windows))]
#[derive(Default)]
pub struct NetworkMonitor;
