//! Extension trait for querying TCP metrics from common socket types.

use crate::{TcpInfoError, TcpMetrics};

/// Extension trait for querying TCP connection health and congestion metrics.
pub trait TcpInfoExt {
    /// Fetch normalized TCP metrics for this connection.
    fn tcp_metrics(&self) -> Result<TcpMetrics, TcpInfoError>;
}

impl TcpInfoExt for std::net::TcpStream {
    fn tcp_metrics(&self) -> Result<TcpMetrics, TcpInfoError> {
        crate::sys::fetch_metrics(self)
    }
}

#[cfg(feature = "socket2")]
impl TcpInfoExt for socket2::Socket {
    fn tcp_metrics(&self) -> Result<TcpMetrics, TcpInfoError> {
        crate::sys::fetch_metrics(self)
    }
}

#[cfg(feature = "tokio")]
impl TcpInfoExt for tokio::net::TcpStream {
    fn tcp_metrics(&self) -> Result<TcpMetrics, TcpInfoError> {
        crate::sys::fetch_metrics(self)
    }
}
