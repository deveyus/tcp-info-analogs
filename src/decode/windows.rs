//! Windows `WSAIoctl(SIO_TCP_INFO, v0)` byte decoder.
//!
//! Layout pinned to `TCP_INFO_v0` from the Windows SDK `mstcpip.h`.

use super::{read_i32, read_u32};
use crate::{TcpInfoError, TcpMetrics, TcpState};
use std::time::Duration;

/// Map the Windows `TCPSTATE` enum (0-indexed, `mstcpip.h`) to [`TcpState`].
pub fn map_windows_tcpstate(state: i32) -> TcpState {
    match state {
        0 => TcpState::Closed,
        1 => TcpState::Listen,
        2 => TcpState::SynSent,
        3 => TcpState::SynReceived,
        4 => TcpState::Established,
        5 => TcpState::FinWait1,
        6 => TcpState::FinWait2,
        7 => TcpState::CloseWait,
        8 => TcpState::Closing,
        9 => TcpState::LastAck,
        10 => TcpState::TimeWait,
        _ => TcpState::Unknown,
    }
}

/// Decode raw bytes returned by `WSAIoctl(SIO_TCP_INFO, v0)`.
///
/// Requires at least 88 bytes matching `TCP_INFO_v0`.
pub fn decode_windows(buf: &[u8]) -> Result<TcpMetrics, TcpInfoError> {
    if buf.len() < 88 {
        return Err(TcpInfoError::BufferTooSmall {
            required: 88,
            provided: buf.len(),
        });
    }

    // Offsets verified against Windows SDK TCP_INFO_v0:
    //   State (i32) at 0..4
    //   Mss at 4..8
    //   RttUs at 20..24
    //   BytesInFlight at 28..32
    //   BytesRetrans at 68..72
    let state_raw = read_i32(buf, 0)?;
    let mss = read_u32(buf, 4)?;
    let rtt_us = read_u32(buf, 20)?;
    let bytes_in_flight = read_u32(buf, 28)?;
    let bytes_retrans = read_u32(buf, 68)?;

    Ok(TcpMetrics {
        rtt: Duration::from_micros(rtt_us as u64),
        rtt_variance: None, // TCP_INFO_v0 does not expose variance
        loss_recovery_active: None,
        bytes_in_flight: bytes_in_flight as u64,
        packets_in_flight: None,
        retransmit_packets: None,
        retransmit_bytes: Some(bytes_retrans as u64),
        sender_mss: if mss > 0 { Some(mss) } else { None },
        state: map_windows_tcpstate(state_raw),
    })
}
