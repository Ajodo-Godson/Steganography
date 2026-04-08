
use ndarray::Array2;
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use sha2::{Digest, Sha256};

use crate::scripts::bitstream::{bits_to_bytes, bytes_to_bits};
use crate::scripts::transform::{forward_dct_2d_block, inverse_dct_2d_block};

const HEADER_LEN_BITS: usize = 32;
const MIN_STRENGTH: f32 = 60.0;

// Multi-coefficient embedding coordinates (mid-frequency zone)
const SAFE_COORDS: &[(usize, usize)] = &[(1, 2)];

fn force_bit(value: f32, bit: bool) -> f32 {
    let mag = value.abs().max(MIN_STRENGTH);
    if bit { mag } else { -mag }
}

fn read_bit(value: f32) -> bool {
    value >= 0.0
}

fn embedding_seed(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"steg-position-seed:v1");
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}

fn shuffled_positions(
    password: &str,
    usable_block_indices: &[usize],
) -> Vec<(usize, usize, usize)> {
    let mut positions = Vec::with_capacity(capacity_bits(usable_block_indices.len()));

    for &block_idx in usable_block_indices {
        for &(r, c) in SAFE_COORDS {
            positions.push((block_idx, r, c));
        }
    }

    let mut rng = StdRng::from_seed(embedding_seed(password));
    positions.shuffle(&mut rng);
    positions
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
            HEADER_LEN_BITS,
            bits.len()
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
    usable_block_indices: &[usize],
    bits: &[bool],
    password: &str,
) -> Result<Vec<Array2<f32>>, String> {
    let cap = capacity_bits(usable_block_indices.len());
    if bits.len() > cap {
        return Err(format!(
            "Not enough capacity: need {} bits, have {} bits",
            bits.len(),
            cap
        ));
    }

    let mut assignments = vec![Vec::new(); blocks.len()];
    for (bit, (block_idx, r, c)) in bits
        .iter()
        .copied()
        .zip(shuffled_positions(password, usable_block_indices).into_iter())
    {
        assignments[block_idx].push((r, c, bit));
    }

    let mut out = Vec::with_capacity(blocks.len());
    for (block, block_assignments) in blocks.iter().zip(assignments.iter()) {
        if block_assignments.is_empty() {
            out.push(block.clone());
            continue;
        }

        let mut freq = forward_dct_2d_block(block);
        for &(r, c, bit) in block_assignments {
            freq[(r, c)] = force_bit(freq[(r, c)], bit);
        }

        out.push(inverse_dct_2d_block(&freq));
    }

    Ok(out)
}

pub fn extract_bits_from_blocks(
    blocks: &[Array2<f32>],
    usable_block_indices: &[usize],
    bit_count: usize,
    password: &str,
) -> Result<Vec<bool>, String> {
    let cap = capacity_bits(usable_block_indices.len());
    if bit_count > cap {
        return Err(format!(
            "Not enough capacity: need {} bits, have {} bits",
            bit_count,
            cap
        ));
    }

    let mut positions_by_block = vec![Vec::new(); blocks.len()];
    for (sequence_idx, (block_idx, r, c)) in shuffled_positions(password, usable_block_indices)
        .into_iter()
        .take(bit_count)
        .enumerate()
    {
        positions_by_block[block_idx].push((sequence_idx, r, c));
    }

    let mut bits = vec![false; bit_count];
    for (block, block_positions) in blocks.iter().zip(positions_by_block.iter()) {
        if block_positions.is_empty() {
            continue;
        }

        let freq = forward_dct_2d_block(block);
        for &(sequence_idx, r, c) in block_positions {
            bits[sequence_idx] = read_bit(freq[(r, c)]);
        }
    }

    Ok(bits)
}
pub fn embed_payload_in_blocks(
    blocks: &[Array2<f32>],
    usable_block_indices: &[usize],
    payload: &[u8],
    password: &str,
) -> Result<Vec<Array2<f32>>, String> {
    let mut bits = encode_length(payload.len())?;
    bits.extend(bytes_to_bits(payload));
    embed_bits_in_blocks(blocks, usable_block_indices, &bits, password)
}

pub fn extract_payload_from_blocks(
    blocks: &[Array2<f32>],
    usable_block_indices: &[usize],
    password: &str,
) -> Result<Vec<u8>, String> {
    let header_bits =
        extract_bits_from_blocks(blocks, usable_block_indices, HEADER_LEN_BITS, password)?;
    let payload_len = decode_length(&header_bits)?;

    let payload_bits = payload_len
        .checked_mul(8)
        .ok_or_else(|| "Payload bit length overflow".to_string())?;

    let total_bits = HEADER_LEN_BITS
        .checked_add(payload_bits)
        .ok_or_else(|| "Total bit length overflow".to_string())?;

    if total_bits > capacity_bits(usable_block_indices.len()) {
        return Err("Invalid password or corrupted stego image".to_string());
    }

    let all_bits =
        extract_bits_from_blocks(blocks, usable_block_indices, total_bits, password)?;
    Ok(bits_to_bytes(&all_bits[HEADER_LEN_BITS..]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::Array2;

    fn sample_blocks(count: usize) -> Vec<Array2<f32>> {
        vec![Array2::from_elem((8, 8), 128.0); count]
    }

    #[test]
    fn shuffled_positions_are_keyed_and_stable() {
        let left = shuffled_positions("alpha", 128);
        let right = shuffled_positions("alpha", 128);
        let different = shuffled_positions("beta", 128);

        assert_eq!(left, right);
        assert_ne!(left, different);
    }

    #[test]
    fn payload_round_trips_with_matching_password() {
        let blocks = sample_blocks(256);
        let payload = b"keyed randomized embedding";

        let embedded = embed_payload_in_blocks(&blocks, payload, "correct horse").unwrap();
        let recovered = extract_payload_from_blocks(&embedded, "correct horse").unwrap();

        assert_eq!(recovered, payload);
    }
}
