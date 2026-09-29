use std::net::Ipv4Addr;

use windows::Win32::NetworkManagement::IpHelper::{IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho, ICMP_ECHO_REPLY};

const PAYLOAD: [u8; 32] = [0u8; 32];

/// Native ICMP echo via IP Helper: no child process, no console window, cheap enough for 1 Hz sampling.
/// Resolution is whole milliseconds, which is plenty for WAN relays.
pub fn ping(ip: Ipv4Addr, timeout_ms: u32) -> Option<f32> {
    unsafe {
        let handle = IcmpCreateFile().ok()?;
        // u64 elements keep the buffer 8-byte aligned, which ICMP_ECHO_REPLY requires.
        let mut reply = [0u64; 32];
        debug_assert!(std::mem::size_of_val(&reply) >= std::mem::size_of::<ICMP_ECHO_REPLY>() + PAYLOAD.len() + 8);

        let received = IcmpSendEcho(
            handle,
            u32::from_ne_bytes(ip.octets()),
            PAYLOAD.as_ptr().cast(),
            PAYLOAD.len() as u16,
            None,
            reply.as_mut_ptr().cast(),
            std::mem::size_of_val(&reply) as u32,
            timeout_ms,
        );

        let rtt = if received > 0 {
            let echo = &*reply.as_ptr().cast::<ICMP_ECHO_REPLY>();
            (echo.Status == 0).then_some(echo.RoundTripTime as f32)
        } else {
            None
        };

        let _ = IcmpCloseHandle(handle);
        rtt
    }
}
