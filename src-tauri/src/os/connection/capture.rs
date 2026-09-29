//! Reads IPv4 packets off the network with root rights: an `AF_PACKET` socket on Linux, a BPF device on macOS.

use std::net::Ipv4Addr;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
use std::io;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::Capture;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::Capture;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub struct Capture;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
impl Capture {
    pub fn open() -> io::Result<Self> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "packet capture is not implemented for this system"))
    }

    pub fn read(&mut self, _each: impl FnMut(&[u8], Option<i64>)) -> io::Result<()> {
        Ok(())
    }
}

/// Every IPv4 address on this machine's interfaces.
pub fn local_addresses() -> Vec<Ipv4Addr> {
    let mut found = Vec::new();
    let mut list: *mut libc::ifaddrs = std::ptr::null_mut();
    // SAFETY: getifaddrs fills `list` with a linked list we only read, then free with freeifaddrs.
    unsafe {
        if libc::getifaddrs(&mut list) != 0 {
            return found;
        }
        let mut node = list;
        while !node.is_null() {
            let addr = (*node).ifa_addr;
            if !addr.is_null() && i32::from((*addr).sa_family) == libc::AF_INET {
                let v4 = &*(addr as *const libc::sockaddr_in);
                found.push(Ipv4Addr::from(u32::from_be(v4.sin_addr.s_addr)));
            }
            node = (*node).ifa_next;
        }
        libc::freeifaddrs(list);
    }
    found
}
