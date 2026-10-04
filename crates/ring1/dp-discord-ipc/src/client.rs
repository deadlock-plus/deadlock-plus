use crate::proto::{self, Activity, Packet, ProtoError, Ready};
use std::fmt;
use std::io::{self, Read, Write};

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Proto(ProtoError),
    Closed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io: {e}"),
            Self::Proto(e) => write!(f, "{e}"),
            Self::Closed => write!(f, "connection closed by discord"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<ProtoError> for Error {
    fn from(e: ProtoError) -> Self {
        Self::Proto(e)
    }
}

pub trait Transport {
    fn send(&mut self, opcode: u32, payload: &[u8]) -> io::Result<()>;
    fn recv(&mut self) -> io::Result<Packet>;
}

impl<T: Read + Write> Transport for T {
    fn send(&mut self, opcode: u32, payload: &[u8]) -> io::Result<()> {
        proto::write_packet(self, opcode, payload)
    }

    fn recv(&mut self) -> io::Result<Packet> {
        proto::read_packet(self)
    }
}

pub struct Client<T: Transport> {
    transport: T,
    ready: Ready,
    pid: u32,
    next_nonce: u64,
}

impl<T: Transport> Client<T> {
    pub fn handshake(mut transport: T, client_id: &str) -> Result<Self, Error> {
        transport.send(proto::OP_HANDSHAKE, &proto::handshake_payload(client_id))?;
        let ready = loop {
            let packet = transport.recv()?;
            match packet.opcode {
                proto::OP_CLOSE => return Err(Error::Closed),
                proto::OP_PING => transport.send(proto::OP_PONG, &packet.payload)?,
                _ => break proto::parse_ready(&packet.payload)?,
            }
        };
        Ok(Self { transport, ready, pid: std::process::id(), next_nonce: 1 })
    }

    pub fn ready(&self) -> &Ready {
        &self.ready
    }

    pub fn set_activity(&mut self, activity: &Activity) -> Result<(), Error> {
        let nonce = self.nonce();
        self.command(&proto::set_activity_payload(self.pid, activity, &nonce), &nonce)
    }

    pub fn clear_activity(&mut self) -> Result<(), Error> {
        let nonce = self.nonce();
        self.command(&proto::clear_activity_payload(self.pid, &nonce), &nonce)
    }

    fn nonce(&mut self) -> String {
        let n = self.next_nonce;
        self.next_nonce += 1;
        n.to_string()
    }

    fn command(&mut self, payload: &[u8], nonce: &str) -> Result<(), Error> {
        self.transport.send(proto::OP_FRAME, payload)?;
        loop {
            let packet = self.transport.recv()?;
            match packet.opcode {
                proto::OP_CLOSE => return Err(Error::Closed),
                proto::OP_PING => self.transport.send(proto::OP_PONG, &packet.payload)?,
                proto::OP_FRAME if proto::parse_response(&packet.payload)?.as_deref() == Some(nonce) => return Ok(()),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod fake {
    use super::*;
    use std::collections::VecDeque;

    #[derive(Default)]
    pub struct FakeTransport {
        pub sent: Vec<Packet>,
        pub replies: VecDeque<Packet>,
    }

    impl FakeTransport {
        pub fn reply(mut self, opcode: u32, json: &str) -> Self {
            self.replies.push_back(Packet { opcode, payload: json.as_bytes().to_vec() });
            self
        }
    }

    impl Transport for FakeTransport {
        fn send(&mut self, opcode: u32, payload: &[u8]) -> io::Result<()> {
            self.sent.push(Packet { opcode, payload: payload.to_vec() });
            Ok(())
        }

        fn recv(&mut self) -> io::Result<Packet> {
            self.replies.pop_front().ok_or_else(|| io::Error::from(io::ErrorKind::UnexpectedEof))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::FakeTransport;
    use super::*;
    use serde_json::Value;

    const READY: &str = r#"{"cmd":"DISPATCH","evt":"READY","data":{"v":1,"user":{"id":"9","username":"u"}}}"#;

    fn ok(nonce: &str) -> String {
        format!(r#"{{"cmd":"SET_ACTIVITY","evt":null,"data":{{}},"nonce":"{nonce}"}}"#)
    }

    fn body(p: &Packet) -> Value {
        serde_json::from_slice(&p.payload).unwrap()
    }

    #[test]
    fn handshake_sends_v1_and_reads_ready() {
        let c = Client::handshake(FakeTransport::default().reply(proto::OP_FRAME, READY), "123").unwrap();
        assert_eq!(c.ready().username.as_deref(), Some("u"));
        assert_eq!(c.transport.sent.len(), 1);
        assert_eq!(c.transport.sent[0].opcode, proto::OP_HANDSHAKE);
        assert_eq!(body(&c.transport.sent[0])["client_id"], "123");
    }

    #[test]
    fn handshake_fails_on_error_event() {
        let t = FakeTransport::default()
            .reply(proto::OP_FRAME, r#"{"evt":"ERROR","data":{"code":4000,"message":"Invalid Client ID"}}"#);
        assert!(matches!(Client::handshake(t, "x"), Err(Error::Proto(ProtoError::Remote { code: 4000, .. }))));
    }

    #[test]
    fn handshake_fails_on_close() {
        let t = FakeTransport::default().reply(proto::OP_CLOSE, r#"{"code":4000,"message":"x"}"#);
        assert!(matches!(Client::handshake(t, "x"), Err(Error::Closed)));
    }

    #[test]
    fn set_activity_sends_frame_with_pid_and_nonce() {
        let t = FakeTransport::default().reply(proto::OP_FRAME, READY).reply(proto::OP_FRAME, &ok("1"));
        let mut c = Client::handshake(t, "1").unwrap();
        c.set_activity(&Activity { details: Some("hi".into()), ..Default::default() }).unwrap();
        let sent = &c.transport.sent[1];
        assert_eq!(sent.opcode, proto::OP_FRAME);
        let v = body(sent);
        assert_eq!(v["cmd"], "SET_ACTIVITY");
        assert_eq!(v["args"]["pid"], std::process::id());
        assert_eq!(v["args"]["activity"]["details"], "hi");
        assert_eq!(v["nonce"], "1");
    }

    #[test]
    fn nonces_are_unique_per_command() {
        let t = FakeTransport::default()
            .reply(proto::OP_FRAME, READY)
            .reply(proto::OP_FRAME, &ok("1"))
            .reply(proto::OP_FRAME, &ok("2"));
        let mut c = Client::handshake(t, "1").unwrap();
        c.set_activity(&Activity::default()).unwrap();
        c.clear_activity().unwrap();
        assert_ne!(body(&c.transport.sent[1])["nonce"], body(&c.transport.sent[2])["nonce"]);
    }

    #[test]
    fn clear_sends_null_activity() {
        let t = FakeTransport::default().reply(proto::OP_FRAME, READY).reply(proto::OP_FRAME, &ok("1"));
        let mut c = Client::handshake(t, "1").unwrap();
        c.clear_activity().unwrap();
        assert!(body(&c.transport.sent[1])["args"]["activity"].is_null());
    }

    #[test]
    fn remote_error_surfaces() {
        let t = FakeTransport::default()
            .reply(proto::OP_FRAME, READY)
            .reply(proto::OP_FRAME, r#"{"evt":"ERROR","data":{"code":4000,"message":"bad"},"nonce":"1"}"#);
        let mut c = Client::handshake(t, "1").unwrap();
        assert!(matches!(c.set_activity(&Activity::default()), Err(Error::Proto(ProtoError::Remote { .. }))));
    }

    #[test]
    fn ping_is_answered_with_pong_and_skipped() {
        let t = FakeTransport::default()
            .reply(proto::OP_FRAME, READY)
            .reply(proto::OP_PING, "{}")
            .reply(proto::OP_FRAME, &ok("1"));
        let mut c = Client::handshake(t, "1").unwrap();
        c.set_activity(&Activity::default()).unwrap();
        assert_eq!(c.transport.sent.last().unwrap().opcode, proto::OP_PONG);
    }

    #[test]
    fn io_failure_surfaces() {
        let mut c = Client::handshake(FakeTransport::default().reply(proto::OP_FRAME, READY), "1").unwrap();
        assert!(matches!(c.set_activity(&Activity::default()), Err(Error::Io(_))));
    }

    #[test]
    fn works_over_plain_read_write() {
        struct Duplex {
            input: io::Cursor<Vec<u8>>,
            output: Vec<u8>,
        }
        impl Read for Duplex {
            fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
                self.input.read(b)
            }
        }
        impl Write for Duplex {
            fn write(&mut self, b: &[u8]) -> io::Result<usize> {
                self.output.write(b)
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let d = Duplex { input: io::Cursor::new(proto::encode(proto::OP_FRAME, READY.as_bytes())), output: Vec::new() };
        let c = Client::handshake(d, "5").unwrap();
        assert!(c.transport.output.starts_with(&0u32.to_le_bytes()));
    }
}
