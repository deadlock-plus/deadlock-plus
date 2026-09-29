//! One ICMP echo, whole-call blocking. Windows uses IP Helper; elsewhere the system `ping` runs.

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::ping;

// Compiled everywhere so its parser tests run on every host.
#[cfg_attr(windows, allow(dead_code))]
mod unix;
#[cfg(not(windows))]
pub use unix::ping;
