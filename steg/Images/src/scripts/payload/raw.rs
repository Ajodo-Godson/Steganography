use super::binary::{put_u16_be, put_u32_be, read_u16_be, read_u32_be};
use super::{KIND_FILE, KIND_TEXT, LEGACY_MAGIC, Payload};

pub(crate) fn encode_text_raw(text: &str) -> Result<Vec<u8>, String> {
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

pub(crate) fn encode_file_raw(name: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
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

pub(crate) fn decode_legacy_payload(raw: &[u8]) -> Result<Payload, String> {
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
        KIND_TEXT => decode_text(raw, &mut offset),
        KIND_FILE => decode_file(raw, &mut offset),
        _ => Err("Invalid payload: unknown payload kind".to_string()),
    }
}

fn decode_text(raw: &[u8], offset: &mut usize) -> Result<Payload, String> {
    let text_len = read_u32_be(raw, offset)? as usize;
    if raw.len() < *offset + text_len {
        return Err("Invalid payload: truncated text".to_string());
    }
    let text = String::from_utf8(raw[*offset..*offset + text_len].to_vec())
        .map_err(|_| "Invalid payload: text is not UTF-8".to_string())?;
    Ok(Payload::Text(text))
}

fn decode_file(raw: &[u8], offset: &mut usize) -> Result<Payload, String> {
    let name_len = read_u16_be(raw, offset)? as usize;
    if raw.len() < *offset + name_len {
        return Err("Invalid payload: truncated file name".to_string());
    }
    let name = String::from_utf8(raw[*offset..*offset + name_len].to_vec())
        .map_err(|_| "Invalid payload: file name is not UTF-8".to_string())?;
    *offset += name_len;

    let file_len = read_u32_be(raw, offset)? as usize;
    if raw.len() < *offset + file_len {
        return Err("Invalid payload: truncated file bytes".to_string());
    }
    let bytes = raw[*offset..*offset + file_len].to_vec();
    Ok(Payload::File { name, bytes })
}
