//! Windows `WSAIoctl(SIO_TCP_INFO)` syscall driver.

use crate::{TcpInfoError, TcpMetrics};
use std::ffi::c_void;
use std::mem;
use std::os::windows::io::AsRawSocket;
use windows_sys::Win32::Networking::WinSock::{
    TCP_INFO_v0, WSAGetLastError, WSAIoctl, SOCKET_ERROR,
};

/// `SIO_TCP_INFO` = `_WSAIORW(IOC_VENDOR, 39)` = `0xD8000027`.
const SIO_TCP_INFO: u32 = 0xD8000027;

/// `WSAENOTCONN`.
const WSAENOTCONN: i32 = 10057;

/// Windows driver.
///
/// `WSAIoctl` is behind the `Win32_System_IO` feature in `windows-sys`.
/// `SIO_TCP_INFO` is unavailable on some Windows 10 builds; those return
/// `WSAEINVAL`, surfaced here as [`TcpInfoError::Io`].
pub fn fetch_metrics<S: AsRawSocket>(s: &S) -> Result<TcpMetrics, TcpInfoError> {
    let mut info: TCP_INFO_v0 = unsafe { mem::zeroed() };
    let mut bytes_ret: u32 = 0;
    let version: u32 = 0; // Request v0

    // windows-sys 0.59 resolves to a windows-targets release that declares
    // WSAIoctl and WSAGetLastError as `unsafe fn`, so an unsafe block is
    // required here. WSAIoctl's safety contract holds: the socket handle is
    // valid for the duration of the call and both buffers are sized above.
    let res = unsafe {
        WSAIoctl(
            s.as_raw_socket() as usize,
            SIO_TCP_INFO,
            &version as *const _ as *const c_void,
            mem::size_of::<u32>() as u32,
            &mut info as *mut _ as *mut c_void,
            mem::size_of::<TCP_INFO_v0>() as u32,
            &mut bytes_ret,
            std::ptr::null_mut(),
            None,
        )
    };

    if res == SOCKET_ERROR {
        // Winsock reports through its own thread-local error, not GetLastError.
        let code = unsafe { WSAGetLastError() };
        if code == WSAENOTCONN {
            return Err(TcpInfoError::NotConnected);
        }
        return Err(TcpInfoError::Io(std::io::Error::from_raw_os_error(code)));
    }

    let written = (bytes_ret as usize).min(mem::size_of::<TCP_INFO_v0>());
    if written < mem::size_of::<TCP_INFO_v0>() {
        return Err(TcpInfoError::BufferTooSmall {
            required: mem::size_of::<TCP_INFO_v0>(),
            provided: written,
        });
    }

    let slice = unsafe { std::slice::from_raw_parts(&info as *const _ as *const u8, written) };
    crate::decode::windows::decode_windows(slice)
}
