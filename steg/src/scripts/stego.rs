use ndarray::Array2;

use crate::scripts::bitstream::{bits_to_bytes, bytes_to_bits};
use crate::scripts::transform::{forward_dct_2d_block, inverse_dct_2d_block};

const HEADER_LEN_BITS: usize = 32;
const EMBED_POS: (usize, usize) = (4, 3);
const MIN_STRENGTH: f32 = 25.0;

fn encode_length(len: usize) -> Result<Vec<bool>, String> {
    let value = u32::try_from(len).map_err(|_| "Payload too large to encode".to_string())?;
    Ok((0..HEADER_LEN_BITS)
        .rev()
        .map(|shift| ((value >> shift) & 1) == 1)
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

    let mut value = 0u32;
    for &bit in bits {
        value = (value << 1) | u32::from(bit);
    }

    Ok(value as usize)
}

fn set_bit_in_block(block: &Array2<f32>, bit: bool) -> Array2<f32> {
    let mut freq = forward_dct_2d_block(block);
    let (row, col) = EMBED_POS;

    let magnitude = freq[(row, col)].abs().max(MIN_STRENGTH);
    freq[(row, col)] = if bit { magnitude } else { -magnitude };

    inverse_dct_2d_block(&freq)
}

fn get_bit_from_block(block: &Array2<f32>) -> bool {
    let freq = forward_dct_2d_block(block);
    let (row, col) = EMBED_POS;
    freq[(row, col)] >= 0.0
}

pub fn embed_bits_in_blocks(
    blocks: &[Array2<f32>],
    bits: &[bool],
) -> Result<Vec<Array2<f32>>, String> {
    if bits.len() > blocks.len() {
        return Err(format!(
            "Not enough blocks to embed bits: need {}, have {}",
            bits.len(),
            blocks.len()
        ));
    }

    let mut output = Vec::with_capacity(blocks.len());

    for (index, block) in blocks.iter().enumerate() {
        if index < bits.len() {
            output.push(set_bit_in_block(block, bits[index]));
        } else {
            output.push(block.clone());
        }
    }

    Ok(output)
}

pub fn extract_bits_from_blocks(
    blocks: &[Array2<f32>],
    bit_count: usize,
) -> Result<Vec<bool>, String> {
    if bit_count > blocks.len() {
        return Err(format!(
            "Not enough blocks to read bits: need {}, have {}",
            bit_count,
            blocks.len()
        ));
    }

    Ok(blocks
        .iter()
        .take(bit_count)
        .map(get_bit_from_block)
        .collect())
}

pub fn embed_payload_in_blocks(
    blocks: &[Array2<f32>],
    payload: &[u8],
) -> Result<Vec<Array2<f32>>, String> {
    let mut bits = encode_length(payload.len())?;
    bits.extend(bytes_to_bits(payload));

    if bits.len() > blocks.len() {
        return Err(format!(
            "Not enough blocks to embed payload: need {}, have {}",
            bits.len(),
            blocks.len()
        ));
    }

    embed_bits_in_blocks(blocks, &bits)
}

pub fn extract_payload_from_blocks(blocks: &[Array2<f32>]) -> Result<Vec<u8>, String> {
    if blocks.len() < HEADER_LEN_BITS {
        return Err(format!(
            "Not enough blocks to read header: need at least {}, have {}",
            HEADER_LEN_BITS,
            blocks.len()
        ));
    }

    let header_bits = extract_bits_from_blocks(blocks, HEADER_LEN_BITS)?;
    let payload_len = decode_length(&header_bits)?;

    let payload_bit_count = payload_len
        .checked_mul(8)
        .ok_or_else(|| "Payload size overflow".to_string())?;

    let total_bits = HEADER_LEN_BITS
        .checked_add(payload_bit_count)
        .ok_or_else(|| "Total bit count overflow".to_string())?;

    if total_bits > blocks.len() {
        return Err(format!(
            "Not enough blocks to read payload: need {}, have {}",
            total_bits,
            blocks.len()
        ));
    }

    let all_bits = extract_bits_from_blocks(blocks, total_bits)?;
    let payload_bits = &all_bits[HEADER_LEN_BITS..];

    Ok(bits_to_bytes(payload_bits))
}