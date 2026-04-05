pub fn bytes_to_bits(bytes: &[u8]) -> Vec<bool> {
    let mut bits = Vec::with_capacity(bytes.len() * 8);

    for &byte in bytes {
        for shift in (0..8).rev() {
            bits.push(((byte >> shift) & 1) == 1);
        }
    }

    bits
}

pub fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(bits.len().div_ceil(8));

    for chunk in bits.chunks(8) {
        let mut byte = 0u8;

        for (i, bit) in chunk.iter().enumerate() {
            if *bit {
                byte |= 1 << (7 - i);
            }
        }

        bytes.push(byte);
    }

    bytes
}
