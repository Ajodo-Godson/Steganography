use ndarray::Array2;

use crate::scripts::transform::{forward_dct_2d_block, inverse_dct_2d_block};

const MIN_STRENGTH: f32 = 60.0;

fn force_bit(value: f32, bit: bool) -> f32 {
    let mag = value.abs().max(MIN_STRENGTH);
    if bit { mag } else { -mag }
}

fn read_bit(value: f32) -> bool {
    value >= 0.0
}

pub(crate) fn embed_assigned_bits(
    blocks: &[Array2<f32>],
    assignments: &[Vec<(usize, usize, bool)>],
) -> Vec<Array2<f32>> {
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
    out
}

pub(crate) fn read_positioned_bits(
    blocks: &[Array2<f32>],
    positions_by_block: &[Vec<(usize, usize, usize)>],
    bit_count: usize,
) -> Vec<bool> {
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
    bits
}
