//! Pure byte fixture tests for the Windows decoder.

use std::time::Duration;
use tcp_info_analogs::decode::windows::decode_windows;
use tcp_info_analogs::{TcpInfoError, TcpState};

#[test]
fn test_windows_golden_buffer() {
    let mut buf = vec![0u8; 88];
    buf[0..4].copy_from_slice(&4i32.to_ne_bytes()); // TCPSTATE_ESTABLISHED
    buf[4..8].copy_from_slice(&1460u32.to_ne_bytes()); // Mss
    buf[20..24].copy_from_slice(&12_400u32.to_ne_bytes()); // RttUs = 12.4ms
    buf[28..32].copy_from_slice(&4380u32.to_ne_bytes()); // BytesInFlight
    buf[68..72].copy_from_slice(&2920u32.to_ne_bytes()); // BytesRetrans

    let m = decode_windows(&buf).unwrap();
    assert_eq!(m.state, TcpState::Established);
    assert_eq!(m.rtt, Duration::from_micros(12_400));
    assert_eq!(m.rtt_variance, None);
    assert_eq!(m.bytes_in_flight, 4380);
    assert_eq!(m.retransmit_bytes, Some(2920));
    assert_eq!(m.sender_mss, Some(1460));
}

#[test]
fn test_windows_short_buffer() {
    let buf = vec![0u8; 87]; // Requires 88
    assert!(matches!(
        decode_windows(&buf),
        Err(TcpInfoError::BufferTooSmall {
            required: 88,
            provided: 87
        })
    ));
}
