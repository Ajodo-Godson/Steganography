const MAGIC: &[u8; 5] = b"STEG1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Payload {
    Text(String),
    File { name: String, bytes: Vec<u8> },
}

const KIND_TEXT: u8 = 1;
const KIND_FILE: u8 = 2;

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

pub fn encode_text(text: &str) -> Result<Vec<u8>, String> {
    let text_bytes = text.as_bytes();
    let text_len =
        u32::try_from(text_bytes.len()).map_err(|_| "Text too large (max 4 GiB-1)".to_string())?;

    let mut out = Vec::with_capacity(5 + 1 + 4 + text_bytes.len());
    out.extend_from_slice(MAGIC);
    out.push(KIND_TEXT);
    put_u32_be(&mut out, text_len);
    out.extend_from_slice(text_bytes);
    Ok(out)
}

pub fn encode_file(name: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
    let name_bytes = name.as_bytes();
    let name_len = u16::try_from(name_bytes.len())
        .map_err(|_| "File name too long (max 65535)".to_string())?;
    let file_len =
        u32::try_from(bytes.len()).map_err(|_| "File too large (max 4 GiB-1)".to_string())?;

    let mut out = Vec::with_capacity(5 + 1 + 2 + name_bytes.len() + 4 + bytes.len());
    out.extend_from_slice(MAGIC);
    out.push(KIND_FILE);
    put_u16_be(&mut out, name_len);
    out.extend_from_slice(name_bytes);
    put_u32_be(&mut out, file_len);
    out.extend_from_slice(bytes);
    Ok(out)
}

pub fn decode_payload(raw: &[u8]) -> Result<Payload, String> {
    if raw.len() < 6 {
        return Err("Invalid payload: too short".to_string());
    }
    if &raw[0..5] != MAGIC {
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
