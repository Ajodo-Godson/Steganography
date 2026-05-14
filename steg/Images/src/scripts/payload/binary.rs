pub(crate) fn put_u16_be(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_be_bytes());
}

pub(crate) fn put_u32_be(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_be_bytes());
}

pub(crate) fn read_u16_be(input: &[u8], offset: &mut usize) -> Result<u16, String> {
    if input.len() < *offset + 2 {
        return Err("Invalid payload: truncated u16".to_string());
    }
    let v = u16::from_be_bytes([input[*offset], input[*offset + 1]]);
    *offset += 2;
    Ok(v)
}

pub(crate) fn read_u32_be(input: &[u8], offset: &mut usize) -> Result<u32, String> {
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
