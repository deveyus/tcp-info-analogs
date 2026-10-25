//! Unconditionally compiled decoders.
//!
//! These parse raw byte buffers returned by the platform syscalls into
//! normalized [`crate::TcpMetrics`].

pub mod linux;
pub mod macos;
pub mod windows;
