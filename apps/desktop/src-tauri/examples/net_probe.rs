use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ferrisetw::parser::Parser;
use ferrisetw::provider::Provider;
use ferrisetw::schema_locator::SchemaLocator;
use ferrisetw::trace::{TraceTrait, UserTrace};
use ferrisetw::EventRecord;
use sysinfo::System;

const KERNEL_NETWORK: &str = "7dd42a49-5329-4832-8dfd-43d979153a88";
const SDR_URL: &str = "https://api.steampowered.com/ISteamApps/GetSDRConfig/v1/?appid=1422450";

#[derive(Default)]
struct Stat {
    packets: u64,
    bytes: u64,
    last: Option<i64>,
    max_gap_ms: f64,
    gaps_over_100ms: u32,
    min_size: u32,
    max_size: u32,
}

type Key = (Ipv4Addr, u16, bool);

fn is_private(ip: Ipv4Addr) -> bool {
    ip.is_private() || ip.is_loopback() || ip.is_unspecified() || ip.is_link_local()
}

fn main() {
    let secs: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(15);
    let target = std::env::args().nth(2).unwrap_or_else(|| "deadlock.exe".to_string());

    let sys = System::new_all();
    let pid = sys
        .processes()
        .iter()
        .find(|(_, p)| p.name().to_string_lossy().eq_ignore_ascii_case(&target))
        .map(|(pid, _)| pid.as_u32())
        .expect("target process is not running");
    println!("{target} pid {pid}, capturing {secs}s");

    let pops: HashMap<Ipv4Addr, String> = {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let json: serde_json::Value = rt.block_on(async { reqwest::get(SDR_URL).await.unwrap().json().await.unwrap() });
        let mut m = HashMap::new();
        for (code, pop) in json["pops"].as_object().unwrap() {
            for relay in pop["relays"].as_array().into_iter().flatten() {
                if let Some(ip) = relay["ipv4"].as_str().and_then(|s| s.parse().ok()) {
                    m.insert(ip, format!("{code} ({})", pop["desc"].as_str().unwrap_or("?")));
                }
            }
        }
        m
    };
    println!("{} known SDR relay IPs", pops.len());

    let _ = std::process::Command::new("logman").args(["stop", "DeadlockPlusNetProbe", "-ets"]).output();

    let stats: Arc<Mutex<HashMap<Key, Stat>>> = Arc::default();
    let cb_stats = stats.clone();
    let events: Arc<Mutex<Vec<(i64, bool, Ipv4Addr, u16, u32)>>> = Arc::default();
    let cb_events = events.clone();

    let callback = move |record: &EventRecord, locator: &SchemaLocator| {
        let id = record.event_id();
        if id != 42 && id != 43 {
            return;
        }
        let Ok(schema) = locator.event_schema(record) else {
            return;
        };
        let parser = Parser::create(record, &schema);
        let Ok(event_pid) = parser.try_parse::<u32>("PID") else {
            return;
        };
        if event_pid != pid {
            return;
        }
        let size: u32 = parser.try_parse("size").unwrap_or(0);
        let (Ok(daddr), Ok(saddr), Ok(dport), Ok(sport)) = (
            parser.try_parse::<u32>("daddr"),
            parser.try_parse::<u32>("saddr"),
            parser.try_parse::<u16>("dport"),
            parser.try_parse::<u16>("sport"),
        ) else {
            return;
        };

        let d = Ipv4Addr::from(daddr.to_le_bytes());
        let s = Ipv4Addr::from(saddr.to_le_bytes());
        let (remote, port) = if is_private(d) && !is_private(s) { (s, sport) } else { (d, dport) };
        let port = port.swap_bytes();
        let inbound = id == 43;

        let now = record.raw_timestamp();
        cb_events.lock().unwrap().push((now, inbound, remote, port, size));
        let mut map = cb_stats.lock().unwrap();
        let st = map.entry((remote, port, inbound)).or_default();
        st.packets += 1;
        st.bytes += size as u64;
        st.min_size = if st.packets == 1 { size } else { st.min_size.min(size) };
        st.max_size = st.max_size.max(size);
        if let Some(last) = st.last {
            let gap = (now - last) as f64 / 10_000.0;
            st.max_gap_ms = st.max_gap_ms.max(gap);
            if gap > 100.0 {
                st.gaps_over_100ms += 1;
            }
        }
        st.last = Some(now);
    };

    const SESSION: &str = "DeadlockPlusNetProbe";
    let start = |callback: Box<dyn Fn(&EventRecord, &SchemaLocator) + Send + Sync>| {
        let provider = Provider::by_guid(KERNEL_NETWORK).add_callback(callback).build();
        UserTrace::new().named(SESSION.to_string()).enable(provider).start()
    };

    let callback: Box<dyn Fn(&EventRecord, &SchemaLocator) + Send + Sync> = Box::new(callback);
    let (_trace, handle) = match start(callback) {
        Ok(t) => t,
        Err(e) => panic!("failed to start ETW trace (run elevated; a stale session is stopped automatically): {e:?}"),
    };
    std::thread::spawn(move || {
        let _ = UserTrace::process_from_handle(handle);
    });

    std::thread::sleep(Duration::from_secs(secs));
    drop(_trace);

    let map = stats.lock().unwrap();
    let mut rows: Vec<_> = map.iter().collect();
    rows.sort_by_key(|(_, s)| std::cmp::Reverse(s.packets));
    println!(
        "\n{:<7} {:<21} {:>7} {:>9} {:>8} {:>9} {:>11}  pop",
        "dir", "remote", "pkts", "bytes", "maxgap", ">100ms", "size min-max"
    );
    for ((ip, port, inbound), s) in rows.iter().take(25) {
        println!(
            "{:<7} {:<21} {:>7} {:>9} {:>6}ms {:>9} {:>11}  {}",
            if *inbound { "recv" } else { "send" },
            format!("{ip}:{port}"),
            s.packets,
            s.bytes,
            s.max_gap_ms as u64,
            s.gaps_over_100ms,
            format!("{}-{}", s.min_size, s.max_size),
            pops.get(ip).map(String::as_str).unwrap_or("(not an SDR relay)")
        );
    }

    let mut flows: HashMap<(Ipv4Addr, u16), Vec<(i64, bool, u32)>> = HashMap::new();
    for (t, inbound, ip, port, size) in events.lock().unwrap().iter() {
        flows.entry((*ip, *port)).or_default().push((*t, *inbound, *size));
    }
    println!(
        "
round trips, pairing only small packets (<= 48 bytes) per flow (send -> next recv), ms:"
    );
    for ((ip, port), evs) in &flows {
        let small: Vec<_> = evs.iter().filter(|(_, _, sz)| *sz <= 48).collect();
        let sends = small.iter().filter(|(_, inbound, _)| !*inbound).count();
        let recvs = small.len() - sends;
        if sends == 0 || recvs == 0 {
            continue;
        }
        let mut pending: std::collections::VecDeque<i64> = Default::default();
        let mut rtts = Vec::new();
        for (t, inbound, _) in small {
            if *inbound {
                if let Some(sent) = pending.pop_front() {
                    rtts.push((*t - sent) as f64 / 10_000.0);
                }
            } else {
                pending.push_back(*t);
            }
        }
        rtts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        if rtts.is_empty() {
            continue;
        }
        let pct = |p: f64| rtts[((rtts.len() - 1) as f64 * p) as usize];
        println!(
            "  {ip}:{port}  small sends={sends} recvs={recvs}  paired={}  min={:.1} p10={:.1} median={:.1} p90={:.1} max={:.1}",
            rtts.len(), rtts[0], pct(0.1), pct(0.5), pct(0.9), rtts[rtts.len() - 1]
        );
    }

    println!(
        "
send size histogram (top 8) per flow:"
    );
    for ((ip, port), evs) in &flows {
        let mut h: HashMap<u32, u32> = HashMap::new();
        for (_, inbound, sz) in evs {
            if !*inbound {
                *h.entry(*sz).or_default() += 1;
            }
        }
        if h.values().sum::<u32>() < 200 {
            continue;
        }
        let mut v: Vec<_> = h.into_iter().collect();
        v.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
        let top: Vec<String> = v.iter().take(8).map(|(sz, c)| format!("{sz}B x{c}")).collect();
        println!("  {ip}:{port}  {}", top.join(", "));
    }
}
