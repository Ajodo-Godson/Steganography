// The core logic: odd/even bit-flipping in mid-frequencies

use ndarray::Array2;

use crate::scripts::transform::{forward_dct_2d_block, inverse_dct_2d_block};

const EMBED_POS: (usize, usize) = (3, 4);
const MIN_MAGNITUDE: i32 = 2;

fn adjust_to_min_magnitude(coeff: i32) -> i32 {
    if coeff == 0 {
        return MIN_MAGNITUDE;
    }

    if coeff.abs() >= MIN_MAGNITUDE {
        coeff
    } else if coeff.is_negative() {
        -MIN_MAGNITUDE
    } else {
        MIN_MAGNITUDE
    }
}

fn matches_bit(coeff: i32, bit: bool) -> bool {
    let is_even = coeff.rem_euclid(2) == 0;
    if bit { is_even } else { !is_even }
}

fn force_parity(value: f32, bit: bool) -> f32 {
    let base = adjust_to_min_magnitude(value.round() as i32);

    if matches_bit(base, bit) {
        return base as f32;
    }

    let candidates = [base - 1, base + 1];
    let mut best = candidates[0];
    let mut best_score = i32::MAX;

    for candidate in candidates {
        let candidate = adjust_to_min_magnitude(candidate);

        if matches_bit(candidate, bit) {
            let score = (candidate - base).abs();
            if score < best_score {
                best = candidate;
                best_score = score;
            }
        }
    }

    best as f32
}

fn bit_from_coeff(value: f32) -> bool {
    let coeff = value.round() as i32;
    coeff.rem_euclid(2) == 0
}

pub fn embed_bits_in_blocks(
    blocks: &[Array2<f32>],
    bits: &[bool],
) -> Result<Vec<Array2<f32>>, String> {
    if bits.len() > blocks.len() {
        return Err(format!(
            "Not enough blocks: need {}, have {}",
            bits.len(),
            blocks.len()
        ));
    }

    let mut output = Vec::with_capacity(blocks.len());

    for (index, block) in blocks.iter().enumerate() {
        if index < bits.len() {
            let bit = bits[index];
            let mut freq_block = forward_dct_2d_block(block);
            let (row, col) = EMBED_POS;

            let original = freq_block[(row, col)];
            freq_block[(row, col)] = force_parity(original, bit);

            let spatial_block = inverse_dct_2d_block(&freq_block);
            output.push(spatial_block);
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
            "Not enough blocks: need {}, have {}",
            bit_count,
            blocks.len()
        ));
    }

    let mut bits = Vec::with_capacity(bit_count);

    for block in blocks.iter().take(bit_count) {
        let freq_block = forward_dct_2d_block(block);
        let (row, col) = EMBED_POS;
        bits.push(bit_from_coeff(freq_block[(row, col)]));
    }

    Ok(bits)
}