//! ETW session that records the game's DXGI present cadence. Passive: no injection, no game memory.

use std::os::windows::process::CommandExt;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ferrisetw::provider::Provider;
use ferrisetw::trace::{TraceTrait, UserTrace};
use ferrisetw::EventRecord;
use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use ts_rs::TS;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

use super::{recent_frametimes_ms, split_focused, FrameStats};
use dp_sync::LockExt;

const DXGI_PROVIDER: &str = "CA11C036-0102-4A2D-A6AD-F03CFED5D3C9";
const DXGI_KEYWORD: u64 = 0x8000_0000_0000_0002;
const PRESENT_START: u16 = 0x2a;
const SESSION: &str = "DeadlockPlusFrames";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// Event timestamps arrive as 100 ns FILETIME because the trace is not opened in raw-timestamp mode.
const TICKS_PER_SECOND: u64 = 10_000_000;
const MAX_FRAMES: usize = 1_000_000;
const LIVE_WINDOW: usize = 300;
const FOCUS_POLL: Duration = Duration::from_millis(50);
/// Frames this close to a focus change are dropped too: the window is polled, and the game hitches as
/// it resumes.
const FOCUS_PAD_TICKS: u64 = 2_500_000;
const UNIX_TO_FILETIME_TICKS: u64 = 116_444_736_000_000_000;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CaptureState {
    #[default]
    Idle,
    WaitingForGame,
    Capturing,
    Failed,
}

#[derive(Debug, Clone, Default, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptureStatus {
    pub state: CaptureState,
    pub error: Option<String>,
    pub frames: u32,
    pub elapsed_ms: u32,
    /// Present events seen from any process. Nonzero with zero `frames` means the game is not
    /// presenting through DXGI.
    pub other_process_events: u32,
    pub truncated: bool,
    pub game_focused: bool,
    pub recent_frametimes_ms: Vec<f32>,
}

struct Shared {
    timestamps: Mutex<Vec<u64>>,
    game_pid: AtomicU32,
    other_events: AtomicU64,
    truncated: AtomicBool,
    game_focused: AtomicBool,
    unfocused: Mutex<Vec<(u64, u64)>>,
    stop: AtomicBool,
    error: Mutex<Option<String>>,
    started: Instant,
}

struct Running {
    shared: Arc<Shared>,
    stop_trace: Sender<()>,
}

#[derive(Default)]
pub struct FrameCapture {
    running: Mutex<Option<Running>>,
}

impl FrameCapture {
    pub fn start(&self) {
        let mut guard = self.running.lock_or_recover();
        if guard.is_some() {
            return;
        }
        let shared = Arc::new(Shared {
            timestamps: Mutex::new(Vec::with_capacity(60 * 60 * 10)),
            game_pid: AtomicU32::new(0),
            other_events: AtomicU64::new(0),
            truncated: AtomicBool::new(false),
            game_focused: AtomicBool::new(true),
            unfocused: Mutex::new(Vec::new()),
            stop: AtomicBool::new(false),
            error: Mutex::new(None),
            started: Instant::now(),
        });
        log::info!("frame capture starting");
        let stop_trace = spawn_trace(shared.clone());
        spawn_pid_poller(shared.clone());
        spawn_focus_poller(shared.clone());
        *guard = Some(Running { shared, stop_trace });
    }

    pub fn status(&self) -> CaptureStatus {
        let guard = self.running.lock_or_recover();
        let Some(r) = guard.as_ref() else {
            return CaptureStatus::default();
        };
        let s = &r.shared;
        let error = s.error.lock_or_recover().clone();
        let (frames, recent) = {
            let ts = s.timestamps.lock_or_recover();
            (ts.len(), recent_frametimes_ms(&ts, LIVE_WINDOW, TICKS_PER_SECOND))
        };
        let state = if error.is_some() {
            CaptureState::Failed
        } else if s.game_pid.load(Ordering::Relaxed) == 0 {
            CaptureState::WaitingForGame
        } else {
            CaptureState::Capturing
        };
        CaptureStatus {
            state,
            error,
            frames: frames as u32,
            elapsed_ms: s.started.elapsed().as_millis().min(u32::MAX as u128) as u32,
            other_process_events: s.other_events.load(Ordering::Relaxed).min(u32::MAX as u64) as u32,
            truncated: s.truncated.load(Ordering::Relaxed),
            game_focused: s.game_focused.load(Ordering::Relaxed),
            recent_frametimes_ms: recent,
        }
    }

    /// Ends the session and returns the statistics for what it recorded.
    pub fn stop(&self) -> FrameStats {
        let Some(r) = self.running.lock_or_recover().take() else {
            return FrameStats::default();
        };
        log::info!("frame capture stopping");
        r.shared.stop.store(true, Ordering::SeqCst);
        let _ = r.stop_trace.send(());
        let timestamps = std::mem::take(&mut *r.shared.timestamps.lock_or_recover());
        let unfocused = std::mem::take(&mut *r.shared.unfocused.lock_or_recover());
        let padded: Vec<(u64, u64)> =
            unfocused.iter().map(|&(a, b)| (a.saturating_sub(FOCUS_PAD_TICKS), b + FOCUS_PAD_TICKS)).collect();
        let mut stats = FrameStats::from_segments(&split_focused(&timestamps, &padded), TICKS_PER_SECOND);
        stats.background_ms =
            unfocused.iter().map(|&(a, b)| b.saturating_sub(a)).sum::<u64>() as f64 * 1000.0 / TICKS_PER_SECOND as f64;
        log::info!(
            "frame capture result: {} frames over {:.1}s, median {:.2} ms, p99 {:.2} ms, max {:.2} ms, {} spikes",
            stats.frame_count,
            stats.duration_ms / 1000.0,
            stats.median_ms,
            stats.p99_ms,
            stats.max_ms,
            stats.spikes.len()
        );
        stats
    }
}

fn spawn_trace(shared: Arc<Shared>) -> Sender<()> {
    let (tx, rx) = mpsc::channel::<()>();

    thread::Builder::new()
        .name("etw-frames-session".into())
        .spawn(move || {
            // ETW sessions outlive the process that created them, so clear one left by a crashed run.
            if let Err(e) = std::process::Command::new("logman")
                .args(["stop", SESSION, "-ets"])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
            {
                log::debug!("could not run logman to clear a stale frame session: {e}");
            }

            let cb_shared = shared.clone();
            let callback = move |record: &EventRecord, _locator: &ferrisetw::schema_locator::SchemaLocator| {
                if record.event_id() != PRESENT_START {
                    return;
                }
                if record.process_id() != cb_shared.game_pid.load(Ordering::Relaxed) {
                    cb_shared.other_events.fetch_add(1, Ordering::Relaxed);
                    return;
                }
                let mut ts = cb_shared.timestamps.lock_or_recover();
                if ts.len() < MAX_FRAMES {
                    ts.push(record.raw_timestamp() as u64);
                } else {
                    cb_shared.truncated.store(true, Ordering::Relaxed);
                }
            };

            let provider = Provider::by_guid(DXGI_PROVIDER).any(DXGI_KEYWORD).add_callback(callback).build();
            match UserTrace::new().named(SESSION.to_string()).enable(provider).start() {
                Ok((trace, handle)) => {
                    log::info!("frame trace started");
                    thread::Builder::new()
                        .name("etw-frames-processor".into())
                        .spawn(move || {
                            if let Err(e) = UserTrace::process_from_handle(handle) {
                                log::warn!("frame trace processing ended with an error: {e:?}");
                            }
                        })
                        .expect("spawn thread");
                    let _ = rx.recv();
                    drop(trace);
                }
                Err(e) => {
                    log::error!("couldn't start the frame trace: {e:?}");
                    *shared.error.lock_or_recover() =
                        Some(format!("couldn't start the frame trace (needs administrator): {e:?}"));
                }
            }
        })
        .expect("spawn thread");

    tx
}

fn spawn_pid_poller(shared: Arc<Shared>) {
    thread::Builder::new()
        .name("frames-pid-poller".into())
        .spawn(move || {
            let mut sys = System::new();
            let mut ticks = 0u32;
            while !shared.stop.load(Ordering::SeqCst) {
                sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
                let pid = sys
                    .processes()
                    .iter()
                    .find(|(_, p)| crate::features::game::is_process(p.name()))
                    .map(|(pid, _)| pid.as_u32())
                    .unwrap_or(0);
                shared.game_pid.store(pid, Ordering::Relaxed);
                ticks += 1;
                if ticks % 5 == 0 {
                    log::debug!(
                        "frame capture: game pid {pid}, {} frames, {} present events from other processes",
                        shared.timestamps.lock_or_recover().len(),
                        shared.other_events.load(Ordering::Relaxed)
                    );
                }
                thread::sleep(Duration::from_secs(1));
            }
        })
        .expect("spawn thread");
}

fn filetime_now() -> u64 {
    let since_unix = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| (d.as_nanos() / 100) as u64).unwrap_or(0);
    since_unix + UNIX_TO_FILETIME_TICKS
}

fn foreground_pid() -> u32 {
    // SAFETY: both calls take no ownership; a null window yields pid 0.
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() {
            return 0;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        pid
    }
}

/// Records when the game window loses the foreground, so tab-outs (where the engine may cap or
/// suspend rendering) do not read as stutter.
fn spawn_focus_poller(shared: Arc<Shared>) {
    thread::Builder::new()
        .name("frames-focus-poller".into())
        .spawn(move || {
            let mut open_since: Option<u64> = None;
            while !shared.stop.load(Ordering::SeqCst) {
                let game = shared.game_pid.load(Ordering::Relaxed);
                let focused = game == 0 || foreground_pid() == game;
                shared.game_focused.store(focused, Ordering::Relaxed);
                match (focused, open_since) {
                    (false, None) => open_since = Some(filetime_now()),
                    (true, Some(start)) => {
                        shared.unfocused.lock_or_recover().push((start, filetime_now()));
                        open_since = None;
                    }
                    _ => {}
                }
                thread::sleep(FOCUS_POLL);
            }
            if let Some(start) = open_since {
                shared.unfocused.lock_or_recover().push((start, filetime_now()));
            }
        })
        .expect("spawn thread");
}
