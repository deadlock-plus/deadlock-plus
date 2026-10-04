use crate::kind::{classify_path, ClientKind, Pipe};
use crate::paths::{socket_dirs, MAX_PIPES};
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

pub type Connection = UnixStream;

fn candidates() -> impl Iterator<Item = (u8, PathBuf)> {
    let dirs = socket_dirs(std::env::var("XDG_RUNTIME_DIR").ok().as_deref(), std::env::var("TMPDIR").ok().as_deref());
    dirs.into_iter().flat_map(|d| (0..MAX_PIPES).map(move |i| (i, d.join(format!("discord-ipc-{i}")))))
}

pub fn discover() -> Vec<Pipe> {
    candidates()
        .filter_map(|(index, path)| {
            let stream = UnixStream::connect(&path).ok()?;
            Some(Pipe { index, kind: label(&stream, &path), path: Some(path) })
        })
        .collect()
}

pub fn connect(pipe: &Pipe) -> io::Result<Connection> {
    match &pipe.path {
        Some(path) => UnixStream::connect(path),
        None => candidates()
            .filter(|(i, _)| *i == pipe.index)
            .find_map(|(_, p)| UnixStream::connect(p).ok())
            .ok_or_else(|| io::ErrorKind::NotFound.into()),
    }
}

/// The peer's executable is the stronger signal. Under Flatpak or Snap the pid lives in another
/// namespace and may not resolve, so the sandbox directory in the socket path is the fallback.
fn label(stream: &UnixStream, path: &Path) -> Option<ClientKind> {
    let from_exe = peer_pid(stream.as_raw_fd()).and_then(exe_path).map(|p| classify_path(&p));
    let from_path = classify_path(&path.to_string_lossy());
    match (from_exe, from_path) {
        (Some(k), _) if k != ClientKind::Other => Some(k),
        (_, k) if k != ClientKind::Other => Some(k),
        (Some(k), _) => Some(k),
        (None, _) => None,
    }
}

#[cfg(target_os = "linux")]
fn peer_pid(fd: i32) -> Option<i32> {
    let mut cred = libc::ucred { pid: 0, uid: 0, gid: 0 };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    let rc = unsafe {
        libc::getsockopt(fd, libc::SOL_SOCKET, libc::SO_PEERCRED, (&mut cred as *mut libc::ucred).cast(), &mut len)
    };
    (rc == 0 && cred.pid > 0).then_some(cred.pid)
}

#[cfg(target_os = "linux")]
fn exe_path(pid: i32) -> Option<String> {
    std::fs::read_link(format!("/proc/{pid}/exe")).ok().map(|p| p.to_string_lossy().into_owned())
}

#[cfg(target_os = "macos")]
fn peer_pid(fd: i32) -> Option<i32> {
    const SOL_LOCAL: libc::c_int = 0;
    const LOCAL_PEERPID: libc::c_int = 0x002;
    let mut pid: libc::pid_t = 0;
    let mut len = std::mem::size_of::<libc::pid_t>() as libc::socklen_t;
    let rc = unsafe { libc::getsockopt(fd, SOL_LOCAL, LOCAL_PEERPID, (&mut pid as *mut libc::pid_t).cast(), &mut len) };
    (rc == 0 && pid > 0).then_some(pid)
}

#[cfg(target_os = "macos")]
fn exe_path(pid: i32) -> Option<String> {
    let mut buf = vec![0u8; 4096];
    let n = unsafe { libc::proc_pidpath(pid, buf.as_mut_ptr().cast(), buf.len() as u32) };
    (n > 0).then(|| String::from_utf8_lossy(&buf[..n as usize]).into_owned())
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn peer_pid(_fd: i32) -> Option<i32> {
    None
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn exe_path(_pid: i32) -> Option<String> {
    None
}
