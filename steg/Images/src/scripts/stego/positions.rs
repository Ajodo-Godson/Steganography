use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use sha2::{Digest, Sha256};

use ndarray::Array2;

const BASE_COORDS: &[(usize, usize)] = &[(1, 2)];
const TEXTURED_COORDS: &[(usize, usize)] = &[(2, 1), (1, 3), (2, 2)];
const TEXTURE_VARIANCE_THRESHOLD: f32 = 400.0;

pub(crate) fn bits_per_block() -> usize {
    BASE_COORDS.len()
}

pub(crate) fn max_bits_per_block() -> usize {
    BASE_COORDS.len() + TEXTURED_COORDS.len()
}

fn embedding_seed(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"steg-position-seed:v1");
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}

pub(crate) fn shuffled_positions(
    password: &str,
    blocks: &[Array2<f32>],
    usable_block_indices: &[usize],
) -> Vec<(usize, usize, usize)> {
    let mut positions = Vec::with_capacity(usable_block_indices.len() * max_bits_per_block());

    for &block_idx in usable_block_indices {
        positions.extend(block_positions(blocks, block_idx));
    }

    let mut rng = StdRng::from_seed(embedding_seed(password));
    positions.shuffle(&mut rng);
    positions
}

pub(crate) fn capacity_bits(blocks: &[Array2<f32>], usable_block_indices: &[usize]) -> usize {
    usable_block_indices
        .iter()
        .map(|&block_idx| block_positions(blocks, block_idx).len())
        .sum()
}

fn block_positions(blocks: &[Array2<f32>], block_idx: usize) -> Vec<(usize, usize, usize)> {
    let mut positions = coords_for_block(&blocks[block_idx])
        .iter()
        .map(|&(r, c)| (block_idx, r, c))
        .collect::<Vec<_>>();
    positions.shrink_to_fit();
    positions
}

fn coords_for_block(block: &Array2<f32>) -> Vec<(usize, usize)> {
    let mut coords = BASE_COORDS.to_vec();
    if is_textured(block) {
        coords.extend_from_slice(TEXTURED_COORDS);
    }
    coords
}

fn is_textured(block: &Array2<f32>) -> bool {
    let len = block.len() as f32;
    let mean = block.iter().sum::<f32>() / len;
    let variance = block
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f32>()
        / len;

    variance >= TEXTURE_VARIANCE_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shuffled_positions_are_keyed_and_stable() {
        let usable_block_indices: Vec<usize> = (0..128).collect();
        let blocks = vec![Array2::from_elem((8, 8), 128.0); usable_block_indices.len()];

        let left = shuffled_positions("alpha", &blocks, &usable_block_indices);
        let right = shuffled_positions("alpha", &blocks, &usable_block_indices);
        let different = shuffled_positions("beta", &blocks, &usable_block_indices);

        assert_eq!(left, right);
        assert_ne!(left, different);
    }

    #[test]
    fn textured_blocks_get_extra_coefficients() {
        let flat = Array2::from_elem((8, 8), 128.0);
        let textured =
            Array2::from_shape_fn((8, 8), |(r, c)| if (r + c) % 2 == 0 { 32.0 } else { 224.0 });

        assert_eq!(coords_for_block(&flat).len(), BASE_COORDS.len());
        assert_eq!(coords_for_block(&textured).len(), max_bits_per_block());
    }
}
