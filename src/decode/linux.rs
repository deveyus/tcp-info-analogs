//! Linux `getsockopt(SOL_TCP, TCP_INFO)` byte decoder.

use super::{read_u32, read_u8};
use crate::{TcpInfoError, TcpMetrics, TcpState};
use std::time::Duration;

/// Map the Linux kernel `tcpi_state` enum to [`TcpState`].
pub fn map_linux_state(state: u8) -> TcpState {
    match state {
        1 => TcpState::Established,
        2 => TcpState::SynSent,
        3 => TcpState::SynReceived,
        4 => TcpState::FinWait1,
        5 => TcpState::FinWait2,
        6 => TcpState::TimeWait,
        7 => TcpState::Closed,
        8 => TcpState::CloseWait,
        9 => TcpState::LastAck,
        10 => TcpState::Listen,
        11 => TcpState::Closing,
        _ => TcpState::Unknown,
    }
}

/// Decode raw bytes returned by `getsockopt(SOL_TCP, TCP_INFO)`.
///
/// Requires at least 76 bytes to extract valid RTT and jitter fields.
/// All multi-byte fields use native endianness, matching the kernel ABI.
pub fn decode_linux(buf: &[u8]) -> Result<TcpMetrics, TcpInfoError> {
    if buf.len() < 76 {
        return Err(TcpInfoError::BufferTooSmall {
            required: 76,
            provided: buf.len(),
        });
    }

    let state = map_linux_state(read_u8(buf, 0)?);
    let ca_state = read_u8(buf, 1)?;

    // Offsets verified against uapi/linux/tcp.h:
    //   tcpi_snd_mss at 16..20
    //   tcpi_unacked at 24..28
    //   tcpi_rtt at 68..72
    //   tcpi_rttvar at 72..76
    //   tcpi_total_retrans at 100..104
    let snd_mss = read_u32(buf, 16)?;
    let unacked = read_u32(buf, 24)?;
    let rtt_us = read_u32(buf, 68)?;
    let rttvar_us = read_u32(buf, 72)?;

    let total_retrans = if buf.len() >= 104 {
        Some(read_u32(buf, 100)? as u64)
    } else {
        None
    };

    Ok(TcpMetrics {
        rtt: Duration::from_micros(rtt_us as u64),
        rtt_variance: Some(Duration::from_micros(rttvar_us as u64)),
        loss_recovery_active: Some(ca_state >= 3), // TCP_CA_Recovery (3) or TCP_CA_Loss (4)
        bytes_in_flight: (unacked as u64) * (snd_mss as u64),
        packets_in_flight: Some(unacked),
        retransmit_packets: total_retrans,
        retransmit_bytes: None,
        sender_mss: if snd_mss > 0 { Some(snd_mss) } else { None },
        state,
    })
}
