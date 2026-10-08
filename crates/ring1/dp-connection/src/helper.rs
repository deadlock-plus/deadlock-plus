use std::collections::HashSet;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::Ipv4Addr;
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use super::capture::{self, Capture};
use super::wire;

const LOCAL_REFRESH: Duration = Duration::from_secs(10);

/// Returns the process exit code.
pub fn run(socket_path: &str) -> i32 {
    let Ok(stream) = UnixStream::connect(socket_path) else {
        return 2;
    };
    let mut out = BufWriter::new(match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return 2,
    });

    let mut capture = match Capture::open() {
        Ok(c) => c,
        Err(e) => {
            let _ = out.write_all(wire::encode_error(&format!("could not capture packets: {e}")).as_bytes());
            let _ = out.flush();
            return 1;
        }
    };

    let filter: Arc<RwLock<HashSet<Ipv4Addr>>> = Arc::default();
    let closed = Arc::new(AtomicBool::new(false));
    {
        let (filter, closed) = (filter.clone(), closed.clone());
        let spawned = thread::Builder::new().name("capture-helper-filter".into()).spawn(move || {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                if let Some(ips) = wire::decode_filter(&line) {
                    *filter.write().unwrap_or_else(|e| e.into_inner()) = ips;
                }
            }
            closed.store(true, Ordering::SeqCst);
        });
        if spawned.is_err() {
            return 2;
        }
    }

    let started = Instant::now();
    let mut local: HashSet<Ipv4Addr> = capture::local_addresses().into_iter().collect();
    let mut local_at = Instant::now();
    while !closed.load(Ordering::SeqCst) {
        if local_at.elapsed() >= LOCAL_REFRESH {
            local = capture::local_addresses().into_iter().collect();
            local_at = Instant::now();
        }
        let mut failed = false;
        let read = capture.read(|ip, stamp| {
            let Some(packet) = wire::parse_ipv4_udp(ip) else { return };
            let Some((remote, inbound)) = wire::direction(&packet, &local) else { return };
            if !filter.read().unwrap_or_else(|e| e.into_inner()).contains(remote.ip()) {
                return;
            }
            let ticks = stamp.unwrap_or_else(|| (started.elapsed().as_nanos() / 100) as i64);
            failed |= out.write_all(wire::encode_packet(inbound, remote, ticks).as_bytes()).is_err();
        });
        if let Err(e) = read {
            let _ = out.write_all(wire::encode_error(&format!("packet capture stopped: {e}")).as_bytes());
            let _ = out.flush();
            return 1;
        }
        if failed || out.flush().is_err() {
            break;
        }
    }
    0
}
