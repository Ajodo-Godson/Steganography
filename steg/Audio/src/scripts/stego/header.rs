use crate::scripts::error::StegoError;

const MAGIC: &[u8; 4] = b"ASTG";
const VERSION: u8 = 1;
const HEADER_LEN: usize = 9;

pub(crate) fn pack(payload: &[u8]) -> Result<Vec<u8>, StegoError> {
    let payload_len = u32::try_from(payload.len()).map_err(|_| StegoError::PayloadTooLarge {
        needed: payload.len(),
        available: u32::MAX as usize,
    })?;

    let mut packed = Vec::with_capacity(HEADER_LEN + payload.len());
    packed.extend_from_slice(MAGIC);
    packed.push(VERSION);
    packed.extend_from_slice(&payload_len.to_be_bytes());
    packed.extend_from_slice(payload);
    Ok(packed)
}

pub(crate) fn unpack(packed: &[u8]) -> Result<&[u8], StegoError> {
    if packed.len() < HEADER_LEN || &packed[..4] != MAGIC || packed[4] != VERSION {
        return Err(StegoError::NoHeaderFound);
    }

    let mut len_bytes = [0u8; 4];
    len_bytes.copy_from_slice(&packed[5..9]);
    let payload_len = u32::from_be_bytes(len_bytes) as usize;
    let end = HEADER_LEN + payload_len;

    if packed.len() < end {
        return Err(StegoError::NoHeaderFound);
    }

    Ok(&packed[HEADER_LEN..end])
}

pub(crate) fn header_len_bits() -> usize {
    HEADER_LEN * 8
}

pub(crate) fn header_len_bytes() -> usize {
    HEADER_LEN
}

pub(crate) fn payload_len_from_header(header: &[u8]) -> Result<usize, StegoError> {
    if header.len() != HEADER_LEN || &header[..4] != MAGIC || header[4] != VERSION {
        return Err(StegoError::NoHeaderFound);
    }

    let mut len_bytes = [0u8; 4];
    len_bytes.copy_from_slice(&header[5..9]);
    Ok(u32::from_be_bytes(len_bytes) as usize)
}

pub(crate) fn bytes_to_bits(bytes: &[u8]) -> impl Iterator<Item = bool> + '_ {
    bytes
        .iter()
        .flat_map(|byte| (0..8).rev().map(move |shift| ((byte >> shift) & 1) == 1))
}

pub(crate) fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
    bits.chunks(8)
        .map(|chunk| {
            chunk
                .iter()
                .fold(0u8, |byte, bit| (byte << 1) | u8::from(*bit))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_and_unpacks_payload() {
        let payload = b"hello wav";
        let packed = pack(payload).unwrap();

        assert_eq!(unpack(&packed).unwrap(), payload);
    }

    #[test]
    fn bits_round_trip_to_bytes() {
        let bytes = b"abc";
        let bits: Vec<bool> = bytes_to_bits(bytes).collect();

        assert_eq!(bits_to_bytes(&bits), bytes);
    }
}
