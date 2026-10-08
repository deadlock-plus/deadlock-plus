use std::ffi::{OsStr, OsString};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// Deadlock runs as `deadlock.exe` under Wine and Proton too.
pub const PROCESS_NAME: &str = "deadlock.exe";

pub fn is_process(name: &OsStr) -> bool {
    name.to_string_lossy().eq_ignore_ascii_case(PROCESS_NAME)
}

/// How long a scan stays valid. Every caller shares it, so the whole app enumerates processes at most this often.
pub const MAX_AGE: Duration = Duration::from_secs(2);

#[derive(Clone)]
struct Proc {
    pid: u32,
    name: OsString,
    start: u64,
}

/// Time-bounded cache over a process scanner. The scan runs on whichever caller finds it stale, under the lock, so
/// concurrent callers share one scan and the first call always sees a fresh value.
struct ScanCache<S> {
    scan: S,
    max_age: Duration,
    last: Mutex<Option<(Instant, Vec<Proc>)>>,
}

impl<S: Fn() -> Vec<Proc>> ScanCache<S> {
    fn new(scan: S, max_age: Duration) -> Self {
        Self { scan, max_age, last: Mutex::new(None) }
    }

    fn with<R>(&self, now: Instant, read: impl FnOnce(&[Proc]) -> R) -> R {
        let mut last = self.last.lock().unwrap_or_else(|e| e.into_inner());
        let fresh = matches!(&*last, Some((at, _)) if now.saturating_duration_since(*at) < self.max_age);
        if !fresh {
            *last = Some((now, (self.scan)()));
        }
        read(last.as_ref().map(|(_, procs)| procs.as_slice()).unwrap_or_default())
    }
}

fn scan_system() -> Vec<Proc> {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    sys.processes()
        .iter()
        .map(|(pid, p)| Proc { pid: pid.as_u32(), name: p.name().to_owned(), start: p.start_time() })
        .collect()
}

type SystemCache = ScanCache<fn() -> Vec<Proc>>;

fn shared() -> &'static SystemCache {
    static CACHE: OnceLock<SystemCache> = OnceLock::new();
    CACHE.get_or_init(|| ScanCache::new(scan_system as fn() -> Vec<Proc>, MAX_AGE))
}

fn running(procs: &[Proc]) -> bool {
    procs.iter().any(|p| is_process(&p.name))
}

fn earliest_start(procs: &[Proc]) -> Option<u64> {
    procs.iter().filter(|p| is_process(&p.name)).map(|p| p.start).min()
}

fn first_pid(procs: &[Proc], matches: fn(&OsStr) -> bool) -> u32 {
    procs.iter().find(|p| matches(&p.name)).map(|p| p.pid).unwrap_or(0)
}

/// Whether Deadlock is running. Reads a process scan shared app-wide and at most [`MAX_AGE`] old.
pub fn is_running() -> bool {
    shared().with(Instant::now(), running)
}

/// Unix seconds at which the oldest running Deadlock process started, or `None` when the game is not running.
pub fn start_time() -> Option<u64> {
    shared().with(Instant::now(), earliest_start)
}

/// Pid of the first process whose name satisfies `matches`, or 0. Shares the same bounded-age scan.
pub fn find_pid(matches: fn(&OsStr) -> bool) -> u32 {
    shared().with(Instant::now(), |procs| first_pid(procs, matches))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proc(pid: u32, name: &str, start: u64) -> Proc {
        Proc { pid, name: name.into(), start }
    }

    #[test]
    fn running_finds_the_game_among_names() {
        assert!(running(&[proc(1, "explorer.exe", 5), proc(2, "Deadlock.exe", 9)]));
        assert!(!running(&[proc(1, "explorer.exe", 5), proc(2, "deadlock-plus.exe", 9)]));
        assert!(!running(&[]));
    }

    #[test]
    fn earliest_start_among_matching_processes_wins() {
        let procs = [proc(1, "explorer.exe", 5), proc(2, "deadlock.exe", 300), proc(3, "Deadlock.exe", 200)];
        assert_eq!(earliest_start(&procs), Some(200));
        assert_eq!(earliest_start(&procs[..1]), None);
    }

    #[test]
    fn first_pid_is_zero_without_a_match() {
        let procs = [proc(7, "deadlock.exe", 1)];
        assert_eq!(first_pid(&procs, is_process), 7);
        assert_eq!(first_pid(&procs[..0], is_process), 0);
    }

    fn counting(calls: &std::sync::Arc<std::sync::atomic::AtomicUsize>, pid: u32) -> ScanCache<impl Fn() -> Vec<Proc>> {
        let calls = calls.clone();
        ScanCache::new(
            move || {
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                vec![proc(pid, "deadlock.exe", 10)]
            },
            Duration::from_secs(2),
        )
    }

    #[test]
    fn first_read_scans_and_fresh_reads_reuse_it() {
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let cache = counting(&calls, 4);
        let t0 = Instant::now();
        assert!(cache.with(t0, running));
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
        assert_eq!(cache.with(t0 + Duration::from_millis(1900), |p| first_pid(p, is_process)), 4);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn stale_reads_rescan() {
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let cache = counting(&calls, 4);
        let t0 = Instant::now();
        cache.with(t0, running);
        cache.with(t0 + Duration::from_secs(2), running);
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[test]
    fn matches_the_game_exe_in_any_case_only() {
        assert!(is_process(OsStr::new("deadlock.exe")));
        assert!(is_process(OsStr::new("Deadlock.EXE")));
        assert!(!is_process(OsStr::new("deadlock")));
        assert!(!is_process(OsStr::new("deadlock-plus.exe")));
    }
}
