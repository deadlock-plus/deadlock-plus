use std::io;

use crate::policy;

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
        let attached = if set == 0 { attach_filter(fd) } else { Err(io::Error::last_os_error()) };
        if let Err(error) = attached {
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

/// Keeps everything but IPv4 UDP out of the socket, and truncates what remains to the headers that are parsed.
fn attach_filter(fd: libc::c_int) -> io::Result<()> {
    let mut program: Vec<libc::sock_filter> = policy::ipv4_udp_filter()
        .iter()
        .map(|i| libc::sock_filter { code: i.code, jt: i.jt, jf: i.jf, k: i.k })
        .collect();
    let fprog = libc::sock_fprog { len: program.len() as u16, filter: program.as_mut_ptr() };
    // SAFETY: `fprog` points at `program`, which outlives the call; the kernel copies the instructions.
    let rc = unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_ATTACH_FILTER,
            &fprog as *const _ as *const libc::c_void,
            std::mem::size_of::<libc::sock_fprog>() as libc::socklen_t,
        )
    };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Gives up root for good once the raw socket is open; an already open socket keeps working. The new identity
/// is not the invoking user's, so a process of that user cannot attach to the helper and take the socket.
pub fn drop_privileges() -> io::Result<()> {
    // SAFETY: geteuid has no preconditions.
    if !policy::should_drop_privileges(unsafe { libc::geteuid() }) {
        return Ok(());
    }
    let id = policy::UNPRIVILEGED_ID;
    // SAFETY: plain syscalls with valid arguments; a null list with length 0 clears the supplementary groups.
    let dropped = unsafe {
        libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) == 0
            && libc::setgroups(0, std::ptr::null()) == 0
            && libc::setresgid(id, id, id) == 0
            && libc::setresuid(id, id, id) == 0
            && libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) == 0
    };
    if !dropped {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: setuid has no memory preconditions; success here means root could be regained.
    if unsafe { libc::setuid(0) } == 0 {
        return Err(io::Error::other("root could not be dropped"));
    }
    Ok(())
}

impl Drop for Capture {
    fn drop(&mut self) {
        // SAFETY: `fd` is owned by this value and closed once.
        unsafe { libc::close(self.fd) };
    }
}
