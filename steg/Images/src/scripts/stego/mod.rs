use ndarray::Array2;

use crate::scripts::bitstream::{bits_to_bytes, bytes_to_bits};

mod block_io;
mod ecc;
mod positions;

pub fn bits_per_block() -> usize {
    positions::bits_per_block()
}

pub fn capacity_bits(block_count: usize) -> usize {
    block_count * bits_per_block()
}

pub fn capacity_payload_bytes(block_count: usize) -> usize {
    capacity_bits(block_count).saturating_sub(ecc::HEADER_LEN_BITS) / 8
}

pub fn adaptive_capacity_bits(blocks: &[Array2<f32>], usable_block_indices: &[usize]) -> usize {
    positions::capacity_bits(blocks, usable_block_indices)
}

pub fn adaptive_capacity_payload_bytes(
    blocks: &[Array2<f32>],
    usable_block_indices: &[usize],
) -> usize {
    adaptive_capacity_bits(blocks, usable_block_indices).saturating_sub(ecc::HEADER_LEN_BITS) / 8
}

pub fn payload_bits_required(payload_len: usize) -> Option<usize> {
    ecc::payload_bits_required(payload_len)
}

pub fn read_file(path: impl AsRef<std::path::Path>) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|err| format!("Failed to read file: {err}"))
}

pub fn embed_bits_in_blocks(
    blocks: &[Array2<f32>],
    usable_block_indices: &[usize],
    bits: &[bool],
    password: &str,
) -> Result<Vec<Array2<f32>>, String> {
    let cap = adaptive_capacity_bits(blocks, usable_block_indices);
    if bits.len() > cap {
        return Err(format!(
            "Not enough capacity: need {} bits, have {} bits",
            bits.len(),
            cap
        ));
    }

    let mut assignments = vec![Vec::new(); blocks.len()];
    for (bit, (block_idx, r, c)) in bits.iter().copied().zip(positions::shuffled_positions(
        password,
        blocks,
        usable_block_indices,
    )) {
        assignments[block_idx].push((r, c, bit));
    }

    Ok(block_io::embed_assigned_bits(blocks, &assignments))
}

pub fn extract_bits_from_blocks(
    blocks: &[Array2<f32>],
    usable_block_indices: &[usize],
    bit_count: usize,
    password: &str,
) -> Result<Vec<bool>, String> {
    let cap = adaptive_capacity_bits(blocks, usable_block_indices);
    if bit_count > cap {
        return Err(format!(
            "Not enough capacity: need {} bits, have {} bits",
            bit_count, cap
        ));
    }

    let mut positions_by_block = vec![Vec::new(); blocks.len()];
    for (sequence_idx, (block_idx, r, c)) in
        positions::shuffled_positions(password, blocks, usable_block_indices)
            .into_iter()
            .take(bit_count)
            .enumerate()
    {
        positions_by_block[block_idx].push((sequence_idx, r, c));
    }

    Ok(block_io::read_positioned_bits(
        blocks,
        &positions_by_block,
        bit_count,
    ))
}

pub fn embed_payload_in_blocks(
    blocks: &[Array2<f32>],
    usable_block_indices: &[usize],
    payload: &[u8],
    password: &str,
) -> Result<Vec<Array2<f32>>, String> {
    let total_bits = payload_bits_required(payload.len())
        .ok_or_else(|| "Total bit length overflow".to_string())?;
    if total_bits > adaptive_capacity_bits(blocks, usable_block_indices) {
        return Err(format!(
            "Not enough capacity: need {} bits, have {} bits",
            total_bits,
            adaptive_capacity_bits(blocks, usable_block_indices)
        ));
    }

    let mut bits = ecc::encode_length(payload.len())?;
    bits.extend(bytes_to_bits(payload));
    embed_bits_in_blocks(blocks, usable_block_indices, &bits, password)
}

pub fn extract_payload_from_blocks(
    blocks: &[Array2<f32>],
    usable_block_indices: &[usize],
    password: &str,
) -> Result<Vec<u8>, String> {
    let header_bits =
        extract_bits_from_blocks(blocks, usable_block_indices, ecc::HEADER_LEN_BITS, password)?;
    let payload_len = ecc::decode_length(&header_bits)?;
    let payload_bits = payload_len
        .checked_mul(8)
        .ok_or_else(|| "Payload bit length overflow".to_string())?;
    let total_bits = ecc::HEADER_LEN_BITS
        .checked_add(payload_bits)
        .ok_or_else(|| "Total bit length overflow".to_string())?;

    if total_bits > adaptive_capacity_bits(blocks, usable_block_indices) {
        return Err("Invalid password or corrupted stego image".to_string());
    }

    let all_bits = extract_bits_from_blocks(blocks, usable_block_indices, total_bits, password)?;
    Ok(bits_to_bytes(&all_bits[ecc::HEADER_LEN_BITS..]))
}

#[cfg(test)]
mod tests;
