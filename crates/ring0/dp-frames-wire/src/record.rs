pub const HEADER_LEN: usize = 16;
pub const RECORD_LEN: usize = 16;
const MAGIC: [u8; 4] = *b"DPFT";
const VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub pid: u32,
}

/// One present call. `timestamp_ns` is a monotonic clock in nanoseconds, `swapchain` a hash of the
/// swapchain handle so several swapchains in one process can be told apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record {
    pub timestamp_ns: u64,
    pub swapchain: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireError {
    BadMagic,
    UnsupportedVersion(u32),
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadMagic => f.write_str("not a Deadlock+ frame file"),
            Self::UnsupportedVersion(v) => write!(f, "frame file version {v} is not supported"),
        }
    }
}

impl std::error::Error for WireError {}

pub fn encode_header(pid: u32) -> [u8; HEADER_LEN] {
    let mut out = [0u8; HEADER_LEN];
    out[..4].copy_from_slice(&MAGIC);
    out[4..8].copy_from_slice(&VERSION.to_le_bytes());
    out[8..12].copy_from_slice(&pid.to_le_bytes());
    out
}

pub fn encode_record(record: Record, out: &mut Vec<u8>) {
    out.extend_from_slice(&record.timestamp_ns.to_le_bytes());
    out.extend_from_slice(&record.swapchain.to_le_bytes());
    out.extend_from_slice(&[0u8; 4]);
}

/// Incremental parser for a file that is still being appended to: bytes may arrive split anywhere.
#[derive(Debug, Default)]
pub struct Reader {
    pending: Vec<u8>,
    header: Option<Header>,
    failed: Option<WireError>,
}

impl Reader {
    pub fn header(&self) -> Option<Header> {
        self.header
    }

    /// Appends the complete records found in `bytes` to `out`. After an error the reader ignores all input.
    pub fn feed(&mut self, bytes: &[u8], out: &mut Vec<Record>) -> Result<(), WireError> {
        if let Some(e) = self.failed {
            return Err(e);
        }
        self.pending.extend_from_slice(bytes);
        let mut at = 0;
        if self.header.is_none() {
            if self.pending.len() < HEADER_LEN {
                return Ok(());
            }
            let head = &self.pending[..HEADER_LEN];
            let version = u32::from_le_bytes([head[4], head[5], head[6], head[7]]);
            let error = if head[..4] != MAGIC {
                Some(WireError::BadMagic)
            } else if version != VERSION {
                Some(WireError::UnsupportedVersion(version))
            } else {
                None
            };
            if let Some(e) = error {
                self.failed = Some(e);
                self.pending.clear();
                return Err(e);
            }
            self.header = Some(Header { pid: u32::from_le_bytes([head[8], head[9], head[10], head[11]]) });
            at = HEADER_LEN;
        }
        while self.pending.len() - at >= RECORD_LEN {
            let r = &self.pending[at..at + RECORD_LEN];
            out.push(Record {
                timestamp_ns: u64::from_le_bytes([r[0], r[1], r[2], r[3], r[4], r[5], r[6], r[7]]),
                swapchain: u32::from_le_bytes([r[8], r[9], r[10], r[11]]),
            });
            at += RECORD_LEN;
        }
        self.pending.drain(..at);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(pid: u32, records: &[Record]) -> Vec<u8> {
        let mut bytes = encode_header(pid).to_vec();
        for r in records {
            encode_record(*r, &mut bytes);
        }
        bytes
    }

    fn rec(t: u64) -> Record {
        Record { timestamp_ns: t, swapchain: 7 }
    }

    #[test]
    fn round_trips_a_whole_file() {
        let records = [rec(1), rec(2), rec(u64::MAX)];
        let mut reader = Reader::default();
        let mut out = Vec::new();
        reader.feed(&file(42, &records), &mut out).unwrap();
        assert_eq!(reader.header(), Some(Header { pid: 42 }));
        assert_eq!(out, records);
    }

    #[test]
    fn splitting_the_bytes_anywhere_gives_the_same_records() {
        let records: Vec<Record> = (1..=5).map(rec).collect();
        let bytes = file(1, &records);
        for split in 0..=bytes.len() {
            let mut reader = Reader::default();
            let mut out = Vec::new();
            reader.feed(&bytes[..split], &mut out).unwrap();
            reader.feed(&bytes[split..], &mut out).unwrap();
            assert_eq!(out, records, "split at {split}");
        }
    }

    #[test]
    fn byte_at_a_time_works() {
        let records = [rec(10), rec(20)];
        let mut reader = Reader::default();
        let mut out = Vec::new();
        for b in file(9, &records) {
            reader.feed(&[b], &mut out).unwrap();
        }
        assert_eq!(out, records);
    }

    #[test]
    fn bad_magic_fails_and_stays_failed() {
        let mut bytes = file(1, &[rec(1)]);
        bytes[0] = b'X';
        let mut reader = Reader::default();
        let mut out = Vec::new();
        assert_eq!(reader.feed(&bytes, &mut out), Err(WireError::BadMagic));
        assert_eq!(reader.feed(&file(1, &[rec(2)]), &mut out), Err(WireError::BadMagic));
        assert!(out.is_empty());
    }

    #[test]
    fn newer_version_is_rejected() {
        let mut bytes = file(1, &[]);
        bytes[4] = 2;
        let mut reader = Reader::default();
        assert_eq!(reader.feed(&bytes, &mut Vec::new()), Err(WireError::UnsupportedVersion(2)));
    }

    #[test]
    fn a_partial_header_yields_nothing_and_no_error() {
        let mut reader = Reader::default();
        let mut out = Vec::new();
        assert_eq!(reader.feed(&encode_header(1)[..10], &mut out), Ok(()));
        assert_eq!(reader.header(), None);
        assert!(out.is_empty());
    }
}
