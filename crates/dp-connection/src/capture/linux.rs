use std::io;

const BUFFER: usize = 65_536;
const READ_TIMEOUT_MS: i32 = 500;

pub struct Capture {
    fd: libc::c_int,
    buffer: Vec<u8>,
}

impl Capture {
    /// `SOCK_DGRAM` strips the link-layer header, so packets start at their network header. `ETH_P_ALL` is
    /// needed even though only IPv4 is read: a socket bound to `ETH_P_IP` never sees outgoing packets.
    /// Both directions, on every interface, are delivered.
    pub fn open() -> io::Result<Self> {
        let protocol = (libc::ETH_P_ALL as u16).to_be();
        // SAFETY: plain syscalls with valid arguments; the descriptor is closed in Drop.
        let fd = unsafe { libc::socket(libc::AF_PACKET, libc::SOCK_DGRAM, i32::from(protocol)) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        let timeout = libc::timeval { tv_sec: 0, tv_usec: (READ_TIMEOUT_MS * 1000).into() };
        // SAFETY: `timeout` outlives the call and the size matches.
        let set = unsafe {
            libc::setsockopt(
                fd,
                libc::SOL_SOCKET,
                libc::SO_RCVTIMEO,
                &timeout as *const _ as *const libc::c_void,
                std::mem::size_of::<libc::timeval>() as libc::socklen_t,
            )
        };
        if set != 0 {
            let error = io::Error::last_os_error();
            // SAFETY: `fd` was opened above and is not used again.
            unsafe { libc::close(fd) };
            return Err(error);
        }
        Ok(Self { fd, buffer: vec![0; BUFFER] })
    }

    /// Waits up to the read timeout for one packet. The callback gets the IP packet and, where the system
    /// supplies one, its own timestamp in 100 ns units.
    pub fn read(&mut self, mut each: impl FnMut(&[u8], Option<i64>)) -> io::Result<()> {
        // SAFETY: the buffer is valid for `BUFFER` bytes.
        let n = unsafe { libc::recv(self.fd, self.buffer.as_mut_ptr().cast(), BUFFER, 0) };
        if n < 0 {
            let error = io::Error::last_os_error();
            return match error.kind() {
                io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted | io::ErrorKind::TimedOut => Ok(()),
                _ => Err(error),
            };
        }
        each(&self.buffer[..n as usize], None);
        Ok(())
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        // SAFETY: `fd` is owned by this value and closed once.
        unsafe { libc::close(self.fd) };
    }
}
