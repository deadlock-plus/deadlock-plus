//! Live connection monitor. Everything here is passive or standard OS networking:
//! kernel ETW network events (which process sent UDP where), ICMP echo, and a process
//! list. No game memory is read and nothing is injected.

use std::collections::{HashMap, VecDeque};
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

use super::history_store::HistoryStore;
use super::types::{EndpointInfo, HistoryPoint, PingStats, RelayInfo, Snapshot};
use dp_connection::{self as connection, Config, Packet, Status};
use dp_server_picker::definitions::find_definition;
use dp_server_picker::sdr::fetch_server_data;
use dp_sync::{LockExt, RwLockExt};

const GAME_ID: &str = "deadlock";
const EXITLAG_EXE: &str = "exitlag.exe";

fn is_exitlag(name: &std::ffi::OsStr) -> bool {
    name.to_string_lossy().eq_ignore_ascii_case(EXITLAG_EXE)
}

const PING_WINDOW: usize = 60;
const PING_KEEP: usize = 120;
const HISTORY_KEEP: usize = 900;
const HISTORY_STORE_KEEP: usize = 20_000;
const MIN_RELAY_PACKETS_PER_SEC: f32 = 15.0;
const MIN_TUNNEL_PACKETS_PER_SEC: f32 = 30.0;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Role {
    Game,
    ExitLag,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct FlowKey {
    role: Role,
    ip: Ipv4Addr,
    port: u16,
}

#[derive(Default)]
struct FlowAgg {
    pkts_in: u32,
    pkts_out: u32,
    last_in: Option<i64>,
    max_in_gap_ms: f32,
}

#[derive(Clone)]
struct PopInfo {
    code: String,
    description: String,
    country_code: Option<String>,
}

#[derive(Default)]
struct Targets {
    relay: Option<Ipv4Addr>,
    tunnel: Vec<Ipv4Addr>,
    exit: Option<Ipv4Addr>,
}

struct Shared {
    window: Mutex<HashMap<FlowKey, FlowAgg>>,
    game_pid: AtomicU32,
    exitlag_pid: AtomicU32,
    relay_map: RwLock<HashMap<Ipv4Addr, PopInfo>>,
    targets: Mutex<Targets>,
    pings: Mutex<HashMap<Ipv4Addr, VecDeque<Option<f32>>>>,
    snapshot: Mutex<Snapshot>,
    history: Mutex<VecDeque<HistoryPoint>>,
    store: Mutex<Option<HistoryStore>>,
    stop: AtomicBool,
}

struct Running {
    shared: Arc<Shared>,
    stop_trace: Sender<()>,
}

#[derive(Default)]
pub struct NetworkMonitor {
    running: Mutex<Option<Running>>,
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn ping_stats(samples: Option<&VecDeque<Option<f32>>>) -> PingStats {
    let Some(samples) = samples else {
        return PingStats::default();
    };
    let recent: Vec<Option<f32>> = samples.iter().rev().take(PING_WINDOW).copied().collect();
    if recent.is_empty() {
        return PingStats::default();
    }

    let ok: Vec<f32> = recent.iter().flatten().copied().collect();
    let loss_pct = 100.0 * (recent.len() - ok.len()) as f32 / recent.len() as f32;
    if ok.is_empty() {
        return PingStats { loss_pct, samples: recent.len() as u32, ..Default::default() };
    }

    let avg = ok.iter().sum::<f32>() / ok.len() as f32;
    // Jitter as the mean absolute difference between consecutive successful samples (chronological).
    let chrono: Vec<f32> = recent.iter().rev().flatten().copied().collect();
    let jitter = if chrono.len() > 1 {
        chrono.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f32>() / (chrono.len() - 1) as f32
    } else {
        0.0
    };

    PingStats {
        current: recent.first().copied().flatten(),
        avg: Some(avg),
        min: ok.iter().copied().reduce(f32::min),
        max: ok.iter().copied().reduce(f32::max),
        jitter: Some(jitter),
        loss_pct,
        samples: recent.len() as u32,
    }
}

fn latest(samples: Option<&VecDeque<Option<f32>>>) -> Option<f32> {
    samples.and_then(|s| s.back().copied().flatten())
}

fn flow_pps(flow: &FlowAgg, secs: f32) -> f32 {
    (flow.pkts_in + flow.pkts_out) as f32 / secs
}

fn pick_game_flow(window: &HashMap<FlowKey, FlowAgg>, secs: f32) -> Option<(&FlowKey, &FlowAgg)> {
    window
        .iter()
        .filter(|(k, f)| k.role == Role::Game && flow_pps(f, secs) >= MIN_RELAY_PACKETS_PER_SEC)
        .max_by(|a, b| flow_pps(a.1, secs).total_cmp(&flow_pps(b.1, secs)))
}

fn pick_tunnel_flows(window: &HashMap<FlowKey, FlowAgg>, secs: f32) -> Vec<(&FlowKey, &FlowAgg)> {
    let mut tunnel: Vec<_> = window
        .iter()
        .filter(|(k, f)| k.role == Role::ExitLag && flow_pps(f, secs) >= MIN_TUNNEL_PACKETS_PER_SEC)
        .collect();
    tunnel.sort_by(|a, b| flow_pps(b.1, secs).total_cmp(&flow_pps(a.1, secs)));
    tunnel.truncate(3);
    tunnel
}

// The exit server is the tunnel endpoint furthest from us; the near one is a local entry point.
fn pick_exit(tunnel_ips: &[Ipv4Addr], pings: &HashMap<Ipv4Addr, VecDeque<Option<f32>>>) -> Option<Ipv4Addr> {
    tunnel_ips
        .iter()
        .filter_map(|ip| ping_stats(pings.get(ip)).avg.map(|avg| (*ip, avg)))
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(ip, _)| ip)
}

impl NetworkMonitor {
    /// `prompt` says the user asked for this, so a permission prompt is acceptable.
    pub fn start(&self, http: reqwest::Client, history_path: PathBuf, prompt: bool) {
        let mut guard = self.running.lock_or_recover();
        if let Some(running) = guard.as_ref() {
            let snap = running.shared.snapshot.lock_or_recover();
            let retry = snap.trace_error.is_some() || (snap.needs_permission && prompt);
            drop(snap);
            if !retry {
                return;
            }
            Self::shutdown(guard.take());
        }

        // Persistence is best-effort: without it the monitor still works, history just resets on restart.
        let (store, saved) = match HistoryStore::open(&history_path, HISTORY_STORE_KEEP) {
            Ok((store, saved)) => (Some(store), saved),
            Err(e) => {
                log::warn!("connection history unavailable, it will reset on restart: {e}");
                (None, Vec::new())
            }
        };
        let resume_from = saved.len().saturating_sub(HISTORY_KEEP);

        let shared = Arc::new(Shared {
            window: Mutex::default(),
            game_pid: AtomicU32::new(0),
            exitlag_pid: AtomicU32::new(0),
            relay_map: RwLock::default(),
            targets: Mutex::default(),
            pings: Mutex::default(),
            snapshot: Mutex::new(Snapshot { monitoring: true, ..Default::default() }),
            history: Mutex::new(saved[resume_from..].iter().cloned().collect()),
            store: Mutex::new(store),
            stop: AtomicBool::new(false),
        });

        log::info!("network monitor starting ({} saved history points)", saved.len());
        let stop_trace = spawn_trace(shared.clone(), prompt);
        spawn_relay_map_loader(shared.clone(), http);
        spawn_aggregator(shared.clone());
        spawn_sampler(shared.clone());

        *guard = Some(Running { shared, stop_trace });
    }

    pub fn stop(&self) {
        Self::shutdown(self.running.lock_or_recover().take());
    }

    fn shutdown(running: Option<Running>) {
        if let Some(r) = running {
            log::info!("network monitor stopping");
            r.shared.stop.store(true, Ordering::SeqCst);
            let _ = r.stop_trace.send(());
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        match self.running.lock_or_recover().as_ref() {
            Some(r) => r.shared.snapshot.lock_or_recover().clone(),
            None => Snapshot::default(),
        }
    }

    pub fn history(&self) -> Vec<HistoryPoint> {
        match self.running.lock_or_recover().as_ref() {
            Some(r) => r.shared.history.lock_or_recover().iter().cloned().collect(),
            None => Vec::new(),
        }
    }
}

fn record_packet(window: &mut HashMap<FlowKey, FlowAgg>, role: Role, packet: &Packet) {
    let flow = window.entry(FlowKey { role, ip: *packet.remote.ip(), port: packet.remote.port() }).or_default();
    if packet.inbound {
        flow.pkts_in += 1;
        if let Some(last) = flow.last_in {
            flow.max_in_gap_ms = flow.max_in_gap_ms.max((packet.ticks_100ns - last) as f32 / 10_000.0);
        }
        flow.last_in = Some(packet.ticks_100ns);
    } else {
        flow.pkts_out += 1;
    }
}

/// Sources that can name the process (Windows) match on it. Others report every packet to a relay, which
/// belong to the game whenever it is running.
fn role_for(pid: Option<u32>, game_pid: u32, exitlag_pid: u32) -> Option<Role> {
    match pid {
        Some(pid) if pid == game_pid => Some(Role::Game),
        Some(pid) if pid == exitlag_pid => Some(Role::ExitLag),
        Some(_) => None,
        None => (game_pid != 0).then_some(Role::Game),
    }
}

fn spawn_trace(shared: Arc<Shared>, prompt: bool) -> Sender<()> {
    let wanted_shared = shared.clone();
    let wanted = Arc::new(move |pid: u32| {
        pid == wanted_shared.game_pid.load(Ordering::Relaxed)
            || pid == wanted_shared.exitlag_pid.load(Ordering::Relaxed)
    });

    let sink_shared = shared.clone();
    let sink = Arc::new(move |packet: Packet| {
        let game = sink_shared.game_pid.load(Ordering::Relaxed);
        let exitlag = sink_shared.exitlag_pid.load(Ordering::Relaxed);
        if let Some(role) = role_for(packet.pid, game, exitlag) {
            record_packet(&mut sink_shared.window.lock_or_recover(), role, &packet);
        }
    });

    let remotes_shared = shared.clone();
    let remotes = Arc::new(move || remotes_shared.relay_map.read_or_recover().keys().copied().collect());

    let status_shared = shared;
    connection::start(Config {
        wanted,
        sink,
        remotes,
        on_status: Box::new(move |status| {
            let mut snap = status_shared.snapshot.lock_or_recover();
            match status {
                Status::Failed(message) => snap.trace_error = Some(message),
                Status::NeedsPermission => snap.needs_permission = true,
            }
        }),
        prompt,
    })
}

fn spawn_relay_map_loader(shared: Arc<Shared>, http: reqwest::Client) {
    tauri::async_runtime::spawn(async move {
        let Some(def) = find_definition(GAME_ID) else {
            log::error!("no game definition for {GAME_ID}, relay names will be missing");
            return;
        };
        let mut failing = false;
        while !shared.stop.load(Ordering::SeqCst) {
            let wait = match fetch_server_data(&http, &def).await {
                Ok(data) => {
                    let mut map = HashMap::new();
                    for group in &data.unclustered {
                        for ip in group.relay_ips.iter().filter_map(|ip| ip.parse::<Ipv4Addr>().ok()) {
                            map.insert(
                                ip,
                                PopInfo {
                                    code: group.id.clone(),
                                    description: group.description.clone(),
                                    country_code: group.country_code.clone(),
                                },
                            );
                        }
                    }
                    log::debug!("relay map loaded with {} addresses", map.len());
                    *shared.relay_map.write_or_recover() = map;
                    failing = false;
                    Duration::from_secs(3600)
                }
                Err(e) => {
                    if failing {
                        log::debug!("relay map fetch still failing: {e}");
                    } else {
                        log::warn!("relay map fetch failed, retrying every 30 s: {e}");
                        failing = true;
                    }
                    Duration::from_secs(30)
                }
            };
            tokio::time::sleep(wait).await;
        }
    });
}

fn spawn_aggregator(shared: Arc<Shared>) {
    thread::Builder::new()
        .name("network-aggregator".into())
        .spawn(move || {
            let mut sys = System::new();
            let mut last_procs = Instant::now() - Duration::from_secs(10);
            let mut last_tick = Instant::now();

            while !shared.stop.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_secs(1));

                if last_procs.elapsed() >= Duration::from_secs(2) {
                    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
                    let find = |matches: fn(&std::ffi::OsStr) -> bool| {
                        sys.processes()
                            .iter()
                            .find(|(_, p)| matches(p.name()))
                            .map(|(pid, _)| pid.as_u32())
                            .unwrap_or(0)
                    };
                    shared.game_pid.store(find(dp_game::is_process), Ordering::Relaxed);
                    shared.exitlag_pid.store(find(is_exitlag), Ordering::Relaxed);
                    last_procs = Instant::now();
                }

                let secs = last_tick.elapsed().as_secs_f32().max(0.1);
                last_tick = Instant::now();
                let window = std::mem::take(&mut *shared.window.lock_or_recover());

                let total = |f: &FlowAgg| flow_pps(f, secs);

                let game_flow = pick_game_flow(&window, secs);
                let tunnel = pick_tunnel_flows(&window, secs);

                let relay_ip = game_flow.map(|(k, _)| k.ip);
                let tunnel_ips: Vec<Ipv4Addr> = tunnel.iter().map(|(k, _)| k.ip).collect();

                let pings = shared.pings.lock_or_recover();
                let exit_ip = pick_exit(&tunnel_ips, &pings);

                {
                    let mut targets = shared.targets.lock_or_recover();
                    targets.relay = relay_ip;
                    targets.tunnel = tunnel_ips.clone();
                    targets.exit = exit_ip;
                }

                let relay_map = shared.relay_map.read_or_recover();
                let relay = game_flow.map(|(k, f)| {
                    let pop = relay_map.get(&k.ip);
                    RelayInfo {
                        ip: k.ip.to_string(),
                        port: k.port,
                        pop_code: pop.map(|p| p.code.clone()),
                        description: pop.map(|p| p.description.clone()),
                        country_code: pop.and_then(|p| p.country_code.clone()),
                        pps_in: f.pkts_in as f32 / secs,
                        pps_out: f.pkts_out as f32 / secs,
                        max_gap_ms: f.max_in_gap_ms,
                        ping: ping_stats(pings.get(&k.ip)),
                    }
                });

                let exitlag_endpoints = tunnel
                    .iter()
                    .map(|(k, f)| EndpointInfo {
                        ip: k.ip.to_string(),
                        port: k.port,
                        pps: total(f),
                        is_exit: Some(k.ip) == exit_ip,
                        ping: ping_stats(pings.get(&k.ip)),
                    })
                    .collect();

                let mut snap = shared.snapshot.lock_or_recover();
                snap.game_running = shared.game_pid.load(Ordering::Relaxed) != 0;
                snap.exitlag_running = shared.exitlag_pid.load(Ordering::Relaxed) != 0;
                snap.relay = relay;
                snap.exitlag_endpoints = exitlag_endpoints;
                snap.updated_at_ms = now_ms();
            }
        })
        .expect("spawn thread");
}

fn spawn_sampler(shared: Arc<Shared>) {
    thread::Builder::new()
        .name("network-sampler".into())
        .spawn(move || {
            let mut store_failing = false;
            while !shared.stop.load(Ordering::SeqCst) {
                let cycle = Instant::now();

                let (relay, tunnel, exit) = {
                    let t = shared.targets.lock_or_recover();
                    (t.relay, t.tunnel.clone(), t.exit)
                };

                let mut wanted: Vec<Ipv4Addr> = tunnel;
                wanted.extend(relay);
                wanted.sort();
                wanted.dedup();

                let results: Vec<(Ipv4Addr, Option<f32>)> = thread::scope(|scope| {
                    let handles: Vec<_> =
                        wanted.iter().map(|ip| scope.spawn(move || (*ip, dp_icmp::ping(*ip, 1000)))).collect();
                    handles.into_iter().filter_map(|h| h.join().ok()).collect()
                });

                let mut pings = shared.pings.lock_or_recover();
                pings.retain(|ip, _| wanted.contains(ip));
                for (ip, rtt) in results {
                    let samples = pings.entry(ip).or_default();
                    samples.push_back(rtt);
                    while samples.len() > PING_KEEP {
                        samples.pop_front();
                    }
                }

                if relay.is_some() || exit.is_some() {
                    let point = HistoryPoint {
                        t: now_ms(),
                        raw: relay.and_then(|ip| latest(pings.get(&ip))),
                        exit: exit.and_then(|ip| latest(pings.get(&ip))),
                    };
                    if let Some(store) = shared.store.lock_or_recover().as_mut() {
                        match store.append(&point) {
                            Ok(()) => store_failing = false,
                            Err(e) if !store_failing => {
                                log::warn!("could not save connection history: {e}");
                                store_failing = true;
                            }
                            Err(_) => {}
                        }
                    }
                    let mut history = shared.history.lock_or_recover();
                    history.push_back(point);
                    while history.len() > HISTORY_KEEP {
                        history.pop_front();
                    }
                }
                drop(pings);

                if let Some(rest) = Duration::from_secs(1).checked_sub(cycle.elapsed()) {
                    thread::sleep(rest);
                }
            }
        })
        .expect("spawn thread");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn samples(values: &[Option<f32>]) -> VecDeque<Option<f32>> {
        values.iter().copied().collect()
    }

    fn ip(last: u8) -> Ipv4Addr {
        Ipv4Addr::new(10, 0, 0, last)
    }

    fn flow(pkts_in: u32, pkts_out: u32) -> FlowAgg {
        FlowAgg { pkts_in, pkts_out, ..Default::default() }
    }

    fn key(role: Role, last: u8) -> FlowKey {
        FlowKey { role, ip: ip(last), port: 27015 }
    }

    fn packet(inbound: bool, ticks_100ns: i64) -> Packet {
        Packet { pid: Some(1), remote: std::net::SocketAddrV4::new(ip(1), 27015), inbound, ticks_100ns }
    }

    #[test]
    fn a_known_process_decides_the_role() {
        assert_eq!(role_for(Some(10), 10, 20), Some(Role::Game));
        assert_eq!(role_for(Some(20), 10, 20), Some(Role::ExitLag));
        assert_eq!(role_for(Some(30), 10, 20), None);
    }

    #[test]
    fn an_unattributed_packet_belongs_to_the_game_only_while_it_runs() {
        assert_eq!(role_for(None, 10, 0), Some(Role::Game));
        assert_eq!(role_for(None, 0, 0), None);
    }

    #[test]
    fn packets_are_counted_per_direction_and_flow() {
        let mut window = HashMap::new();
        record_packet(&mut window, Role::Game, &packet(true, 0));
        record_packet(&mut window, Role::Game, &packet(false, 5));
        record_packet(&mut window, Role::ExitLag, &packet(true, 5));
        let game = &window[&key(Role::Game, 1)];
        assert_eq!((game.pkts_in, game.pkts_out), (1, 1));
        assert_eq!(window[&key(Role::ExitLag, 1)].pkts_in, 1);
    }

    #[test]
    fn the_longest_gap_between_inbound_packets_is_kept_in_milliseconds() {
        let mut window = HashMap::new();
        for ticks in [0, 100_000, 600_000, 700_000] {
            record_packet(&mut window, Role::Game, &packet(true, ticks));
        }
        assert_eq!(window[&key(Role::Game, 1)].max_in_gap_ms, 50.0);
    }

    #[test]
    fn ping_stats_without_samples_is_empty() {
        assert_eq!(ping_stats(None), PingStats::default());
        assert_eq!(ping_stats(Some(&VecDeque::new())), PingStats::default());
    }

    #[test]
    fn ping_stats_computes_avg_min_max_and_current() {
        let s = samples(&[Some(10.0), Some(30.0), Some(20.0)]);
        let stats = ping_stats(Some(&s));
        assert_eq!(stats.avg, Some(20.0));
        assert_eq!(stats.min, Some(10.0));
        assert_eq!(stats.max, Some(30.0));
        assert_eq!(stats.current, Some(20.0));
        assert_eq!(stats.samples, 3);
        assert_eq!(stats.loss_pct, 0.0);
    }

    #[test]
    fn ping_stats_jitter_is_mean_step_between_consecutive_samples() {
        let s = samples(&[Some(10.0), Some(30.0), Some(20.0)]);
        assert_eq!(ping_stats(Some(&s)).jitter, Some(15.0));
    }

    #[test]
    fn ping_stats_jitter_skips_lost_samples() {
        let s = samples(&[Some(10.0), None, Some(20.0)]);
        assert_eq!(ping_stats(Some(&s)).jitter, Some(10.0));
    }

    #[test]
    fn ping_stats_counts_loss_and_current_is_none_when_latest_lost() {
        let s = samples(&[Some(10.0), None, None, Some(20.0), None]);
        let stats = ping_stats(Some(&s));
        assert_eq!(stats.loss_pct, 60.0);
        assert_eq!(stats.current, None);
        assert_eq!(stats.avg, Some(15.0));
    }

    #[test]
    fn ping_stats_all_lost_reports_full_loss_without_timings() {
        let s = samples(&[None, None]);
        let stats = ping_stats(Some(&s));
        assert_eq!(stats.loss_pct, 100.0);
        assert_eq!(stats.avg, None);
        assert_eq!(stats.jitter, None);
        assert_eq!(stats.samples, 2);
    }

    #[test]
    fn ping_stats_only_uses_the_latest_window() {
        let mut values = vec![Some(1000.0); 10];
        values.extend(vec![Some(10.0); PING_WINDOW]);
        let stats = ping_stats(Some(&samples(&values)));
        assert_eq!(stats.avg, Some(10.0));
        assert_eq!(stats.samples, PING_WINDOW as u32);
    }

    #[test]
    fn game_flow_picks_the_busiest_flow_above_threshold() {
        let window = HashMap::from([
            (key(Role::Game, 1), flow(10, 10)),
            (key(Role::Game, 2), flow(50, 50)),
            (key(Role::Game, 3), flow(5, 5)),
        ]);
        let (k, _) = pick_game_flow(&window, 1.0).unwrap();
        assert_eq!(k.ip, ip(2));
    }

    #[test]
    fn game_flow_ignores_quiet_flows_and_other_roles() {
        let window = HashMap::from([(key(Role::Game, 1), flow(5, 5)), (key(Role::ExitLag, 2), flow(500, 500))]);
        assert!(pick_game_flow(&window, 1.0).is_none());
    }

    #[test]
    fn game_flow_threshold_scales_with_elapsed_seconds() {
        let window = HashMap::from([(key(Role::Game, 1), flow(20, 20))]);
        assert!(pick_game_flow(&window, 1.0).is_some());
        assert!(pick_game_flow(&window, 4.0).is_none());
    }

    #[test]
    fn tunnel_flows_keep_top_three_busiest_first() {
        let window = HashMap::from([
            (key(Role::ExitLag, 1), flow(40, 0)),
            (key(Role::ExitLag, 2), flow(90, 0)),
            (key(Role::ExitLag, 3), flow(60, 0)),
            (key(Role::ExitLag, 4), flow(70, 0)),
            (key(Role::ExitLag, 5), flow(10, 0)),
            (key(Role::Game, 6), flow(900, 0)),
        ]);
        let ips: Vec<_> = pick_tunnel_flows(&window, 1.0).iter().map(|(k, _)| k.ip).collect();
        assert_eq!(ips, vec![ip(2), ip(4), ip(3)]);
    }

    #[test]
    fn exit_is_the_endpoint_with_the_highest_average_ping() {
        let pings =
            HashMap::from([(ip(1), samples(&[Some(5.0), Some(5.0)])), (ip(2), samples(&[Some(121.0), Some(123.0)]))]);
        assert_eq!(pick_exit(&[ip(1), ip(2)], &pings), Some(ip(2)));
    }

    #[test]
    fn exit_is_none_until_an_endpoint_has_a_successful_sample() {
        let pings = HashMap::from([(ip(1), samples(&[None, None]))]);
        assert_eq!(pick_exit(&[ip(1), ip(2)], &pings), None);
    }
}
