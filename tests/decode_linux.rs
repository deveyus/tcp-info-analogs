//! Pure byte fixture tests for the Linux decoder.

use std::time::Duration;
use tcp_info_analogs::decode::linux::decode_linux;
use tcp_info_analogs::{TcpInfoError, TcpState};

#[test]
fn test_linux_golden_buffer() {
    let mut buf = vec![0u8; 104];
    buf[0] = 1; // TCP_ESTABLISHED
    buf[1] = 0; // TCP_CA_Open
    buf[16..20].copy_from_slice(&1448u32.to_ne_bytes()); // snd_mss
    buf[24..28].copy_from_slice(&5u32.to_ne_bytes()); // unacked pkts
    buf[68..72].copy_from_slice(&18_500u32.to_ne_bytes()); // rtt = 18.5ms
    buf[72..76].copy_from_slice(&2_100u32.to_ne_bytes()); // rttvar = 2.1ms
    buf[100..104].copy_from_slice(&3u32.to_ne_bytes()); // total_retrans = 3

    let m = decode_linux(&buf).unwrap();
    assert_eq!(m.state, TcpState::Established);
    assert_eq!(m.rtt, Duration::from_micros(18_500));
    assert_eq!(m.rtt_variance, Some(Duration::from_micros(2_100)));
    assert_eq!(m.sender_mss, Some(1448));
    assert_eq!(m.packets_in_flight, Some(5));
    assert_eq!(m.bytes_in_flight, 5 * 1448);
    assert_eq!(m.retransmit_packets, Some(3));
    assert_eq!(m.loss_recovery_active, Some(false));
}

#[test]
fn test_linux_loss_recovery_flag() {
    let mut buf = vec![0u8; 76];
    buf[0] = 1;
    buf[1] = 4; // TCP_CA_Loss
    assert_eq!(decode_linux(&buf).unwrap().loss_recovery_active, Some(true));
}

#[test]
fn test_linux_short_buffer() {
    let buf = vec![0u8; 75]; // Missing the final byte of rttvar (requires 76)
    assert!(matches!(
        decode_linux(&buf),
        Err(TcpInfoError::BufferTooSmall {
            required: 76,
            provided: 75
        })
    ));
}
