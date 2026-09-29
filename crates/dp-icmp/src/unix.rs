use std::net::Ipv4Addr;
use std::process::Command;

fn parse_round_trip(output: &str) -> Option<f32> {
    let rest = &output[output.find("time=")? + 5..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect();
    digits.parse().ok()
}

/// The reply-wait flag is in seconds on Linux and milliseconds on macOS.
fn wait_arg(timeout_ms: u32) -> String {
    if cfg!(target_os = "macos") {
        timeout_ms.to_string()
    } else {
        timeout_ms.div_ceil(1000).max(1).to_string()
    }
}

pub fn ping(ip: Ipv4Addr, timeout_ms: u32) -> Option<f32> {
    let output = Command::new("ping").args(["-c", "1", "-W", &wait_arg(timeout_ms), &ip.to_string()]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    parse_round_trip(&String::from_utf8_lossy(&output.stdout).to_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_round_trip_from_linux_and_macos_output() {
        let linux = "64 bytes from 155.133.226.1: icmp_seq=1 ttl=52 time=23.4 ms\n";
        let macos = "64 bytes from 155.133.226.1: icmp_seq=0 ttl=52 time=23.417 ms\n";
        assert_eq!(parse_round_trip(linux), Some(23.4));
        assert_eq!(parse_round_trip(macos), Some(23.417));
    }

    #[test]
    fn reads_a_sub_millisecond_reply() {
        assert_eq!(parse_round_trip("time=0.045 ms"), Some(0.045));
    }

    #[test]
    fn output_without_a_reply_has_no_round_trip() {
        assert_eq!(parse_round_trip("100% packet loss"), None);
        assert_eq!(parse_round_trip("time=abc ms"), None);
    }
}
