//! Linux `getsockopt(SOL_TCP, TCP_INFO)` syscall driver.

use crate::{TcpInfoError, TcpMetrics};
use std::os::fd::AsRawFd;

/// Size of the raw buffer handed to the kernel. The kernel truncates its
/// reply to this length; 256 bytes exceeds the current `struct tcp_info`.
const BUFFER_LEN: usize = 256;

pub fn fetch_metrics<S: AsRawFd>(s: &S) -> Result<TcpMetrics, TcpInfoError> {
    let mut buf = [0u8; BUFFER_LEN];
    let mut len = BUFFER_LEN as libc::socklen_t;

    let res = unsafe {
        libc::getsockopt(
            s.as_raw_fd(),
            libc::SOL_TCP,
            libc::TCP_INFO,
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

    // The kernel cannot report more bytes than the supplied buffer, so the
    // clamp keeps `written` at or below BUFFER_LEN and `get` cannot fail.
    // A hypothetical None maps to a truthful Io error rather than a panic.
    let written = (len as usize).min(BUFFER_LEN);
    let owned = buf.get(..written).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "kernel reported more bytes than the supplied buffer",
        )
    })?;
    crate::decode::linux::decode_linux(owned)
}
