//! Fallback driver for unsupported platforms.

use crate::{TcpInfoError, TcpMetrics};

pub fn fetch_metrics<S>(_s: &S) -> Result<TcpMetrics, TcpInfoError> {
    Err(TcpInfoError::UnsupportedPlatform)
}
