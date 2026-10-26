// Example: scratch demo; expect() on setup failures is the idiomatic style
// for throwaway samples, opposed to the library's strict posture.
#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Read TCP_INFO in a hot loop on loopback and report samples per second.
//!
//! Run with `cargo run --example poll_rate`. The measured ceiling on loopback
//! is the getsockopt round trip; the pure-decode floor is covered separately
//! by `scripts/bench_iai.sh` (sub-100 Ir per sample).

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use std::time::Instant;

use tcp_info_analogs::TcpInfoExt;

fn main() {
    let seconds = Duration::from_secs(2);
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback listener");
    let addr = listener.local_addr().expect("peer address");

    let mut client = TcpStream::connect(addr).expect("client connect");
    let (mut server, _) = listener.accept().expect("server accept");

    // Push a few events through so the connection carries real traffic.
    let payload = [7u8; 64];
    server.write_all(&payload).expect("write payload");
    let mut sink = [0u8; 64];
    let _ = client.read(&mut sink); // No-op if the socket remains open (server stays up).

    let mut samples = 0u64;
    let mut error_count = 0u32;

    let start = Instant::now();
    while start.elapsed() < seconds {
        match client.tcp_metrics() {
            Ok(_metrics) => samples += 1,
            Err(_err) => error_count += 1,
        }
    }

    let elapsed = start.elapsed();
    let elapsed_ns = elapsed.as_nanos() as u64;
    println!(
        "samples/s    : {:.0}",
        samples as f64 * 1e9 / elapsed_ns as f64
    );
    println!("avg per sample : {} ns", elapsed_ns / samples.max(1));
    println!("sample count   : {samples} spanning {seconds:?}");
    println!("errors         : {error_count}");
}
