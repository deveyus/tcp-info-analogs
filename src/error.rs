//! Error type for metric queries and decoding.

use std::fmt;

/// Errors encountered while querying or decoding TCP connection metrics.
#[non_exhaustive]
#[derive(Debug)]
pub enum TcpInfoError {
    /// Socket is not connected (returned by the OS as `ENOTCONN` / `WSAENOTCONN`).
    NotConnected,
    /// Kernel returned fewer bytes than needed to decode minimum operational metrics.
    BufferTooSmall {
        /// Bytes required by the platform decoder.
        required: usize,
        /// Bytes actually populated by the kernel.
        provided: usize,
    },
    /// Low-level OS I/O failure.
    Io(std::io::Error),
    /// Platform is not supported by this crate.
    UnsupportedPlatform,
}

impl fmt::Display for TcpInfoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConnected => write!(f, "socket is not connected"),
            Self::BufferTooSmall { required, provided } => {
                write!(
                    f,
                    "buffer too small: required {required} bytes, provided {provided}"
                )
            }
            Self::Io(err) => write!(f, "OS error: {err}"),
            Self::UnsupportedPlatform => write!(f, "unsupported platform"),
        }
    }
}

impl std::error::Error for TcpInfoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for TcpInfoError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
