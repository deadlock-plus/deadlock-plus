use std::collections::HashSet;
use std::net::{Ipv4Addr, SocketAddrV4};

const PROTOCOL_UDP: u8 = 17;

pub struct UdpPacket {
    pub src: SocketAddrV4,
    pub dst: SocketAddrV4,
}

/// Reads an IPv4 header and the UDP ports behind it. Later fragments carry no UDP header and are skipped.
pub fn parse_ipv4_udp(ip: &[u8]) -> Option<UdpPacket> {
    if ip.len() < 20 || ip[0] >> 4 != 4 {
        return None;
    }
    let header_len = usize::from(ip[0] & 0x0f) * 4;
    if header_len < 20 || ip.len() < header_len + 8 || ip[9] != PROTOCOL_UDP {
        return None;
    }
    if u16::from_be_bytes([ip[6], ip[7]]) & 0x1fff != 0 {
        return None;
    }
    let udp = &ip[header_len..];
    Some(UdpPacket {
        src: SocketAddrV4::new(Ipv4Addr::new(ip[12], ip[13], ip[14], ip[15]), u16::from_be_bytes([udp[0], udp[1]])),
        dst: SocketAddrV4::new(Ipv4Addr::new(ip[16], ip[17], ip[18], ip[19]), u16::from_be_bytes([udp[2], udp[3]])),
    })
}

/// The far end and whether the packet is coming in, judged by which side is one of this machine's addresses.
/// Traffic between two local addresses, or two foreign ones, is not ours to report.
pub fn direction(packet: &UdpPacket, local: &HashSet<Ipv4Addr>) -> Option<(SocketAddrV4, bool)> {
    match (local.contains(packet.src.ip()), local.contains(packet.dst.ip())) {
        (true, false) => Some((packet.dst, false)),
        (false, true) => Some((packet.src, true)),
        _ => None,
    }
}

pub fn encode_packet(inbound: bool, remote: SocketAddrV4, ticks_100ns: i64) -> String {
    format!("P {} {} {} {ticks_100ns}\n", if inbound { 'i' } else { 'o' }, remote.ip(), remote.port())
}

pub fn decode_packet(line: &str) -> Option<(bool, SocketAddrV4, i64)> {
    let mut parts = line.trim_end().split(' ');
    if parts.next()? != "P" {
        return None;
    }
    let inbound = match parts.next()? {
        "i" => true,
        "o" => false,
        _ => return None,
    };
    let remote = SocketAddrV4::new(parts.next()?.parse().ok()?, parts.next()?.parse().ok()?);
    let ticks = parts.next()?.parse().ok()?;
    parts.next().is_none().then_some((inbound, remote, ticks))
}

pub fn encode_error(message: &str) -> String {
    format!("E {}\n", message.replace(['\n', '\r'], " "))
}

pub fn decode_error(line: &str) -> Option<String> {
    line.trim_end().strip_prefix("E ").map(str::to_string)
}

pub fn encode_filter(ips: &[Ipv4Addr]) -> String {
    let list: Vec<String> = ips.iter().map(Ipv4Addr::to_string).collect();
    format!("F {}\n", list.join(","))
}

pub fn decode_filter(line: &str) -> Option<HashSet<Ipv4Addr>> {
    let rest = line.trim_end().strip_prefix('F')?;
    let list = if rest.is_empty() { rest } else { rest.strip_prefix(' ')? };
    list.split(',').filter(|s| !s.is_empty()).map(|s| s.parse().ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip(a: u8, b: u8, c: u8, d: u8) -> Ipv4Addr {
        Ipv4Addr::new(a, b, c, d)
    }

    fn datagram(src: (Ipv4Addr, u16), dst: (Ipv4Addr, u16), header_words: u8, fragment: u16) -> Vec<u8> {
        let header_len = usize::from(header_words) * 4;
        let mut p = vec![0u8; header_len + 12];
        p[0] = 0x40 | header_words;
        p[6..8].copy_from_slice(&fragment.to_be_bytes());
        p[9] = PROTOCOL_UDP;
        p[12..16].copy_from_slice(&src.0.octets());
        p[16..20].copy_from_slice(&dst.0.octets());
        p[header_len..header_len + 2].copy_from_slice(&src.1.to_be_bytes());
        p[header_len + 2..header_len + 4].copy_from_slice(&dst.1.to_be_bytes());
        p
    }

    fn local() -> HashSet<Ipv4Addr> {
        HashSet::from([ip(192, 168, 1, 5)])
    }

    #[test]
    fn reads_addresses_and_ports_from_a_plain_header() {
        let p = parse_ipv4_udp(&datagram((ip(192, 168, 1, 5), 50000), (ip(155, 133, 226, 1), 27015), 5, 0)).unwrap();
        assert_eq!(p.src, SocketAddrV4::new(ip(192, 168, 1, 5), 50000));
        assert_eq!(p.dst, SocketAddrV4::new(ip(155, 133, 226, 1), 27015));
    }

    #[test]
    fn finds_the_udp_header_after_ip_options() {
        let p = parse_ipv4_udp(&datagram((ip(1, 1, 1, 1), 1111), (ip(2, 2, 2, 2), 2222), 7, 0)).unwrap();
        assert_eq!((p.src.port(), p.dst.port()), (1111, 2222));
    }

    #[test]
    fn skips_other_protocols_fragments_and_junk() {
        let mut tcp = datagram((ip(1, 1, 1, 1), 1), (ip(2, 2, 2, 2), 2), 5, 0);
        tcp[9] = 6;
        assert!(parse_ipv4_udp(&tcp).is_none());
        assert!(parse_ipv4_udp(&datagram((ip(1, 1, 1, 1), 1), (ip(2, 2, 2, 2), 2), 5, 185)).is_none());
        let mut v6 = datagram((ip(1, 1, 1, 1), 1), (ip(2, 2, 2, 2), 2), 5, 0);
        v6[0] = 0x65;
        assert!(parse_ipv4_udp(&v6).is_none());
        assert!(parse_ipv4_udp(&[0x45; 10]).is_none());
        assert!(parse_ipv4_udp(&datagram((ip(1, 1, 1, 1), 1), (ip(2, 2, 2, 2), 2), 5, 0)[..24]).is_none());
    }

    #[test]
    fn a_first_fragment_still_counts() {
        let more_fragments = 0x2000;
        assert!(parse_ipv4_udp(&datagram((ip(1, 1, 1, 1), 1), (ip(2, 2, 2, 2), 2), 5, more_fragments)).is_some());
    }

    #[test]
    fn direction_names_the_far_end_by_which_side_is_ours() {
        let out = parse_ipv4_udp(&datagram((ip(192, 168, 1, 5), 50000), (ip(155, 133, 226, 1), 27015), 5, 0)).unwrap();
        let inn = parse_ipv4_udp(&datagram((ip(155, 133, 226, 1), 27015), (ip(192, 168, 1, 5), 50000), 5, 0)).unwrap();
        let relay = SocketAddrV4::new(ip(155, 133, 226, 1), 27015);
        assert_eq!(direction(&out, &local()), Some((relay, false)));
        assert_eq!(direction(&inn, &local()), Some((relay, true)));
    }

    #[test]
    fn direction_ignores_traffic_that_is_not_between_us_and_someone_else() {
        let lan = HashSet::from([ip(192, 168, 1, 5), ip(192, 168, 1, 6)]);
        let inside = parse_ipv4_udp(&datagram((ip(192, 168, 1, 5), 1), (ip(192, 168, 1, 6), 2), 5, 0)).unwrap();
        let foreign = parse_ipv4_udp(&datagram((ip(8, 8, 8, 8), 1), (ip(9, 9, 9, 9), 2), 5, 0)).unwrap();
        assert_eq!(direction(&inside, &lan), None);
        assert_eq!(direction(&foreign, &lan), None);
    }

    #[test]
    fn packet_lines_round_trip() {
        let remote = SocketAddrV4::new(ip(155, 133, 226, 1), 27015);
        assert_eq!(decode_packet(&encode_packet(true, remote, 123_456)), Some((true, remote, 123_456)));
        assert_eq!(decode_packet(&encode_packet(false, remote, -5)), Some((false, remote, -5)));
    }

    #[test]
    fn malformed_packet_lines_are_rejected() {
        for line in [
            "",
            "P",
            "P x 1.1.1.1 1 1",
            "P i 1.1.1.1 99999 1",
            "P i nope 1 1",
            "P i 1.1.1.1 1 1 extra",
            "Q i 1.1.1.1 1 1",
        ] {
            assert_eq!(decode_packet(line), None, "{line:?}");
        }
    }

    #[test]
    fn errors_stay_on_one_line() {
        assert_eq!(decode_error(&encode_error("no access\nsecond line")), Some("no access second line".into()));
        assert_eq!(decode_error("P i 1.1.1.1 1 1"), None);
    }

    #[test]
    fn filters_round_trip_including_the_empty_one() {
        let ips = [ip(1, 1, 1, 1), ip(2, 2, 2, 2)];
        assert_eq!(decode_filter(&encode_filter(&ips)), Some(ips.into_iter().collect()));
        assert_eq!(decode_filter(&encode_filter(&[])), Some(HashSet::new()));
        assert_eq!(decode_filter("F 1.1.1.1,nope\n"), None);
        assert_eq!(decode_filter("P i 1.1.1.1 1 1"), None);
    }
}
