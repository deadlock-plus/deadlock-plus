use std::io;
use std::process::Command;

// ioctl numbers from <net/bpf.h>; the `libc` crate does not export them.
const BIOCGBLEN: libc::c_ulong = 0x4004_4266;
const BIOCSETIF: libc::c_ulong = 0x8020_426c;
const BIOCIMMEDIATE: libc::c_ulong = 0x8004_4270;
const BIOCGDLT: libc::c_ulong = 0x4004_426a;
const BIOCSSEESENT: libc::c_ulong = 0x8004_4277;
const BIOCSRTIMEOUT: libc::c_ulong = 0x8010_426d;

const DLT_NULL: u32 = 0;
const DLT_EN10MB: u32 = 1;
const DLT_RAW: u32 = 12;
const BPF_HEADER_LEN: usize = 18;
const READ_TIMEOUT_MS: i32 = 500;

pub struct Capture {
    fd: libc::c_int,
    buffer: Vec<u8>,
    link_header: usize,
}

/// Interface of the default route, e.g. `en0`.
fn default_interface() -> io::Result<String> {
    let output = Command::new("route").args(["-n", "get", "default"]).output()?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.trim().strip_prefix("interface:").map(|name| name.trim().to_string()))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no default network interface"))
}

fn ioctl<T>(fd: libc::c_int, request: libc::c_ulong, arg: &mut T) -> io::Result<()> {
    // SAFETY: `arg` is a valid, exclusively borrowed value of the size the request expects.
    if unsafe { libc::ioctl(fd, request, arg as *mut T) } < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// The 4-byte-aligned length of one record, header included.
fn record_len(header_len: usize, captured: usize) -> usize {
    (header_len + captured + 3) & !3
}

/// Bytes before the IP header for each link type; `None` for anything not IPv4-carrying.
fn link_header_len(dlt: u32) -> Option<usize> {
    match dlt {
        DLT_NULL => Some(4),
        DLT_EN10MB => Some(14),
        DLT_RAW => Some(0),
        _ => None,
    }
}

impl Capture {
    pub fn open() -> io::Result<Self> {
        let interface = default_interface()?;
        let mut fd = -1;
        for n in 0..256 {
            let path = std::ffi::CString::new(format!("/dev/bpf{n}")).expect("no NUL in a formatted path");
            // SAFETY: `path` is a valid C string.
            fd = unsafe { libc::open(path.as_ptr(), libc::O_RDONLY) };
            if fd >= 0 {
                break;
            }
        }
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let capture = Self::configure(fd, &interface);
        if capture.is_err() {
            // SAFETY: `fd` was opened above and is not used again.
            unsafe { libc::close(fd) };
        }
        capture
    }

    fn configure(fd: libc::c_int, interface: &str) -> io::Result<Self> {
        // SAFETY: an all-zero ifreq is a valid value.
        let mut request: libc::ifreq = unsafe { std::mem::zeroed() };
        for (slot, byte) in request.ifr_name.iter_mut().zip(interface.bytes().take(libc::IFNAMSIZ - 1)) {
            *slot = byte as libc::c_char;
        }
        ioctl(fd, BIOCSETIF, &mut request)?;
        ioctl(fd, BIOCIMMEDIATE, &mut 1u32)?;
        ioctl(fd, BIOCSSEESENT, &mut 1u32)?;
        ioctl(fd, BIOCSRTIMEOUT, &mut libc::timeval { tv_sec: 0, tv_usec: READ_TIMEOUT_MS * 1000 })?;
        let (mut length, mut dlt) = (0u32, 0u32);
        ioctl(fd, BIOCGBLEN, &mut length)?;
        ioctl(fd, BIOCGDLT, &mut dlt)?;
        let link_header = link_header_len(dlt)
            .ok_or_else(|| io::Error::new(io::ErrorKind::Unsupported, format!("unsupported link type {dlt}")))?;
        Ok(Self { fd, buffer: vec![0; length as usize], link_header })
    }

    /// Waits up to the read timeout for a batch. The callback gets each IP packet and its kernel timestamp
    /// in 100 ns units.
    pub fn read(&mut self, mut each: impl FnMut(&[u8], Option<i64>)) -> io::Result<()> {
        // SAFETY: the buffer is valid for its full length.
        let n = unsafe { libc::read(self.fd, self.buffer.as_mut_ptr().cast(), self.buffer.len()) };
        if n < 0 {
            let error = io::Error::last_os_error();
            return match error.kind() {
                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted | io::ErrorKind::TimedOut => Ok(()),
                _ => Err(error),
            };
        }
        let data = &self.buffer[..n as usize];
        let mut at = 0;
        while at + BPF_HEADER_LEN <= data.len() {
            let record = &data[at..];
            let seconds = i64::from(i32::from_ne_bytes(record[0..4].try_into().expect("4 bytes")));
            let micros = i64::from(i32::from_ne_bytes(record[4..8].try_into().expect("4 bytes")));
            let captured = u32::from_ne_bytes(record[8..12].try_into().expect("4 bytes")) as usize;
            let header_len = usize::from(u16::from_ne_bytes(record[16..18].try_into().expect("2 bytes")));
            let start = header_len + self.link_header;
            if header_len + captured > record.len() {
                break;
            }
            if start <= header_len + captured {
                each(&record[start..header_len + captured], Some(seconds * 10_000_000 + micros * 10));
            }
            at += record_len(header_len, captured);
        }
        Ok(())
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        // SAFETY: `fd` is owned by this value and closed once.
        unsafe { libc::close(self.fd) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_are_padded_to_four_bytes() {
        assert_eq!(record_len(18, 60), 80);
        assert_eq!(record_len(18, 62), 80);
        assert_eq!(record_len(18, 63), 84);
    }

    #[test]
    fn link_headers_match_the_common_interface_types() {
        assert_eq!(link_header_len(DLT_EN10MB), Some(14));
        assert_eq!(link_header_len(DLT_NULL), Some(4));
        assert_eq!(link_header_len(DLT_RAW), Some(0));
        assert_eq!(link_header_len(105), None);
    }
}
