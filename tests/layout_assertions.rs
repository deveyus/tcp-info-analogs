//! Compile-time layout assertions against portable mirrors and, when run on
//! the platform itself, the real bindings.

use core::mem::{align_of, offset_of, size_of};

// Native Windows C-layout mirror for cross-platform layout verification.
#[repr(C)]
struct WindowsTcpInfoV0Mirror {
    state: i32,
    mss: u32,
    connection_time_ms: u64,
    timestamps_enabled: u8,
    _pad1: [u8; 3],
    rtt_us: u32,
    min_rtt_us: u32,
    bytes_in_flight: u32,
    cwnd: u32,
    snd_wnd: u32,
    rcv_wnd: u32,
    rcv_buf: u32,
    bytes_out: u64,
    bytes_in: u64,
    bytes_reordered: u32,
    bytes_retrans: u32,
    fast_retrans: u32,
    dup_acks_in: u32,
    timeout_episodes: u32,
    syn_retrans: u8,
    _pad2: [u8; 3],
}

// Native Darwin C-layout mirror for cross-platform layout verification.
#[repr(C)]
struct DarwinTcpConnectionInfoMirror {
    state: u8,
    snd_wscale: u8,
    rcv_wscale: u8,
    _pad1: u8,
    options: u32,
    flags: u32,
    rto: u32,
    maxseg: u32,
    snd_ssthresh: u32,
    snd_cwnd: u32,
    snd_wnd: u32,
    snd_sbbytes: u32,
    rcv_wnd: u32,
    rttcur: u32,
    srtt: u32,
    rttvar: u32,
    tfo_flags: u32,
    txpackets: u64,
    txbytes: u64,
    txretransmitbytes: u64,
    rxpackets: u64,
    rxbytes: u64,
    rxoutoforderbytes: u64,
    txretransmitpackets: u64,
}

#[test]
fn assert_windows_v0_layout() {
    assert_eq!(size_of::<WindowsTcpInfoV0Mirror>(), 88);
    if cfg!(target_pointer_width = "64") {
        assert_eq!(align_of::<WindowsTcpInfoV0Mirror>(), 8);
    }

    assert_eq!(offset_of!(WindowsTcpInfoV0Mirror, state), 0);
    assert_eq!(offset_of!(WindowsTcpInfoV0Mirror, mss), 4);
    assert_eq!(offset_of!(WindowsTcpInfoV0Mirror, connection_time_ms), 8);
    assert_eq!(offset_of!(WindowsTcpInfoV0Mirror, timestamps_enabled), 16);
    assert_eq!(offset_of!(WindowsTcpInfoV0Mirror, rtt_us), 20);
    assert_eq!(offset_of!(WindowsTcpInfoV0Mirror, min_rtt_us), 24);
    assert_eq!(offset_of!(WindowsTcpInfoV0Mirror, bytes_in_flight), 28);
    assert_eq!(offset_of!(WindowsTcpInfoV0Mirror, bytes_retrans), 68);
    assert_eq!(offset_of!(WindowsTcpInfoV0Mirror, timeout_episodes), 80);
    assert_eq!(offset_of!(WindowsTcpInfoV0Mirror, syn_retrans), 84);
}

#[test]
fn assert_darwin_connection_info_layout() {
    assert_eq!(size_of::<DarwinTcpConnectionInfoMirror>(), 112);
    if cfg!(target_pointer_width = "64") {
        assert_eq!(align_of::<DarwinTcpConnectionInfoMirror>(), 8);
    }

    assert_eq!(offset_of!(DarwinTcpConnectionInfoMirror, state), 0);
    assert_eq!(offset_of!(DarwinTcpConnectionInfoMirror, flags), 8);
    assert_eq!(offset_of!(DarwinTcpConnectionInfoMirror, maxseg), 16);
    assert_eq!(offset_of!(DarwinTcpConnectionInfoMirror, snd_sbbytes), 32);
    assert_eq!(offset_of!(DarwinTcpConnectionInfoMirror, rttcur), 40);
    assert_eq!(offset_of!(DarwinTcpConnectionInfoMirror, srtt), 44);
    assert_eq!(offset_of!(DarwinTcpConnectionInfoMirror, rttvar), 48);
    assert_eq!(offset_of!(DarwinTcpConnectionInfoMirror, tfo_flags), 52);
    assert_eq!(offset_of!(DarwinTcpConnectionInfoMirror, txpackets), 56);
    assert_eq!(offset_of!(DarwinTcpConnectionInfoMirror, txbytes), 64);
    assert_eq!(
        offset_of!(DarwinTcpConnectionInfoMirror, txretransmitbytes),
        72
    );
    assert_eq!(
        offset_of!(DarwinTcpConnectionInfoMirror, txretransmitpackets),
        104
    );
}

// On Linux, assert against the real binding.
#[cfg(target_os = "linux")]
#[test]
fn assert_linux_layout_matches_binding() {
    use libc::tcp_info;

    assert_eq!(offset_of!(tcp_info, tcpi_state), 0);
    assert_eq!(offset_of!(tcp_info, tcpi_ca_state), 1);
    assert_eq!(offset_of!(tcp_info, tcpi_snd_mss), 16);
    assert_eq!(offset_of!(tcp_info, tcpi_unacked), 24);
    assert_eq!(offset_of!(tcp_info, tcpi_rtt), 68);
    assert_eq!(offset_of!(tcp_info, tcpi_rttvar), 72);
    assert_eq!(offset_of!(tcp_info, tcpi_total_retrans), 100);
}

// On Windows, assert against the real binding, not the portable mirror.
#[cfg(target_os = "windows")]
#[test]
fn assert_windows_layout_matches_binding() {
    use windows_sys::Win32::Networking::WinSock::TCP_INFO_v0;

    assert_eq!(size_of::<TCP_INFO_v0>(), 88);
    assert_eq!(offset_of!(TCP_INFO_v0, State), 0);
    assert_eq!(offset_of!(TCP_INFO_v0, Mss), 4);
    assert_eq!(offset_of!(TCP_INFO_v0, ConnectionTimeMs), 8);
    assert_eq!(offset_of!(TCP_INFO_v0, TimestampsEnabled), 16);
    assert_eq!(offset_of!(TCP_INFO_v0, RttUs), 20);
    assert_eq!(offset_of!(TCP_INFO_v0, MinRttUs), 24);
    assert_eq!(offset_of!(TCP_INFO_v0, BytesInFlight), 28);
    assert_eq!(offset_of!(TCP_INFO_v0, BytesRetrans), 68);
    assert_eq!(offset_of!(TCP_INFO_v0, TimeoutEpisodes), 80);
    assert_eq!(offset_of!(TCP_INFO_v0, SynRetrans), 84);
}

// On macOS, assert the pre-TFO fields against the real binding.
// Fields after the TFO area are not asserted: the libc binding expands the
// kernel's single u32 bitfield into fifteen u32 fields.
#[cfg(target_os = "macos")]
#[test]
fn assert_darwin_layout_matches_binding() {
    use libc::tcp_connection_info;

    assert_eq!(offset_of!(tcp_connection_info, tcpi_state), 0);
    assert_eq!(offset_of!(tcp_connection_info, tcpi_flags), 8);
    assert_eq!(offset_of!(tcp_connection_info, tcpi_maxseg), 16);
    assert_eq!(offset_of!(tcp_connection_info, tcpi_snd_sbbytes), 32);
    assert_eq!(offset_of!(tcp_connection_info, tcpi_rttcur), 40);
    assert_eq!(offset_of!(tcp_connection_info, tcpi_srtt), 44);
    assert_eq!(offset_of!(tcp_connection_info, tcpi_rttvar), 48);
}
