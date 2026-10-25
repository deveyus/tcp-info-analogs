//! macOS `getsockopt(IPPROTO_TCP, TCP_CONNECTION_INFO)` syscall driver.

use crate::{TcpInfoError, TcpMetrics};
use std::os::fd::AsRawFd;

/// Size of the raw buffer handed to the kernel. The kernel truncates its
/// reply to this length; 256 bytes exceeds `struct tcp_connection_info`.
const BUFFER_LEN: usize = 256;

/// `TCP_CONNECTION_INFO` from Darwin `netinet/tcp.h`.
const TCP_CONNECTION_INFO: libc::c_int = 0x106;

/// Minimum bytes the decoder needs (`tcpi_txretransmitpackets` ends at 112).
const REQUIRED_LEN: usize = 112;

pub fn fetch_metrics<S: AsRawFd>(s: &S) -> Result<TcpMetrics, TcpInfoError> {
    let mut buf = [0u8; BUFFER_LEN];
    let mut len = BUFFER_LEN as libc::socklen_t;

    let res = unsafe {
        libc::getsockopt(
            s.as_raw_fd(),
            libc::IPPROTO_TCP,
            TCP_CONNECTION_INFO,
            buf.as_mut_ptr() as *mut libc::c_void,
            &mut len,
        )
    };

    if res != 0 {
        let err = std::io::Error::last_os_error();
        if err.raw_os_error() == Some(libc::ENOTCONN) {
            return Err(TcpInfoError::NotConnected);
        }
        return Err(TcpInfoError::Io(err));
    }

    let written = (len as usize).min(BUFFER_LEN);
    if written < REQUIRED_LEN {
        return Err(TcpInfoError::BufferTooSmall {
            required: REQUIRED_LEN,
            provided: written,
        });
    }

    crate::decode::macos::decode_macos(&buf[..written])
}
