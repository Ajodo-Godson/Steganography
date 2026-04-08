use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use std::io::{Read, Write};

const LEGACY_MAGIC: &[u8; 5] = b"STEG1";
const WRAPPED_MAGIC: &[u8; 5] = b"STEG2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Text(String),
    File { name: String, bytes: Vec<u8> },
}

const KIND_TEXT: u8 = 1;
const KIND_FILE: u8 = 2;

const STORAGE_RAW: u8 = 0;
const STORAGE_ZLIB: u8 = 1;

fn put_u16_be(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn put_u32_be(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

fn read_u16_be(input: &[u8], offset: &mut usize) -> Result<u16, String> {
    if input.len() < *offset + 2 {
        return Err("Invalid payload: truncated u16".to_string());
    }
    let v = u16::from_be_bytes([input[*offset], input[*offset + 1]]);
    *offset += 2;
    Ok(v)
}

fn read_u32_be(input: &[u8], offset: &mut usize) -> Result<u32, String> {
    if input.len() < *offset + 4 {
        return Err("Invalid payload: truncated u32".to_string());
    }
    let v = u32::from_be_bytes([
        input[*offset],
        input[*offset + 1],
        input[*offset + 2],
        input[*offset + 3],
    ]);
    *offset += 4;
    Ok(v)
}

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

fn wrap_payload(inner: &[u8]) -> Result<Vec<u8>, String> {
    let compressed = compress_bytes(inner)?;
    let (storage_kind, stored_bytes) = if compressed.len() < inner.len() {
        (STORAGE_ZLIB, compressed)
    } else {
        (STORAGE_RAW, inner.to_vec())
    };

    let original_len =
        u32::try_from(inner.len()).map_err(|_| "Payload too large (max 4 GiB-1)".to_string())?;

    let mut out = Vec::with_capacity(5 + 1 + 4 + stored_bytes.len());
    out.extend_from_slice(WRAPPED_MAGIC);
    out.push(storage_kind);
    put_u32_be(&mut out, original_len);
    out.extend_from_slice(&stored_bytes);
    Ok(out)
}

fn unwrap_payload(raw: &[u8]) -> Result<Vec<u8>, String> {
    if raw.len() < 10 {
        return Err("Invalid payload container: too short".to_string());
    }
    if &raw[0..5] != WRAPPED_MAGIC {
        return Err("Invalid payload container: bad magic".to_string());
    }

    let mut offset = 5;
    let storage_kind = raw[offset];
    offset += 1;

    let original_len = read_u32_be(raw, &mut offset)? as usize;
    let body = &raw[offset..];

    let inner = match storage_kind {
        STORAGE_RAW => body.to_vec(),
        STORAGE_ZLIB => decompress_bytes(body)?,
        _ => return Err("Invalid payload container: unknown storage kind".to_string()),
    };

    if inner.len() != original_len {
        return Err(format!(
            "Invalid payload container: expected {} bytes after decode, got {}",
            original_len,
            inner.len()
        ));
    }

    Ok(inner)
}

fn encode_text_raw(text: &str) -> Result<Vec<u8>, String> {
    let text_bytes = text.as_bytes();
    let text_len =
        u32::try_from(text_bytes.len()).map_err(|_| "Text too large (max 4 GiB-1)".to_string())?;

    let mut out = Vec::with_capacity(5 + 1 + 4 + text_bytes.len());
    out.extend_from_slice(LEGACY_MAGIC);
    out.push(KIND_TEXT);
    put_u32_be(&mut out, text_len);
    out.extend_from_slice(text_bytes);
    Ok(out)
}

fn encode_file_raw(name: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
    let name_bytes = name.as_bytes();
    let name_len = u16::try_from(name_bytes.len())
        .map_err(|_| "File name too long (max 65535)".to_string())?;
    let file_len =
        u32::try_from(bytes.len()).map_err(|_| "File too large (max 4 GiB-1)".to_string())?;

    let mut out = Vec::with_capacity(5 + 1 + 2 + name_bytes.len() + 4 + bytes.len());
    out.extend_from_slice(LEGACY_MAGIC);
    out.push(KIND_FILE);
    put_u16_be(&mut out, name_len);
    out.extend_from_slice(name_bytes);
    put_u32_be(&mut out, file_len);
    out.extend_from_slice(bytes);
    Ok(out)
}

fn decode_legacy_payload(raw: &[u8]) -> Result<Payload, String> {
    if raw.len() < 6 {
        return Err("Invalid payload: too short".to_string());
    }
    if &raw[0..5] != LEGACY_MAGIC {
        return Err("Invalid payload: bad magic".to_string());
    }

    let mut offset = 5;
    let kind = raw[offset];
    offset += 1;

    match kind {
        KIND_TEXT => {
            let text_len = read_u32_be(raw, &mut offset)? as usize;
            if raw.len() < offset + text_len {
                return Err("Invalid payload: truncated text".to_string());
            }
            let text = String::from_utf8(raw[offset..offset + text_len].to_vec())
                .map_err(|_| "Invalid payload: text is not UTF-8".to_string())?;
            Ok(Payload::Text(text))
        }
        KIND_FILE => {
            let name_len = read_u16_be(raw, &mut offset)? as usize;
            if raw.len() < offset + name_len {
                return Err("Invalid payload: truncated file name".to_string());
            }
            let name = String::from_utf8(raw[offset..offset + name_len].to_vec())
                .map_err(|_| "Invalid payload: file name is not UTF-8".to_string())?;
            offset += name_len;

            let file_len = read_u32_be(raw, &mut offset)? as usize;
            if raw.len() < offset + file_len {
                return Err("Invalid payload: truncated file bytes".to_string());
            }
            let bytes = raw[offset..offset + file_len].to_vec();

            Ok(Payload::File { name, bytes })
        }
        _ => Err("Invalid payload: unknown payload kind".to_string()),
    }
}

pub fn encode_text(text: &str) -> Result<Vec<u8>, String> {
    let raw = encode_text_raw(text)?;
    wrap_payload(&raw)
}

pub fn encode_file(name: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
    let raw = encode_file_raw(name, bytes)?;
    wrap_payload(&raw)
}

pub fn decode_payload(raw: &[u8]) -> Result<Payload, String> {
    if raw.starts_with(WRAPPED_MAGIC) {
        let inner = unwrap_payload(raw)?;
        return decode_legacy_payload(&inner);
    }

    decode_legacy_payload(raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrapped_text_round_trip() {
        let packed = encode_text("hello hello hello hello").unwrap();
        let decoded = decode_payload(&packed).unwrap();
        assert_eq!(decoded, Payload::Text("hello hello hello hello".to_string()));
    }

    #[test]
    fn legacy_payload_still_decodes() {
        let raw = encode_text_raw("legacy").unwrap();
        let decoded = decode_payload(&raw).unwrap();
        assert_eq!(decoded, Payload::Text("legacy".to_string()));
    }
}
