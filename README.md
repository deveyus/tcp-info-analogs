# tcp-info-analogs

[![docs.rs](https://img.shields.io/docsrs/tcp-info-analogs)](https://docs.rs/tcp-info-analogs)
[![CI](https://github.com/deveyus/tcp-info-analogs/actions/workflows/ci.yml/badge.svg)](https://github.com/deveyus/tcp-info-analogs/actions/workflows/ci.yml)
![MSRV 1.77](https://img.shields.io/badge/MSRV-1.77-orange)

The portable analog of Linux's `TCP_INFO`: TCP connection health and
congestion metrics — SRTT, RTT variance, loss recovery, bytes and packets
in flight, retransmit counts, sender MSS, and TCP state — through one
extension trait on Linux, macOS, and Windows.

Cross-platform `TCP_INFO` is the problem nobody wants to hand-roll:
Linux exposes `getsockopt(TCP_INFO)`, macOS a different option code
backed by an XNU-only struct, and Windows an entirely different
`WSAIoctl` payload, each with different field availability and units.
This crate does the syscall dance per platform, decodes the raw kernel
bytes with verified offsets, and normalizes what overlaps — then
documents honestly what does not.

```rust
use tcp_info_analogs::TcpInfoExt;

let metrics = std::net::TcpStream::connect("example.com:443")?.tcp_metrics()?;
println!("srtt = {:?}", metrics.rtt);
```

Works on `std::net::TcpStream`, plus `socket2::Socket` and
`tokio::net::TcpStream` behind features. Unsupported Unix targets
compile and return `TcpInfoError::UnsupportedPlatform` at call time.


## Design

- **Byte-slice decoders.** Each platform's kernel reply is decoded from
  raw bytes with checked readers (no panics possible), pinned to verified
  offsets from the actual kernel ABIs:

  | Platform | Syscall | Layout |
  | --- | --- | --- |
  | Linux | `getsockopt(SOL_TCP, TCP_INFO)` | `uapi/linux/tcp.h`, 76-byte minimum |
  | macOS | `getsockopt(IPPROTO_TCP, TCP_CONNECTION_INFO)` | XNU `tcp_connection_info`, 112 bytes |
  | Windows | `WSAIoctl(SIO_TCP_INFO)` | SDK `TCP_INFO_v0`, 88 bytes |

  Offsets are asserted at compile time with `offset_of!` against each
  platform's real binding on its native host, plus portable mirrors so
  every decoder's fixture tests run on every host in CI.

- **Zero heap allocations.** The syscall path uses stack buffers only;
  `TcpMetrics` is pure `Copy` data. Verified by instruction-count
  benchmarks: full decode is 66-93 Ir (`benches/`), buffer-to-error is 38
  Ir. A raw getsockopt round trip (~2 µs on loopback) dominates the per
  sample cost, so the library adds ~1% over the bare syscall: about 480k
  samples/sec sustained on loopback (`cargo run --example poll_rate`).

- **API evolution guards.** `TcpMetrics` and `TcpInfoError` are
  `#[non_exhaustive]`. In-flight unknown kernel states map to
  `TcpState::Unknown` rather than failing forward.

## Platform notes

| Metric | Linux | macOS | Windows |
| --- | --- | --- | --- |
| `rtt` | µs (`tcpi_rtt`) | ms (`tcpi_srtt`) | µs (`RttUs`) |
| `rtt_variance` | µs (`tcpi_rttvar`) | ms (`tcpi_rttvar`) | not exposed |
| `bytes_in_flight` | coarse (unacked × MSS) | socket-buffer occupancy | native wire |
| `retransmit_packets` | lifetime | lifetime | not exposed |
| `retransmit_bytes` | not exposed | lifetime | wraps at `u32::MAX` |

`bytes_in_flight` means different things per platform; the metric docs
carry the divergence warning inline.

## Features

- `socket2`: `TcpInfoExt for socket2::Socket`
- `tokio`: `TcpInfoExt for tokio::net::TcpStream`

## Performance

```text
$ cargo run --release --example poll_rate
samples/s    : 478358
avg per sample : 2090 ns
sample count   : 956717 spanning 2s
errors         : 0
```

(Measured on one host; treat as shape, not a promise. The ceil value is
the getsockopt round trip, not the crate.)

Instruction-count baselines (`scripts/bench_iai.sh`, iai-callgrind) gate
decode-path regressions in CI. The `benches/` workspace member is
documented internally; the 1.77 MSRV gate is scoped to the library and
tests because the bench chain requires a newer toolchain.

## MSRV and CI

MSRV is 1.77 (`offset_of!` era) and enforced with a committed
`Cargo.lock` and `--locked`. CI runs the test matrix on all three OSes,
clippy with `-D warnings`, rustfmt, the MSRV gate, and the iai
instruction-count job:
[.github/workflows/ci.yml](.github/workflows/ci.yml).

## Roadmap

Tracked as issues: batched sampling via `io_uring` (`GETSOCKOPT`),
netlink `inet_diag` fleet snapshots, and BPF-iterator zero-syscall
polling. See the issue tracker for the current state.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
