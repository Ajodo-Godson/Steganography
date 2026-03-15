use ndarray::Array2;

use crate::scripts::bitstream::{bits_to_bytes, bytes_to_bits};
use crate::scripts::transform::{forward_dct_2d_block, inverse_dct_2d_block};

const HEADER_LEN_BITS: usize = 32;
const MIN_STRENGTH: f32 = 18.0;

// Multi-coefficient embedding coordinates (mid-frequency zone)
const SAFE_COORDS: &[(usize, usize)] = &[
    (1, 2), (2, 1), (2, 2), (1, 3), (3, 1), (2, 3), (3, 2), (1, 4), (4, 1),
];

fn force_bit(value: f32, bit: bool) -> f32 {
    let mag = value.abs().max(MIN_STRENGTH);
    if bit { mag } else { -mag }
}

fn read_bit(value: f32) -> bool {
    value >= 0.0
}

pub fn bits_per_block() -> usize {
    SAFE_COORDS.len()
}

pub fn capacity_bits(block_count: usize) -> usize {
    block_count * bits_per_block()
}

pub fn capacity_payload_bytes(block_count: usize) -> usize {
    capacity_bits(block_count).saturating_sub(HEADER_LEN_BITS) / 8
}

fn encode_length(len: usize) -> Result<Vec<bool>, String> {
    let val = u32::try_from(len).map_err(|_| "Payload too large".to_string())?;
    Ok((0..HEADER_LEN_BITS)
        .rev()
        .map(|shift| ((val >> shift) & 1) == 1)
        .collect())
}

fn decode_length(bits: &[bool]) -> Result<usize, String> {
    if bits.len() != HEADER_LEN_BITS {
        return Err(format!(
            "Invalid header length: expected {}, got {}",
            HEADER_LEN_BITS, bits.len()
        ));
    }

    let mut v = 0u32;
    for &bit in bits {
        v = (v << 1) | u32::from(bit);
    }
    Ok(v as usize)
}

pub fn embed_bits_in_blocks(
    blocks: &[Array2<f32>],
    bits: &[bool],
) -> Result<Vec<Array2<f32>>, String> {
    let cap = capacity_bits(blocks.len());
    if bits.len() > cap {
        return Err(format!(
            "Not enough capacity: need {} bits, have {} bits",
            bits.len(),
            cap
        ));
    }

    let mut out = Vec::with_capacity(blocks.len());
    let mut bit_idx = 0usize;

    for block in blocks {
        if bit_idx >= bits.len() {
            out.push(block.clone());
            continue;
        }

        let mut freq = forward_dct_2d_block(block);

        for &(r, c) in SAFE_COORDS {
            if bit_idx >= bits.len() {
                break;
            }
            freq[(r, c)] = force_bit(freq[(r, c)], bits[bit_idx]);
            bit_idx += 1;
        }

        out.push(inverse_dct_2d_block(&freq));
    }

    Ok(out)
}

pub fn extract_bits_from_blocks(
    blocks: &[Array2<f32>],
    bit_count: usize,
) -> Result<Vec<bool>, String> {
    let cap = capacity_bits(blocks.len());
    if bit_count > cap {
        return Err(format!(
            "Not enough capacity: need {} bits, have {} bits",
            bit_count,
            cap
        ));
    }

    let mut bits = Vec::with_capacity(bit_count);

    'outer: for block in blocks {
        let freq = forward_dct_2d_block(block);
        for &(r, c) in SAFE_COORDS {
            if bits.len() >= bit_count {
                break 'outer;
            }
            bits.push(read_bit(freq[(r, c)]));
        }
    }

    Ok(bits)
}

pub fn embed_payload_in_blocks(
    blocks: &[Array2<f32>],
    payload: &[u8],
) -> Result<Vec<Array2<f32>>, String> {
    let mut bits = encode_length(payload.len())?;
    bits.extend(bytes_to_bits(payload));
    embed_bits_in_blocks(blocks, &bits)
}

pub fn extract_payload_from_blocks(blocks: &[Array2<f32>]) -> Result<Vec<u8>, String> {
    // Read 32-bit payload length header first
    let header_bits = extract_bits_from_blocks(blocks, HEADER_LEN_BITS)?;
    let payload_len = decode_length(&header_bits)?;

    let payload_bits = payload_len
        .checked_mul(8)
        .ok_or_else(|| "Payload bit length overflow".to_string())?;

    let total_bits = HEADER_LEN_BITS
        .checked_add(payload_bits)
        .ok_or_else(|| "Total bit length overflow".to_string())?;

    let all_bits = extract_bits_from_blocks(blocks, total_bits)?;
    Ok(bits_to_bytes(&all_bits[HEADER_LEN_BITS..]))
}