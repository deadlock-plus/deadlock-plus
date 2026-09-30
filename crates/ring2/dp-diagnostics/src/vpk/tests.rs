use super::*;

const ABC_CRC: u32 = 0x3524_41C2;

fn header(tree_len: usize, data_len: usize) -> Vec<u8> {
    let mut h = Vec::new();
    h.extend(0x55AA_1234u32.to_le_bytes());
    h.extend(2u32.to_le_bytes());
    h.extend((tree_len as u32).to_le_bytes());
    h.extend((data_len as u32).to_le_bytes());
    h.extend([0u8; 12]);
    h
}

fn dir_with_entry(archive_index: u16, offset: u32, length: u32, preload: &[u8], data: &[u8]) -> Vec<u8> {
    let mut tree = Vec::new();
    for s in ["vdata", "scripts", "heroes"] {
        tree.extend(s.as_bytes());
        tree.push(0);
    }
    tree.extend(ABC_CRC.to_le_bytes());
    tree.extend((preload.len() as u16).to_le_bytes());
    tree.extend(archive_index.to_le_bytes());
    tree.extend(offset.to_le_bytes());
    tree.extend(length.to_le_bytes());
    tree.extend(0xFFFFu16.to_le_bytes());
    tree.extend(preload);
    tree.extend([0u8, 0, 0]);
    let mut out = header(tree.len(), data.len());
    out.extend(tree);
    out.extend(data);
    out
}

#[test]
fn crc32_matches_known_value() {
    assert_eq!(crc32(b"abc"), ABC_CRC);
    assert_eq!(crc32(b"hello"), 0x3610_A686);
}

#[test]
fn parses_hand_built_embedded_archive() {
    let bytes = dir_with_entry(0x7FFF, 0, 3, &[], b"abc");
    let vpk = Vpk::parse(&bytes).unwrap();
    assert_eq!(vpk.entries.len(), 1);
    let e = &vpk.entries[0];
    assert_eq!(e.path, "scripts/heroes.vdata");
    assert_eq!(e.crc, ABC_CRC);
    assert_eq!(vpk.read(&bytes, e, |_| None).unwrap(), b"abc");
}

#[test]
fn reads_from_external_archive() {
    let bytes = dir_with_entry(0, 2, 3, &[], &[]);
    let vpk = Vpk::parse(&bytes).unwrap();
    let archive: &[u8] = b"..abc..";
    let data = vpk.read(&bytes, &vpk.entries[0], |i| (i == 0).then_some(archive));
    assert_eq!(data.unwrap(), b"abc");
}

#[test]
fn preload_bytes_come_first() {
    let bytes = dir_with_entry(0, 0, 1, b"ab", &[]);
    let vpk = Vpk::parse(&bytes).unwrap();
    let archive: &[u8] = b"c";
    let data = vpk.read(&bytes, &vpk.entries[0], |_| Some(archive));
    assert_eq!(data.unwrap(), b"abc");
}

#[test]
fn missing_archive_or_out_of_range_read_is_none() {
    let bytes = dir_with_entry(3, 0, 3, &[], &[]);
    let vpk = Vpk::parse(&bytes).unwrap();
    assert!(vpk.read(&bytes, &vpk.entries[0], |_| None).is_none());
    let short: &[u8] = b"ab";
    assert!(vpk.read(&bytes, &vpk.entries[0], |_| Some(short)).is_none());
}

#[test]
fn write_then_parse_round_trips_contents() {
    let files: [(&str, &[u8]); 4] = [
        ("scripts/heroes.vdata", b"hero data"),
        ("scripts/abilities.vdata", b"ability data"),
        ("readme.txt", b"root file"),
        ("dir/sub/noext", b""),
    ];
    let bytes = write(&files);
    let vpk = Vpk::parse(&bytes).unwrap();
    assert_eq!(vpk.entries.len(), files.len());
    for (path, data) in files {
        let e = vpk.entries.iter().find(|e| e.path == path).unwrap_or_else(|| panic!("missing {path}"));
        assert_eq!(e.crc, crc32(data));
        assert_eq!(vpk.read(&bytes, e, |_| None).unwrap(), data);
    }
}

#[test]
fn write_normalises_backslashes() {
    let bytes = write(&[("scripts\\heroes.vdata", b"x")]);
    let vpk = Vpk::parse(&bytes).unwrap();
    assert_eq!(vpk.entries[0].path, "scripts/heroes.vdata");
}

#[test]
fn prefix_len_covers_header_and_tree_only() {
    let bytes = dir_with_entry(0x7FFF, 0, 3, &[], b"abc");
    let len = Vpk::prefix_len(&bytes).unwrap();
    assert_eq!(len, bytes.len() - 3);
    assert_eq!(Vpk::prefix_len(&bytes[..12]).unwrap(), len);
    assert_eq!(Vpk::prefix_len(&bytes[..11]).unwrap_err(), VpkError::Truncated);
}

#[test]
fn parses_from_prefix_and_locates_embedded_entry() {
    let bytes = dir_with_entry(0x7FFF, 1, 3, b"P", b".abc");
    let prefix = &bytes[..Vpk::prefix_len(&bytes).unwrap()];
    let vpk = Vpk::parse(prefix).unwrap();
    let e = &vpk.entries[0];
    assert_eq!(&prefix[e.preload.clone()], b"P");
    let range = vpk.embedded_range(e).unwrap();
    assert_eq!(&bytes[range.start as usize..range.end as usize], b"abc");
}

#[test]
fn embedded_range_is_none_for_split_archives() {
    let bytes = dir_with_entry(0, 0, 3, &[], &[]);
    let vpk = Vpk::parse(&bytes).unwrap();
    assert!(vpk.embedded_range(&vpk.entries[0]).is_none());
}

#[test]
fn rejects_bad_signature() {
    let mut bytes = write(&[("a.txt", b"x")]);
    bytes[0] ^= 0xFF;
    assert_eq!(Vpk::parse(&bytes).unwrap_err(), VpkError::BadSignature);
}

#[test]
fn rejects_unsupported_version() {
    let mut bytes = write(&[("a.txt", b"x")]);
    bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    assert_eq!(Vpk::parse(&bytes).unwrap_err(), VpkError::UnsupportedVersion(1));
}

#[test]
fn rejects_truncated_tree() {
    let bytes = write(&[("scripts/heroes.vdata", b"x")]);
    assert_eq!(Vpk::parse(&bytes[..34]).unwrap_err(), VpkError::Truncated);
    assert_eq!(Vpk::parse(&bytes[..10]).unwrap_err(), VpkError::Truncated);
}
