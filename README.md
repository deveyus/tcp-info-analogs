# tcp-info-analogs

Zero-allocation, cross-platform normalized TCP connection health and
congestion metrics for Linux, macOS, and Windows.

Query SRTT, RTT variance, loss recovery, bytes/packets in flight,
retransmit counts, sender MSS, and TCP state through one extension
trait, on any of `std::net::TcpStream`, `socket2::Socket`, or
`tokio::net::TcpStream`.

```rust
use tcp_info_analogs::TcpInfoExt;

let metrics = std::net::TcpStream::connect("example.com:443")?.tcp_metrics()?;
println!("srtt = {:?}", metrics.rtt);
```

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
carry the divergence warning inline. Unsupported Unix targets compile
and return `TcpInfoError::UnsupportedPlatform` at call time.

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

Instruction-count baselines (`scripts/bench_iai.sh`, iai-callgrind) gate
decode-path regressions in CI. The `benches/` workspace member is
documented internally; the 1.77 MSRV gate is scoped to the library and
tests because the bench chain requires a newer toolchain.

## MSRV and CI

MSRV is 1.77 (`offset_of!` era) and enforced with a committed
`Cargo.lock` and `--locked`. CI (`.github/workflows/ci.yml`) runs the
test matrix on all three OSes, clippy with `-D warnings`, rustfmt,
the MSRV gate, and the iai instruction-count job.

## Roadmap

Tracked as issues: batched sampling via `io_uring` (`GETSOCKOPT`),
netlink `inet_diag` fleet snapshots, and BPF-iterator zero-syscall
polling. See the issue tracker for the current state.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
