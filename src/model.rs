//! Normalized TCP connection state and metric types.

use std::time::Duration;

/// TCP connection lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TcpState {
    /// Connection is inactive or uninitialized.
    Closed,
    /// Listening for incoming connection requests.
    Listen,
    /// SYN packet sent; awaiting SYN-ACK.
    SynSent,
    /// SYN received; awaiting ACK.
    SynReceived,
    /// Normal, open data-transfer state.
    Established,
    /// Local side initiated close; FIN-1 sent.
    FinWait1,
    /// Remote ACK of FIN received; awaiting remote FIN.
    FinWait2,
    /// Remote initiated close; local awaiting close.
    CloseWait,
    /// Both sides closed concurrently.
    Closing,
    /// Local sent final FIN; awaiting final ACK.
    LastAck,
    /// Waiting for enough time to pass to ensure the remote received the ACK of its close.
    TimeWait,
    /// Operating system reported an unknown or unrecognized state code.
    Unknown,
}

/// Normalized TCP connection operational metrics.
///
/// Constructed only by this crate; downstream code reads the fields.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TcpMetrics {
    /// Smoothed Round Trip Time (SRTT).
    pub rtt: Duration,

    /// RTT Variance (jitter).
    /// - Linux: `Some(from_micros(tcpi_rttvar))`
    /// - macOS: `Some(from_millis(tcpi_rttvar))`
    /// - Windows: `None` (TCP_INFO_v0 does not expose variance)
    pub rtt_variance: Option<Duration>,

    /// Instantaneous loss-recovery flag.
    /// - Linux: `Some(ca_state >= TCP_CA_Recovery)`
    /// - macOS: `Some(tcpi_flags & TCPCI_FLAG_LOSSRECOVERY != 0)`
    /// - Windows: `None` (TCP_INFO_v0 does not expose an instantaneous recovery bit)
    pub loss_recovery_active: Option<bool>,

    /// Data currently in flight (unacknowledged) in BYTES.
    ///
    /// ## Platform Divergence Warning
    /// - **Linux:** Coarse estimate calculated as `tcpi_unacked * tcpi_snd_mss`.
    ///   Ignores SACK holes, lost packet markers, and sub-MSS deliveries.
    /// - **macOS:** Reports send-buffer occupancy via `tcpi_snd_sbbytes`.
    ///   Measures buffered unsent plus unacked data, not on-the-wire transit alone.
    /// - **Windows:** Native wire measurement via `BytesInFlight`.
    pub bytes_in_flight: u64,

    /// Unacknowledged packets in flight.
    /// Always populated on Linux (`tcpi_unacked`); `None` on macOS and Windows.
    pub packets_in_flight: Option<u32>,

    /// Lifetime cumulative retransmitted packets.
    /// Populated on Linux (`tcpi_total_retrans`) and macOS
    /// (`tcpi_txretransmitpackets`); `None` on Windows v0.
    /// Note: Wraps at `u32::MAX` on long-lived connections.
    pub retransmit_packets: Option<u64>,

    /// Lifetime cumulative retransmitted bytes (macOS and Windows only).
    /// Note: Wraps at `u32::MAX` on Windows v0.
    pub retransmit_bytes: Option<u64>,

    /// Sender Maximum Segment Size (MSS) in bytes.
    pub sender_mss: Option<u32>,

    /// TCP finite state machine state.
    pub state: TcpState,
}
