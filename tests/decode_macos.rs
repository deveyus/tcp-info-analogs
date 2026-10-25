//! Pure byte fixture tests for the macOS decoder.

use std::time::Duration;
use tcp_info_analogs::decode::macos::decode_macos;
use tcp_info_analogs::{TcpInfoError, TcpState};

#[test]
fn test_macos_golden_buffer() {
    let mut buf = vec![0u8; 112];
    buf[0] = 4; // Darwin TCP_ESTABLISHED
    buf[8..12].copy_from_slice(&1u32.to_ne_bytes()); // flags = TCPCI_FLAG_LOSSRECOVERY
    buf[16..20].copy_from_slice(&1440u32.to_ne_bytes()); // maxseg
    buf[32..36].copy_from_slice(&8192u32.to_ne_bytes()); // snd_sbbytes
    buf[40..44].copy_from_slice(&999u32.to_ne_bytes()); // rttcur (must be ignored)
    buf[44..48].copy_from_slice(&22u32.to_ne_bytes()); // srtt = 22ms
    buf[48..52].copy_from_slice(&4u32.to_ne_bytes()); // rttvar = 4ms
    buf[72..80].copy_from_slice(&4320u64.to_ne_bytes()); // txretransmitbytes = 4320
    buf[104..112].copy_from_slice(&17u64.to_ne_bytes()); // txretransmitpackets = 17

    let m = decode_macos(&buf).unwrap();
    assert_eq!(m.state, TcpState::Established);
    assert_eq!(m.rtt, Duration::from_millis(22));
    assert_eq!(m.rtt_variance, Some(Duration::from_millis(4)));
    assert_eq!(m.bytes_in_flight, 8192);
    assert_eq!(m.retransmit_bytes, Some(4320));
    assert_eq!(m.retransmit_packets, Some(17));
    assert_eq!(m.sender_mss, Some(1440));
    assert_eq!(m.loss_recovery_active, Some(true));
}

#[test]
fn test_macos_short_buffer() {
    let buf = vec![0u8; 111]; // Requires 112
    assert!(matches!(
        decode_macos(&buf),
        Err(TcpInfoError::BufferTooSmall {
            required: 112,
            provided: 111
        })
    ));
}
