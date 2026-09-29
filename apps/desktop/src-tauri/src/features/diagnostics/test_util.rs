//! Builders for synthetic compiled-script resources.

pub fn block(tag: &[u8; 4], payload: &[u8]) -> (Vec<u8>, Vec<u8>) {
    (tag.to_vec(), payload.to_vec())
}

/// Resource wrapper: size, header version, version, table offset, table count, then the table
/// (tag, offset relative to the offset field, size) and the block payloads.
pub fn vjs_c(blocks: &[(Vec<u8>, Vec<u8>)]) -> Vec<u8> {
    let table_start = 16;
    let payload_start = table_start + blocks.len() * 12;
    let mut table = Vec::new();
    let mut payloads: Vec<u8> = Vec::new();
    for (i, (tag, payload)) in blocks.iter().enumerate() {
        let field_pos = table_start + i * 12 + 4;
        let offset = payload_start + payloads.len() - field_pos;
        table.extend(tag);
        table.extend((offset as u32).to_le_bytes());
        table.extend((payload.len() as u32).to_le_bytes());
        payloads.extend(payload);
    }
    let mut out = Vec::new();
    out.extend(0u32.to_le_bytes());
    out.extend(12u16.to_le_bytes());
    out.extend(4u16.to_le_bytes());
    out.extend(8u32.to_le_bytes());
    out.extend((blocks.len() as u32).to_le_bytes());
    out.extend(table);
    out.extend(payloads);
    let len = out.len() as u32;
    out[..4].copy_from_slice(&len.to_le_bytes());
    out
}
