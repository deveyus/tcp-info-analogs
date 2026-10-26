//! Darwin `getsockopt(IPPROTO_TCP, TCP_CONNECTION_INFO)` byte decoder.
//!
//! Layout pinned to XNU `struct tcp_connection_info`
//! (`bsd/netinet/tcp.h`). The fields used here are stable across
//! xnu-7195 through xnu main.

use super::{read_u32, read_u64, read_u8};
use crate::{TcpInfoError, TcpMetrics, TcpState};
use std::time::Duration;

/// `TCPCI_FLAG_LOSSRECOVERY` bit from Apple XNU `bsd/netinet/tcp.h`.
pub const TCPCI_FLAG_LOSSRECOVERY: u32 = 0x00000001;

/// Map the Darwin `tcpi_state` enum to [`TcpState`].
pub fn map_darwin_state(state: u8) -> TcpState {
    match state {
        0 => TcpState::Closed,
        1 => TcpState::Listen,
        2 => TcpState::SynSent,
        3 => TcpState::SynReceived,
        4 => TcpState::Established,
        5 => TcpState::CloseWait,
        6 => TcpState::FinWait1,
        7 => TcpState::Closing,
        8 => TcpState::LastAck,
        9 => TcpState::FinWait2,
        10 => TcpState::TimeWait,
        _ => TcpState::Unknown,
    }
}

/// Decode raw bytes returned by `getsockopt(IPPROTO_TCP, TCP_CONNECTION_INFO)`.
///
/// Requires at least 112 bytes, the full `struct tcp_connection_info`.
/// RTT fields are already milliseconds; XNU divides by `TCP_RTT_SHIFT`
/// before exporting the struct.
pub fn decode_macos(buf: &[u8]) -> Result<TcpMetrics, TcpInfoError> {
    if buf.len() < 112 {
        return Err(TcpInfoError::BufferTooSmall {
            required: 112,
            provided: buf.len(),
        });
    }

    // Offsets verified against XNU struct tcp_connection_info:
    //   tcpi_state at 0
    //   tcpi_flags at 8..12
    //   tcpi_maxseg at 16..20
    //   tcpi_snd_sbbytes at 32..36
    //   tcpi_rttcur at 40..44 (deliberately ignored; not the smoothed value)
    //   tcpi_srtt at 44..48
    //   tcpi_rttvar at 48..52
    //   tcpi_tfo_* bitfield at 52..56
    //   tcpi_txpackets at 56..64
    //   tcpi_txbytes at 64..72
    //   tcpi_txretransmitbytes at 72..80
    //   tcpi_txretransmitpackets at 104..112
    let state = map_darwin_state(read_u8(buf, 0)?);
    let flags = read_u32(buf, 8)?;
    let maxseg = read_u32(buf, 16)?;
    let snd_sbbytes = read_u32(buf, 32)?;
    let srtt_ms = read_u32(buf, 44)?;
    let rttvar_ms = read_u32(buf, 48)?;
    let tx_retrans_bytes = read_u64(buf, 72)?;
    let tx_retrans_packets = read_u64(buf, 104)?;

    let is_retransmitting = (flags & TCPCI_FLAG_LOSSRECOVERY) != 0;

    Ok(TcpMetrics {
        rtt: Duration::from_millis(srtt_ms as u64),
        rtt_variance: Some(Duration::from_millis(rttvar_ms as u64)),
        loss_recovery_active: Some(is_retransmitting),
        bytes_in_flight: snd_sbbytes as u64,
        packets_in_flight: None,
        retransmit_packets: Some(tx_retrans_packets),
        retransmit_bytes: Some(tx_retrans_bytes),
        sender_mss: if maxseg > 0 { Some(maxseg) } else { None },
        state,
    })
}
