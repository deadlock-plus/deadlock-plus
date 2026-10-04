use serde_json::{json, Value};
use std::fmt;
use std::io::{self, Read, Write};

pub const OP_HANDSHAKE: u32 = 0;
pub const OP_FRAME: u32 = 1;
pub const OP_CLOSE: u32 = 2;
pub const OP_PING: u32 = 3;
pub const OP_PONG: u32 = 4;

/// Discord's own cap is far lower; this only stops a bad length from allocating gigabytes.
const MAX_PAYLOAD: u32 = 1 << 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Packet {
    pub opcode: u32,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Button {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Activity {
    pub details: Option<String>,
    pub state: Option<String>,
    pub start: Option<i64>,
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
    pub buttons: Vec<Button>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ready {
    pub user_id: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug)]
pub enum ProtoError {
    Truncated,
    TooLarge(u32),
    BadJson(String),
    Remote { code: i64, message: String },
    NotReady(String),
}

impl fmt::Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => write!(f, "truncated packet"),
            Self::TooLarge(n) => write!(f, "packet length {n} exceeds limit"),
            Self::BadJson(e) => write!(f, "invalid json: {e}"),
            Self::Remote { code, message } => write!(f, "discord error {code}: {message}"),
            Self::NotReady(what) => write!(f, "expected READY, got {what}"),
        }
    }
}

impl std::error::Error for ProtoError {}

pub fn encode(opcode: u32, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + payload.len());
    out.extend_from_slice(&opcode.to_le_bytes());
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
    out
}

fn header(h: [u8; 8]) -> Result<(u32, u32), ProtoError> {
    let opcode = u32::from_le_bytes([h[0], h[1], h[2], h[3]]);
    let len = u32::from_le_bytes([h[4], h[5], h[6], h[7]]);
    if len > MAX_PAYLOAD {
        return Err(ProtoError::TooLarge(len));
    }
    Ok((opcode, len))
}

/// Returns the packet and the number of bytes consumed, or `None` if `buf` holds only part of one.
pub fn decode(buf: &[u8]) -> Result<Option<(Packet, usize)>, ProtoError> {
    let Some(h) = buf.get(..8) else { return Ok(None) };
    let (opcode, len) = header(h.try_into().unwrap())?;
    let end = 8 + len as usize;
    let Some(payload) = buf.get(8..end) else { return Ok(None) };
    Ok(Some((Packet { opcode, payload: payload.to_vec() }, end)))
}

pub fn write_packet<W: Write>(w: &mut W, opcode: u32, payload: &[u8]) -> io::Result<()> {
    w.write_all(&encode(opcode, payload))?;
    w.flush()
}

pub fn read_packet<R: Read>(r: &mut R) -> io::Result<Packet> {
    let mut h = [0u8; 8];
    r.read_exact(&mut h)?;
    let (opcode, len) = header(h).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut payload = vec![0u8; len as usize];
    r.read_exact(&mut payload)?;
    Ok(Packet { opcode, payload })
}

pub fn handshake_payload(client_id: &str) -> Vec<u8> {
    json!({"v": 1, "client_id": client_id}).to_string().into_bytes()
}

fn parse(payload: &[u8]) -> Result<Value, ProtoError> {
    serde_json::from_slice(payload).map_err(|e| ProtoError::BadJson(e.to_string()))
}

fn remote_error(v: &Value) -> Option<ProtoError> {
    if v["evt"] != "ERROR" {
        return None;
    }
    let data = &v["data"];
    Some(ProtoError::Remote {
        code: data["code"].as_i64().unwrap_or(0),
        message: data["message"].as_str().unwrap_or_default().to_owned(),
    })
}

pub fn parse_ready(payload: &[u8]) -> Result<Ready, ProtoError> {
    let v = parse(payload)?;
    if let Some(e) = remote_error(&v) {
        return Err(e);
    }
    if v["evt"] != "READY" {
        return Err(ProtoError::NotReady(v["evt"].to_string()));
    }
    let user = &v["data"]["user"];
    Ok(Ready {
        user_id: user["id"].as_str().map(str::to_owned),
        username: user["username"].as_str().map(str::to_owned),
    })
}

fn activity_json(a: &Activity) -> Value {
    let mut out = serde_json::Map::new();
    if let Some(d) = &a.details {
        out.insert("details".into(), json!(d));
    }
    if let Some(s) = &a.state {
        out.insert("state".into(), json!(s));
    }
    if let Some(start) = a.start {
        out.insert("timestamps".into(), json!({"start": start}));
    }
    let mut assets = serde_json::Map::new();
    for (key, val) in [
        ("large_image", &a.large_image),
        ("large_text", &a.large_text),
        ("small_image", &a.small_image),
        ("small_text", &a.small_text),
    ] {
        if let Some(v) = val {
            assets.insert(key.into(), json!(v));
        }
    }
    if !assets.is_empty() {
        out.insert("assets".into(), Value::Object(assets));
    }
    if !a.buttons.is_empty() {
        let buttons: Vec<Value> = a.buttons.iter().map(|b| json!({"label": b.label, "url": b.url})).collect();
        out.insert("buttons".into(), Value::Array(buttons));
    }
    Value::Object(out)
}

fn command(pid: u32, activity: Value, nonce: &str) -> Vec<u8> {
    json!({"cmd": "SET_ACTIVITY", "nonce": nonce, "args": {"pid": pid, "activity": activity}}).to_string().into_bytes()
}

pub fn set_activity_payload(pid: u32, activity: &Activity, nonce: &str) -> Vec<u8> {
    command(pid, activity_json(activity), nonce)
}

pub fn clear_activity_payload(pid: u32, nonce: &str) -> Vec<u8> {
    command(pid, Value::Null, nonce)
}

/// Checks a command response frame: `Ok(nonce)` for success, `Err(Remote)` for an ERROR event.
pub fn parse_response(payload: &[u8]) -> Result<Option<String>, ProtoError> {
    let v = parse(payload)?;
    if let Some(e) = remote_error(&v) {
        return Err(e);
    }
    Ok(v["nonce"].as_str().map(str::to_owned))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json(p: &[u8]) -> Value {
        serde_json::from_slice(p).unwrap()
    }

    #[test]
    fn encode_layout() {
        let b = encode(1, b"{}");
        assert_eq!(b, [1, 0, 0, 0, 2, 0, 0, 0, b'{', b'}']);
    }

    #[test]
    fn decode_roundtrip() {
        let b = encode(OP_FRAME, b"hello");
        let (p, used) = decode(&b).unwrap().unwrap();
        assert_eq!(p, Packet { opcode: 1, payload: b"hello".to_vec() });
        assert_eq!(used, b.len());
    }

    #[test]
    fn decode_partial_and_trailing() {
        let mut b = encode(1, b"abc");
        assert!(decode(&b[..5]).unwrap().is_none());
        assert!(decode(&b[..10]).unwrap().is_none());
        b.extend_from_slice(&encode(2, b"x"));
        let (_, used) = decode(&b).unwrap().unwrap();
        assert_eq!(used, 11);
    }

    #[test]
    fn decode_rejects_oversize() {
        let mut b = vec![1, 0, 0, 0];
        b.extend_from_slice(&u32::MAX.to_le_bytes());
        assert!(matches!(decode(&b), Err(ProtoError::TooLarge(_))));
    }

    #[test]
    fn io_roundtrip() {
        let mut buf = Vec::new();
        write_packet(&mut buf, OP_PING, b"p").unwrap();
        let p = read_packet(&mut buf.as_slice()).unwrap();
        assert_eq!(p, Packet { opcode: OP_PING, payload: b"p".to_vec() });
    }

    #[test]
    fn read_packet_eof_is_error() {
        let b = encode(1, b"abc");
        assert!(read_packet(&mut &b[..9]).is_err());
    }

    #[test]
    fn handshake_is_v1_with_client_id() {
        assert_eq!(json(&handshake_payload("123")), json!({"v": 1, "client_id": "123"}));
    }

    #[test]
    fn ready_parses_user() {
        let p = br#"{"cmd":"DISPATCH","evt":"READY","data":{"v":1,"user":{"id":"9","username":"x"}}}"#;
        let r = parse_ready(p).unwrap();
        assert_eq!(r, Ready { user_id: Some("9".into()), username: Some("x".into()) });
    }

    #[test]
    fn ready_without_user_is_fine() {
        let r = parse_ready(br#"{"cmd":"DISPATCH","evt":"READY","data":{"v":1}}"#).unwrap();
        assert_eq!(r, Ready { user_id: None, username: None });
    }

    #[test]
    fn ready_rejects_other_events_and_errors() {
        assert!(matches!(
            parse_ready(br#"{"evt":"ERROR","data":{"code":4000,"message":"bad"}}"#),
            Err(ProtoError::Remote { code: 4000, .. })
        ));
        assert!(matches!(parse_ready(br#"{"evt":"OTHER"}"#), Err(ProtoError::NotReady(_))));
        assert!(matches!(parse_ready(b"nope"), Err(ProtoError::BadJson(_))));
    }

    #[test]
    fn buttons_are_sent_as_label_and_url() {
        let a = Activity {
            buttons: vec![Button { label: "Site".into(), url: "https://example.com".into() }],
            ..Default::default()
        };
        assert_eq!(
            json(&set_activity_payload(1, &a, "n"))["args"]["activity"]["buttons"],
            json!([{"label": "Site", "url": "https://example.com"}])
        );
    }

    #[test]
    fn no_buttons_omits_the_field() {
        let a = Activity { details: Some("d".into()), ..Default::default() };
        assert!(json(&set_activity_payload(1, &a, "n"))["args"]["activity"].get("buttons").is_none());
    }

    #[test]
    fn set_activity_full() {
        let a = Activity {
            details: Some("d".into()),
            state: Some("s".into()),
            start: Some(1700000000),
            large_image: Some("https://x/l.png".into()),
            large_text: Some("lt".into()),
            small_image: Some("https://x/s.png".into()),
            small_text: Some("st".into()),
            buttons: Vec::new(),
        };
        assert_eq!(
            json(&set_activity_payload(42, &a, "n1")),
            json!({
                "cmd": "SET_ACTIVITY",
                "nonce": "n1",
                "args": {
                    "pid": 42,
                    "activity": {
                        "details": "d",
                        "state": "s",
                        "timestamps": {"start": 1700000000},
                        "assets": {
                            "large_image": "https://x/l.png", "large_text": "lt",
                            "small_image": "https://x/s.png", "small_text": "st"
                        }
                    }
                }
            })
        );
    }

    #[test]
    fn set_activity_omits_unset_fields() {
        let a = Activity { details: Some("d".into()), ..Default::default() };
        let v = json(&set_activity_payload(1, &a, "n"));
        assert_eq!(v["args"]["activity"], json!({"details": "d"}));
    }

    #[test]
    fn set_activity_assets_only_when_present() {
        let a = Activity { small_image: Some("i".into()), ..Default::default() };
        let v = json(&set_activity_payload(1, &a, "n"));
        assert_eq!(v["args"]["activity"], json!({"assets": {"small_image": "i"}}));
    }

    #[test]
    fn clear_sends_null_activity() {
        let v = json(&clear_activity_payload(7, "n2"));
        assert_eq!(v, json!({"cmd": "SET_ACTIVITY", "nonce": "n2", "args": {"pid": 7, "activity": null}}));
    }

    #[test]
    fn response_ok_returns_nonce() {
        assert_eq!(
            parse_response(br#"{"cmd":"SET_ACTIVITY","data":{},"evt":null,"nonce":"n"}"#).unwrap(),
            Some("n".into())
        );
    }

    #[test]
    fn response_error_is_remote() {
        let e = parse_response(
            br#"{"cmd":"SET_ACTIVITY","evt":"ERROR","data":{"code":4000,"message":"nope"},"nonce":"n"}"#,
        )
        .unwrap_err();
        assert!(matches!(e, ProtoError::Remote { code: 4000, ref message } if message == "nope"));
    }
}
