//! Deterministic instruction-count baselines for the byte decoders
//! (`cargo bench --bench decode`).
//!
//! Counts are machine-independent, so they gate regressions without the
//! wall-clock noise that plagues timing benchmarks. Fixtures are fixed-size
//! stack arrays; the benched bodies are pure decode calls with zero heap
//! traffic, so any Ir drift or new allocation shows up as a regression.
//!
//! Requires `iai-callgrind-runner` on PATH (see scripts/bench_iai.sh) and
//! valgrind. Bench package members isolate the 1.81-minimum toolchain from
//! the library's 1.77 MSRV, so the bench crate carries its own MSRV.

#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use std::hint::black_box;

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use tcp_info_analogs::decode::linux::decode_linux;
use tcp_info_analogs::decode::macos::decode_macos;
use tcp_info_analogs::decode::windows::decode_windows;

fn fixture_linux_full() -> [u8; 104] {
    let mut buf = [0u8; 104];
    buf[0] = 1; // TCP_ESTABLISHED
    buf[1] = 0; // TCP_CA_Open
    buf[16..20].copy_from_slice(&1448u32.to_ne_bytes());
    buf[24..28].copy_from_slice(&5u32.to_ne_bytes());
    buf[68..72].copy_from_slice(&18_500u32.to_ne_bytes());
    buf[72..76].copy_from_slice(&2_100u32.to_ne_bytes());
    buf[100..104].copy_from_slice(&3u32.to_ne_bytes());
    buf
}

fn fixture_linux_min() -> [u8; 76] {
    let mut buf = [0u8; 76];
    buf[0] = 1;
    buf[1] = 4; // TCP_CA_Loss
    buf[16..20].copy_from_slice(&1448u32.to_ne_bytes());
    buf[24..28].copy_from_slice(&5u32.to_ne_bytes());
    buf[68..72].copy_from_slice(&18_500u32.to_ne_bytes());
    buf[72..76].copy_from_slice(&2_100u32.to_ne_bytes());
    buf
}

fn fixture_macos() -> [u8; 112] {
    let mut buf = [0u8; 112];
    buf[0] = 4; // Darwin TCP_ESTABLISHED
    buf[8..12].copy_from_slice(&1u32.to_ne_bytes());
    buf[16..20].copy_from_slice(&1440u32.to_ne_bytes());
    buf[32..36].copy_from_slice(&8192u32.to_ne_bytes());
    buf[44..48].copy_from_slice(&22u32.to_ne_bytes());
    buf[48..52].copy_from_slice(&4u32.to_ne_bytes());
    buf[72..80].copy_from_slice(&4320u64.to_ne_bytes());
    buf[104..112].copy_from_slice(&17u64.to_ne_bytes());
    buf
}

fn fixture_windows() -> [u8; 88] {
    let mut buf = [0u8; 88];
    buf[0..4].copy_from_slice(&4i32.to_ne_bytes());
    buf[4..8].copy_from_slice(&1460u32.to_ne_bytes());
    buf[20..24].copy_from_slice(&12_400u32.to_ne_bytes());
    buf[28..32].copy_from_slice(&4380u32.to_ne_bytes());
    buf[68..72].copy_from_slice(&2920u32.to_ne_bytes());
    buf
}

// Reduce each decoded bundle to a blackboxed field so the compiler cannot
// elide the call, and the benchmark measures only the decode path.
fn bench_output(m: &tcp_info_analogs::TcpMetrics) -> u64 {
    black_box(m.rtt.as_nanos() as u64) ^ black_box(m.bytes_in_flight)
}

#[library_benchmark]
fn decode_linux_full() -> u64 {
    bench_output(&decode_linux(&fixture_linux_full()).unwrap())
}

#[library_benchmark]
fn decode_linux_min() -> u64 {
    // Exercises the bare-minimum path where tcpi_total_retrans stays None.
    bench_output(&decode_linux(&fixture_linux_min()).unwrap())
}

#[library_benchmark]
fn decode_macos_full() -> u64 {
    bench_output(&decode_macos(&fixture_macos()).unwrap())
}

#[library_benchmark]
fn decode_windows_full() -> u64 {
    bench_output(&decode_windows(&fixture_windows()).unwrap())
}

#[library_benchmark]
fn decode_linux_short_buffer() -> u64 {
    // Error path: constructs the BufferTooSmall value without decoding.
    let short = [0u8; 75];
    match decode_linux(black_box(&short[..])) {
        Err(tcp_info_analogs::TcpInfoError::BufferTooSmall { required, provided }) => {
            black_box((required + provided) as u64)
        }
        _ => 0,
    }
}

library_benchmark_group!(
    name = decode_hot_paths;
    benchmarks =
        decode_linux_full,
        decode_linux_min,
        decode_macos_full,
        decode_windows_full,
        decode_linux_short_buffer
);

main!(library_benchmark_groups = decode_hot_paths);
