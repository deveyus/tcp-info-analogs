// Test suite: the loopback fixture is known-good; unwrap() asserts setup
// outcomes directly.
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Loopback syscall integration test; runs on every supported host.

#![cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]

use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use tcp_info_analogs::{TcpInfoExt, TcpState};

#[test]
fn loopback_established_metrics() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let client = TcpStream::connect(addr).unwrap();
    let (_server, _) = listener.accept().unwrap();

    let m = client.tcp_metrics().unwrap();
    assert_eq!(m.state, TcpState::Established);
    assert!(m.rtt >= Duration::ZERO);
    assert!(m.sender_mss.is_some());
}
