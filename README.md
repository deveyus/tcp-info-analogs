# tcp-info-analogs

Zero-allocation normalized TCP connection health and congestion metrics for
Linux, macOS, and Windows.

Query `SRTT`, RTT variance, loss recovery, bytes/packets in flight,
retransmit counts, sender MSS, and TCP state through one extension trait,
regardless of platform ABI.

```rust
use tcp_info_analogs::TcpInfoExt;

let metrics = std::net::TcpStream::connect("example.com:443")?.tcp_metrics()?;
println!("srtt = {:?}", metrics.rtt);
```

## Platform notes

| Metric                | Linux                    | macOS                          | Windows        |
| --------------------- | ------------------------ | ------------------------------ | -------------- |
| `rtt`                 | µs (`tcpi_rtt`)          | ms (`tcpi_srtt`)               | µs (`RttUs`)   |
| `rtt_variance`        | µs (`tcpi_rttvar`)       | ms (`tcpi_rttvar`)             | not exposed    |
| `bytes_in_flight`     | coarse (unacked × MSS)   | socket-buffer occupancy        | native wire    |
| `retransmit_packets`  | lifetime (`tcpi_total_retrans`) | lifetime (`tcpi_txretransmitpackets`) | not exposed |
| `retransmit_bytes`    | not exposed              | lifetime (`tcpi_txretransmitbytes`)   | wraps at u32::MAX |

## Features

- `socket2`: `TcpInfoExt for socket2::Socket`
- `tokio`: `TcpInfoExt for tokio::net::TcpStream`

Unsupported Unix targets compile and return
`TcpInfoError::UnsupportedPlatform` at call time.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
