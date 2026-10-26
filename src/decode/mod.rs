//! Unconditionally compiled decoders.
//!
//! These parse raw byte buffers returned by the platform syscalls into
//! normalized [`crate::TcpMetrics`].
//!
//! Every field read goes through the checked [`read_*`] helpers below, so a
//! decoder can never panic, no matter what byte counts the kernel reports.

pub mod linux;
pub mod macos;
pub mod windows;

use crate::TcpInfoError;

/// Borrow `count` bytes at `off`, reporting [`TcpInfoError::BufferTooSmall`]
/// if any part of the range is beyond the buffer.
fn read_chunk(buf: &[u8], off: usize, count: usize) -> Result<&[u8], TcpInfoError> {
    match buf.get(off..off + count) {
        Some(chunk) => Ok(chunk),
        None => Err(TcpInfoError::BufferTooSmall {
            required: off + count,
            provided: buf.len(),
        }),
    }
}

/// Read one byte at `off`.
pub(crate) fn read_u8(buf: &[u8], off: usize) -> Result<u8, TcpInfoError> {
    match read_chunk(buf, off, 1)?.first() {
        Some(&byte) => Ok(byte),
        None => Err(TcpInfoError::BufferTooSmall {
            required: off + 1,
            provided: buf.len(),
        }),
    }
}

/// Read a native-endian `u32` at `off`.
pub(crate) fn read_u32(buf: &[u8], off: usize) -> Result<u32, TcpInfoError> {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(read_chunk(buf, off, 4)?);
    Ok(u32::from_ne_bytes(bytes))
}

/// Read a native-endian `i32` at `off`.
pub(crate) fn read_i32(buf: &[u8], off: usize) -> Result<i32, TcpInfoError> {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(read_chunk(buf, off, 4)?);
    Ok(i32::from_ne_bytes(bytes))
}

/// Read a native-endian `u64` at `off`.
pub(crate) fn read_u64(buf: &[u8], off: usize) -> Result<u64, TcpInfoError> {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(read_chunk(buf, off, 8)?);
    Ok(u64::from_ne_bytes(bytes))
}
