use crate::kind::{classify_path, Pipe};
use crate::paths::MAX_PIPES;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::io::AsRawHandle;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Pipes::GetNamedPipeServerProcessId;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};

const ERROR_PIPE_BUSY: i32 = 231;

pub type Connection = File;

fn open(index: u8) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(format!(r"\\.\pipe\discord-ipc-{index}"))
}

pub fn discover() -> Vec<Pipe> {
    (0..MAX_PIPES)
        .filter_map(|i| match open(i) {
            Ok(file) => Some(Pipe::new(i, server_exe(&file).map(|p| classify_path(&p)))),
            // All instances taken: the pipe exists but cannot be inspected.
            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) => Some(Pipe::new(i, None)),
            Err(_) => None,
        })
        .collect()
}

pub fn connect(pipe: &Pipe) -> io::Result<Connection> {
    open(pipe.index)
}

fn server_exe(file: &File) -> Option<String> {
    let handle = HANDLE(file.as_raw_handle());
    let mut pid = 0u32;
    unsafe { GetNamedPipeServerProcessId(handle, &mut pid).ok()? };
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()? };
    let mut buf = vec![0u16; 1024];
    let mut len = buf.len() as u32;
    let result = unsafe {
        QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, windows::core::PWSTR(buf.as_mut_ptr()), &mut len)
    };
    let _ = unsafe { CloseHandle(process) };
    result.ok()?;
    Some(String::from_utf16_lossy(&buf[..len as usize]))
}
