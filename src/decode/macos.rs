//! Darwin `getsockopt(IPPROTO_TCP, TCP_CONNECTION_INFO)` byte decoder.
//!
//! Layout pinned to XNU `struct tcp_connection_info`
//! (`bsd/netinet/tcp.h`). The fields used here are stable across
//! xnu-7195 through xnu main.

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
    let state = map_darwin_state(buf[0]);
    let flags = u32::from_ne_bytes(buf[8..12].try_into().unwrap());
    let maxseg = u32::from_ne_bytes(buf[16..20].try_into().unwrap());
    let snd_sbbytes = u32::from_ne_bytes(buf[32..36].try_into().unwrap());
    let srtt_ms = u32::from_ne_bytes(buf[44..48].try_into().unwrap());
    let rttvar_ms = u32::from_ne_bytes(buf[48..52].try_into().unwrap());
    let tx_retrans_bytes = u64::from_ne_bytes(buf[72..80].try_into().unwrap());
    let tx_retrans_packets = u64::from_ne_bytes(buf[104..112].try_into().unwrap());

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
