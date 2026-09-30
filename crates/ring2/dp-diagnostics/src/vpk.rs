use std::collections::BTreeMap;
use std::ops::Range;

const SIGNATURE: u32 = 0x55AA_1234;
const HEADER_LEN: usize = 28;
const EMBEDDED: u16 = 0x7FFF;
const ENTRY_END: u16 = 0xFFFF;
/// The tree cannot hold an empty string (it terminates a level), so "no directory" and
/// "no extension" are both stored as a single space.
const NONE: &str = " ";

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum VpkError {
    #[error("not a VPK file")]
    BadSignature,
    #[error("unsupported VPK version {0}")]
    UnsupportedVersion(u32),
    #[error("VPK is truncated or malformed")]
    Truncated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: String,
    pub crc: u32,
    /// Bytes stored inline in the directory tree, ahead of the archive data.
    pub preload: Range<usize>,
    pub archive_index: u16,
    pub offset: u32,
    pub length: u32,
}

#[derive(Debug, Clone)]
pub struct Vpk {
    pub entries: Vec<Entry>,
    data_start: usize,
}

pub fn crc32(data: &[u8]) -> u32 {
    let mut crc = flate2::Crc::new();
    crc.update(data);
    crc.sum()
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], VpkError> {
        let end = self.pos.checked_add(n).ok_or(VpkError::Truncated)?;
        let slice = self.bytes.get(self.pos..end).ok_or(VpkError::Truncated)?;
        self.pos = end;
        Ok(slice)
    }

    fn u16(&mut self) -> Result<u16, VpkError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    fn u32(&mut self) -> Result<u32, VpkError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    fn cstr(&mut self) -> Result<String, VpkError> {
        let rest = self.bytes.get(self.pos..).ok_or(VpkError::Truncated)?;
        let len = rest.iter().position(|&b| b == 0).ok_or(VpkError::Truncated)?;
        self.pos += len + 1;
        Ok(String::from_utf8_lossy(&rest[..len]).into_owned())
    }
}

impl Vpk {
    /// Bytes of header plus directory tree, read from at least the first 12 bytes. Lets a caller
    /// read just that prefix of a large file and hand it to [`Vpk::parse`].
    pub fn prefix_len(head: &[u8]) -> Result<usize, VpkError> {
        let mut head = Reader { bytes: head, pos: 0 };
        if head.u32()? != SIGNATURE {
            return Err(VpkError::BadSignature);
        }
        let version = head.u32()?;
        if version != 2 {
            return Err(VpkError::UnsupportedVersion(version));
        }
        let tree_size = head.u32()? as usize;
        HEADER_LEN.checked_add(tree_size).ok_or(VpkError::Truncated)
    }

    /// Accepts either the whole file or just the [`Vpk::prefix_len`] prefix.
    pub fn parse(dir: &[u8]) -> Result<Vpk, VpkError> {
        let data_start = Self::prefix_len(dir)?;
        let tree = dir.get(..data_start).ok_or(VpkError::Truncated)?;

        let mut r = Reader { bytes: tree, pos: HEADER_LEN };
        let mut entries = Vec::new();
        loop {
            let ext = r.cstr()?;
            if ext.is_empty() {
                break;
            }
            loop {
                let folder = r.cstr()?;
                if folder.is_empty() {
                    break;
                }
                loop {
                    let name = r.cstr()?;
                    if name.is_empty() {
                        break;
                    }
                    let crc = r.u32()?;
                    let preload_len = r.u16()? as usize;
                    let archive_index = r.u16()?;
                    let offset = r.u32()?;
                    let length = r.u32()?;
                    if r.u16()? != ENTRY_END {
                        return Err(VpkError::Truncated);
                    }
                    let preload = r.pos..r.pos + preload_len;
                    r.take(preload_len)?;
                    entries.push(Entry {
                        path: join_path(&folder, &name, &ext),
                        crc,
                        preload,
                        archive_index,
                        offset,
                        length,
                    });
                }
            }
        }
        Ok(Vpk { entries, data_start })
    }

    /// Absolute byte range of an embedded entry's data within the dir file, excluding preload.
    /// `None` for entries stored in a numbered archive.
    pub fn embedded_range(&self, entry: &Entry) -> Option<Range<u64>> {
        if entry.archive_index != EMBEDDED {
            return None;
        }
        let start = self.data_start as u64 + u64::from(entry.offset);
        Some(start..start + u64::from(entry.length))
    }

    /// `None` if the archive is unavailable or the entry points outside its bounds.
    pub fn read<'a>(&self, dir: &'a [u8], entry: &Entry, archive: impl Fn(u16) -> Option<&'a [u8]>) -> Option<Vec<u8>> {
        let mut out = dir.get(entry.preload.clone())?.to_vec();
        let (source, start) = if entry.archive_index == EMBEDDED {
            (dir, self.data_start.checked_add(entry.offset as usize)?)
        } else {
            (archive(entry.archive_index)?, entry.offset as usize)
        };
        let end = start.checked_add(entry.length as usize)?;
        out.extend_from_slice(source.get(start..end)?);
        Some(out)
    }
}

fn join_path(folder: &str, name: &str, ext: &str) -> String {
    let mut path = String::new();
    if folder != NONE {
        path.push_str(folder);
        path.push('/');
    }
    path.push_str(name);
    if ext != NONE {
        path.push('.');
        path.push_str(ext);
    }
    path
}

fn split_path(path: &str) -> (String, String, String) {
    let path = path.replace('\\', "/");
    let (folder, file) = path.rsplit_once('/').unwrap_or((NONE, &path));
    match file.rsplit_once('.') {
        Some((name, ext)) if !name.is_empty() && !ext.is_empty() => {
            (ext.to_string(), folder.to_string(), name.to_string())
        }
        _ => (NONE.to_string(), folder.to_string(), file.to_string()),
    }
}

struct Placed {
    name: String,
    crc: u32,
    offset: u32,
    length: u32,
}

pub fn write(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut data = Vec::new();
    let mut tree_map: BTreeMap<String, BTreeMap<String, Vec<Placed>>> = BTreeMap::new();
    for (path, contents) in files {
        let (ext, folder, name) = split_path(path);
        tree_map.entry(ext).or_default().entry(folder).or_default().push(Placed {
            name,
            crc: crc32(contents),
            offset: data.len() as u32,
            length: contents.len() as u32,
        });
        data.extend_from_slice(contents);
    }

    let mut tree = Vec::new();
    for (ext, folders) in &tree_map {
        push_cstr(&mut tree, ext);
        for (folder, placed) in folders {
            push_cstr(&mut tree, folder);
            for file in placed {
                push_cstr(&mut tree, &file.name);
                tree.extend(file.crc.to_le_bytes());
                tree.extend(0u16.to_le_bytes());
                tree.extend(EMBEDDED.to_le_bytes());
                tree.extend(file.offset.to_le_bytes());
                tree.extend(file.length.to_le_bytes());
                tree.extend(ENTRY_END.to_le_bytes());
            }
            tree.push(0);
        }
        tree.push(0);
    }
    tree.push(0);

    let mut out = Vec::with_capacity(HEADER_LEN + tree.len() + data.len());
    out.extend(SIGNATURE.to_le_bytes());
    out.extend(2u32.to_le_bytes());
    out.extend((tree.len() as u32).to_le_bytes());
    out.extend((data.len() as u32).to_le_bytes());
    out.extend([0u8; 12]);
    out.extend(tree);
    out.extend(data);
    out
}

fn push_cstr(out: &mut Vec<u8>, s: &str) {
    out.extend(s.as_bytes());
    out.push(0);
}

#[cfg(test)]
mod tests;
