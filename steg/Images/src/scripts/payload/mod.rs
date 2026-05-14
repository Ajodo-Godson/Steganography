mod binary;
mod container;
mod raw;

use container::{unwrap_payload, wrap_payload};
use raw::{decode_legacy_payload, encode_file_raw, encode_text_raw};

pub(crate) const LEGACY_MAGIC: &[u8; 5] = b"STEG1";
pub(crate) const WRAPPED_MAGIC: &[u8; 5] = b"STEG2";
pub(crate) const RICH_MAGIC: &[u8; 5] = b"STEG3";

pub(crate) const KIND_TEXT: u8 = 1;
pub(crate) const KIND_FILE: u8 = 2;

pub(crate) const STORAGE_RAW: u8 = 0;
pub(crate) const STORAGE_ZLIB: u8 = 1;

pub(crate) const PAYLOAD_VERSION: u8 = 3;
pub(crate) const ALGORITHM_DCT_LUMA: u16 = 1;
pub(crate) const FIRST_CHUNK_INDEX: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Text(String),
    File { name: String, bytes: Vec<u8> },
}

pub fn encode_text(text: &str) -> Result<Vec<u8>, String> {
    let raw = encode_text_raw(text)?;
    wrap_payload(&raw, "text/plain; charset=utf-8")
}

pub fn encode_file(name: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
    let raw = encode_file_raw(name, bytes)?;
    wrap_payload(&raw, mime_for_name(name))
}

pub fn decode_payload(raw: &[u8]) -> Result<Payload, String> {
    if raw.starts_with(WRAPPED_MAGIC) || raw.starts_with(RICH_MAGIC) {
        let inner = unwrap_payload(raw)?;
        return decode_legacy_payload(&inner);
    }

    decode_legacy_payload(raw)
}

fn mime_for_name(name: &str) -> &'static str {
    match name
        .rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
    {
        Some(ext) if ext == "txt" => "text/plain",
        Some(ext) if ext == "pdf" => "application/pdf",
        Some(ext) if ext == "png" => "image/png",
        Some(ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg",
        Some(ext) if ext == "wav" => "audio/wav",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripts::payload::binary::put_u32_be;

    #[test]
    fn wrapped_text_round_trip() {
        let packed = encode_text("hello hello hello hello").unwrap();
        assert!(packed.starts_with(RICH_MAGIC));
        let decoded = decode_payload(&packed).unwrap();
        assert_eq!(
            decoded,
            Payload::Text("hello hello hello hello".to_string())
        );
    }

    #[test]
    fn legacy_payload_still_decodes() {
        let raw = encode_text_raw("legacy").unwrap();
        let decoded = decode_payload(&raw).unwrap();
        assert_eq!(decoded, Payload::Text("legacy".to_string()));
    }

    #[test]
    fn steg2_wrapped_payload_still_decodes() {
        let inner = encode_text_raw("wrapped legacy").unwrap();
        let mut packed = Vec::new();
        packed.extend_from_slice(WRAPPED_MAGIC);
        packed.push(STORAGE_RAW);
        put_u32_be(&mut packed, inner.len() as u32);
        packed.extend_from_slice(&inner);

        let decoded = decode_payload(&packed).unwrap();
        assert_eq!(decoded, Payload::Text("wrapped legacy".to_string()));
    }
}
