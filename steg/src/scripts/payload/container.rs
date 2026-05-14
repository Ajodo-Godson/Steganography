use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use std::io::{Read, Write};

use super::binary::{put_u16_be, put_u32_be, read_u16_be, read_u32_be};
use super::{
    ALGORITHM_DCT_LUMA, FIRST_CHUNK_INDEX, PAYLOAD_VERSION, RICH_MAGIC, STORAGE_RAW, STORAGE_ZLIB,
    WRAPPED_MAGIC,
};

fn compress_bytes(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(input)
        .map_err(|e| format!("Compression failed: {e}"))?;
    encoder
        .finish()
        .map_err(|e| format!("Compression failed: {e}"))
}

fn decompress_bytes(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = ZlibDecoder::new(input);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| format!("Decompression failed: {e}"))?;
    Ok(out)
}

struct ContainerHeader {
    storage_kind: u8,
    mime_type: String,
    chunk_index: u32,
    algorithm_id: u16,
    original_len: usize,
}

pub(crate) fn wrap_payload(inner: &[u8], mime_type: &str) -> Result<Vec<u8>, String> {
    let compressed = compress_bytes(inner)?;
    let (storage_kind, stored_bytes) = if compressed.len() < inner.len() {
        (STORAGE_ZLIB, compressed)
    } else {
        (STORAGE_RAW, inner.to_vec())
    };

    let original_len =
        u32::try_from(inner.len()).map_err(|_| "Payload too large (max 4 GiB-1)".to_string())?;
    let mime_len = u16::try_from(mime_type.len()).map_err(|_| "MIME type too long".to_string())?;
    let mut out = Vec::with_capacity(18 + mime_type.len() + stored_bytes.len());
    out.extend_from_slice(RICH_MAGIC);
    out.push(PAYLOAD_VERSION);
    out.push(storage_kind);
    put_u16_be(&mut out, mime_len);
    out.extend_from_slice(mime_type.as_bytes());
    put_u32_be(&mut out, FIRST_CHUNK_INDEX);
    put_u16_be(&mut out, ALGORITHM_DCT_LUMA);
    put_u32_be(&mut out, original_len);
    out.extend_from_slice(&stored_bytes);
    Ok(out)
}

pub(crate) fn unwrap_payload(raw: &[u8]) -> Result<Vec<u8>, String> {
    if raw.starts_with(RICH_MAGIC) {
        return unwrap_rich_payload(raw);
    }
    unwrap_v2_payload(raw)
}

fn unwrap_v2_payload(raw: &[u8]) -> Result<Vec<u8>, String> {
    if raw.len() < 10 {
        return Err("Invalid payload container: too short".to_string());
    }
    if &raw[0..5] != WRAPPED_MAGIC {
        return Err("Invalid payload container: bad magic".to_string());
    }

    let mut offset = 5;
    offset += 1;

    let header = ContainerHeader {
        storage_kind: raw[offset - 1],
        mime_type: String::new(),
        chunk_index: 0,
        algorithm_id: 0,
        original_len: read_u32_be(raw, &mut offset)? as usize,
    };
    let body = &raw[offset..];
    decode_body(&header, body)
}

fn unwrap_rich_payload(raw: &[u8]) -> Result<Vec<u8>, String> {
    if raw.len() < 18 {
        return Err("Invalid payload container: too short".to_string());
    }

    let mut offset = 5;
    let version = raw[offset];
    offset += 1;
    if version != PAYLOAD_VERSION {
        return Err("Invalid payload container: unsupported version".to_string());
    }

    let storage_kind = raw[offset];
    offset += 1;
    let mime_len = read_u16_be(raw, &mut offset)? as usize;
    if raw.len() < offset + mime_len + 10 {
        return Err("Invalid payload container: truncated metadata".to_string());
    }
    let mime_type = String::from_utf8(raw[offset..offset + mime_len].to_vec())
        .map_err(|_| "Invalid payload container: MIME type is not UTF-8".to_string())?;
    offset += mime_len;

    let header = ContainerHeader {
        storage_kind,
        mime_type,
        chunk_index: read_u32_be(raw, &mut offset)?,
        algorithm_id: read_u16_be(raw, &mut offset)?,
        original_len: read_u32_be(raw, &mut offset)? as usize,
    };
    let body = &raw[offset..];
    validate_metadata(&header)?;
    decode_body(&header, body)
}

fn decode_body(header: &ContainerHeader, body: &[u8]) -> Result<Vec<u8>, String> {
    let inner = match header.storage_kind {
        STORAGE_RAW => body.to_vec(),
        STORAGE_ZLIB => decompress_bytes(body)?,
        _ => return Err("Invalid payload container: unknown storage kind".to_string()),
    };

    if inner.len() != header.original_len {
        return Err(format!(
            "Invalid payload container: expected {} bytes after decode, got {}",
            header.original_len,
            inner.len()
        ));
    }
    Ok(inner)
}

fn validate_metadata(header: &ContainerHeader) -> Result<(), String> {
    if header.chunk_index != FIRST_CHUNK_INDEX || header.algorithm_id != ALGORITHM_DCT_LUMA {
        return Err("Invalid payload container: unsupported metadata".to_string());
    }
    if header.mime_type.is_empty() {
        return Err("Invalid payload container: empty MIME type".to_string());
    }
    Ok(())
}
