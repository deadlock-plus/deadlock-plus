use std::fs;
use std::io::Read;
use std::path::Path;

const DEADLOCK_APP_ID: &[u8] = b"1422450";
const HOST_SUFFIX: &[u8] = b".valve.net";
const MAX_BYTES_TO_READ: usize = 200;
const PATH_END_MARKERS: &[u8] = b" '\0\n\r\"";

fn find(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    haystack.get(from..)?.windows(needle.len()).position(|w| w == needle).map(|p| p + from)
}

/// Steam's HTTP cache entries start with the request URL; only the first bytes matter.
pub fn extract_replay_url_from_bytes(data: &[u8]) -> Option<String> {
    let mut from = 0;
    while let Some(i) = find(data, HOST_SUFFIX, from) {
        from = i + 1;
        let host_start =
            (0..i).rev().find(|&p| !data[p].is_ascii_alphanumeric() && data[p] != b'.').map_or(0, |p| p + 1);
        let host_end = i + HOST_SUFFIX.len();
        let Ok(host) = std::str::from_utf8(&data[host_start..host_end]) else {
            continue;
        };
        if !host.starts_with("replay") {
            continue;
        }

        let Some(path_start) = data[host_end..].iter().position(|&b| b == b'/').map(|p| p + host_end) else {
            continue;
        };
        let path_slice = &data[path_start..];
        let Some(path_end) = path_slice.iter().position(|b| PATH_END_MARKERS.contains(b)) else {
            continue;
        };
        let Ok(path) = std::str::from_utf8(&path_slice[..path_end]) else {
            continue;
        };
        if find(path.as_bytes(), DEADLOCK_APP_ID, 0).is_none() {
            continue;
        }
        return Some(format!("http://{host}{path}"));
    }
    None
}

pub fn extract_replay_url(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut data = vec![0u8; MAX_BYTES_TO_READ];
    let n = file.read(&mut data).ok()?;
    data.truncate(n);
    extract_replay_url_from_bytes(&data)
}

pub fn scan_directory(dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_directory(&path, out);
        } else if let Some(url) = extract_replay_url(&path) {
            out.push(url);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_url_in_cache_header() {
        let data = b"\x01\x02http://replay404.valve.net/1422450/37959196_937530290.meta.bz2\0rest";
        assert_eq!(
            extract_replay_url_from_bytes(data).as_deref(),
            Some("http://replay404.valve.net/1422450/37959196_937530290.meta.bz2")
        );
    }

    #[test]
    fn ignores_other_games_and_hosts() {
        assert!(extract_replay_url_from_bytes(b"http://replay1.valve.net/730/1_2.dem.bz2\0").is_none());
        assert!(extract_replay_url_from_bytes(b"http://cdn.valve.net/1422450/1_2.dem.bz2\0").is_none());
    }

    #[test]
    fn ignores_unrelated_bytes() {
        assert!(extract_replay_url_from_bytes(b"nothing to see here").is_none());
    }
}
