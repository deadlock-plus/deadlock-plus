pub mod commands;
pub mod definitions;
#[cfg(windows)]
mod external;
#[cfg(windows)]
mod firewall;
mod ping;
pub(crate) mod sdr;
mod state;
#[cfg(windows)]
mod sync;
mod validate;

pub use state::ServerPickerState;

#[cfg(windows)]
pub use sync::start;
#[cfg(windows)]
pub(crate) use sync::SYNC_JOB;

#[cfg(not(windows))]
pub fn start(_app: &tauri::AppHandle) {}
