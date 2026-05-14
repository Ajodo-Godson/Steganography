pub(crate) const LENGTH_LEN_BITS: usize = 32;
pub(crate) const HEADER_LEN_BITS: usize = (LENGTH_LEN_BITS / 4) * 7;

pub(crate) fn payload_bits_required(payload_len: usize) -> Option<usize> {
    payload_len
        .checked_mul(8)
        .and_then(|payload_bits| HEADER_LEN_BITS.checked_add(payload_bits))
}

pub(crate) fn encode_length(len: usize) -> Result<Vec<bool>, String> {
    let val = u32::try_from(len).map_err(|_| "Payload too large".to_string())?;
    let length_bits: Vec<bool> = (0..LENGTH_LEN_BITS)
        .rev()
        .map(|shift| ((val >> shift) & 1) == 1)
        .collect();

    let mut encoded = Vec::with_capacity(HEADER_LEN_BITS);
    for chunk in length_bits.chunks_exact(4) {
        encoded.extend(encode_hamming_7_4(chunk));
    }
    Ok(encoded)
}

pub(crate) fn decode_length(bits: &[bool]) -> Result<usize, String> {
    if bits.len() != HEADER_LEN_BITS {
        return Err(format!(
            "Invalid header length: expected {}, got {}",
            HEADER_LEN_BITS,
            bits.len()
        ));
    }

    let mut length_bits = Vec::with_capacity(LENGTH_LEN_BITS);
    for chunk in bits.chunks_exact(7) {
        length_bits.extend(decode_hamming_7_4(chunk)?);
    }

    let mut v = 0u32;
    for &bit in &length_bits {
        v = (v << 1) | u32::from(bit);
    }
    Ok(v as usize)
}

fn encode_hamming_7_4(nibble: &[bool]) -> [bool; 7] {
    debug_assert_eq!(nibble.len(), 4);
    let (d1, d2, d3, d4) = (nibble[0], nibble[1], nibble[2], nibble[3]);
    [d1 ^ d2 ^ d4, d1 ^ d3 ^ d4, d1, d2 ^ d3 ^ d4, d2, d3, d4]
}

fn decode_hamming_7_4(codeword: &[bool]) -> Result<[bool; 4], String> {
    if codeword.len() != 7 {
        return Err(format!(
            "Invalid Hamming codeword length: expected 7, got {}",
            codeword.len()
        ));
    }

    let mut corrected = [false; 7];
    corrected.copy_from_slice(codeword);
    let s1 = corrected[0] ^ corrected[2] ^ corrected[4] ^ corrected[6];
    let s2 = corrected[1] ^ corrected[2] ^ corrected[5] ^ corrected[6];
    let s4 = corrected[3] ^ corrected[4] ^ corrected[5] ^ corrected[6];
    let syndrome = usize::from(s1) | (usize::from(s2) << 1) | (usize::from(s4) << 2);

    if syndrome != 0 {
        corrected[syndrome - 1] = !corrected[syndrome - 1];
    }
    Ok([corrected[2], corrected[4], corrected[5], corrected[6]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hamming_header_corrects_single_bit_per_codeword() {
        let length = 0x1234_abcdusize;
        let encoded = encode_length(length).unwrap();

        for codeword_start in (0..HEADER_LEN_BITS).step_by(7) {
            let mut damaged = encoded.clone();
            damaged[codeword_start] = !damaged[codeword_start];
            assert_eq!(decode_length(&damaged).unwrap(), length);
        }
    }
}
