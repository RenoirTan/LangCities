//! library that listens to a certain selection of signals
//! and rebroadcasts them using [`tokio::sync::broadcast::Sender`]
//!
//! also includes some opinions on what each signal does what

pub mod lc;
#[cfg(unix)]
pub mod unix_tokio;
#[cfg(windows)]
pub mod windows;

#[cfg(feature = "signal-hook")]
pub mod unix_signal_hook;
