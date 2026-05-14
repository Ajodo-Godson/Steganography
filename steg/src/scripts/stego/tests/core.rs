use super::super::positions;
use super::super::*;
use crate::scripts::crypto;
use crate::scripts::transform::{forward_dct_2d_block, inverse_dct_2d_block};
use ndarray::Array2;

fn sample_blocks(count: usize) -> Vec<Array2<f32>> {
    vec![Array2::from_elem((8, 8), 128.0); count]
}

fn sample_textured_blocks(count: usize) -> Vec<Array2<f32>> {
    vec![
        Array2::from_shape_fn((8, 8), |(r, c)| {
            if (r + c) % 2 == 0 { 32.0 } else { 224.0 }
        });
        count
    ]
}

fn sample_usable_block_indices(count: usize) -> Vec<usize> {
    (0..count).collect()
}

fn flip_embedded_bit(
    blocks: &mut [Array2<f32>],
    usable_block_indices: &[usize],
    sequence_idx: usize,
    password: &str,
) {
    let (block_idx, r, c) =
        positions::shuffled_positions(password, blocks, usable_block_indices)[sequence_idx];
    let mut freq = forward_dct_2d_block(&blocks[block_idx]);
    freq[(r, c)] = -freq[(r, c)];
    blocks[block_idx] = inverse_dct_2d_block(&freq);
}

#[test]
fn payload_round_trips_with_matching_password() {
    let blocks = sample_blocks(512);
    let usable_block_indices = sample_usable_block_indices(blocks.len());
    let payload = b"keyed randomized embedding";

    let embedded =
        embed_payload_in_blocks(&blocks, &usable_block_indices, payload, "correct horse").unwrap();
    let recovered =
        extract_payload_from_blocks(&embedded, &usable_block_indices, "correct horse").unwrap();

    assert_eq!(recovered, payload);
}

#[test]
fn extraction_with_wrong_password_fails_cleanly() {
    let blocks = sample_blocks(512);
    let usable_block_indices = sample_usable_block_indices(blocks.len());
    let encrypted = crypto::encrypt_payload(b"secret", "correct password").unwrap();

    let embedded = embed_payload_in_blocks(
        &blocks,
        &usable_block_indices,
        &encrypted,
        "correct password",
    )
    .unwrap();

    match extract_payload_from_blocks(&embedded, &usable_block_indices, "wrong password") {
        Ok(extracted) => assert!(crypto::decrypt_payload(&extracted, "wrong password").is_err()),
        Err(err) => assert!(err.contains("Invalid password") || err.contains("corrupted")),
    }
}

#[test]
fn payload_at_capacity_boundary_round_trips() {
    let blocks = sample_blocks(128);
    let usable_block_indices = sample_usable_block_indices(blocks.len());
    let payload = vec![0xa5; capacity_payload_bytes(usable_block_indices.len())];

    let embedded =
        embed_payload_in_blocks(&blocks, &usable_block_indices, &payload, "capacity").unwrap();
    let recovered =
        extract_payload_from_blocks(&embedded, &usable_block_indices, "capacity").unwrap();

    assert_eq!(recovered, payload);
}

#[test]
fn payload_over_capacity_is_rejected() {
    let blocks = sample_blocks(128);
    let usable_block_indices = sample_usable_block_indices(blocks.len());
    let payload = vec![0xa5; capacity_payload_bytes(usable_block_indices.len()) + 1];

    let err =
        embed_payload_in_blocks(&blocks, &usable_block_indices, &payload, "capacity").unwrap_err();

    assert!(err.contains("Not enough capacity"));
}

#[test]
fn corrupted_header_bit_still_extracts_payload() {
    let blocks = sample_blocks(512);
    let usable_block_indices = sample_usable_block_indices(blocks.len());
    let payload = b"header ecc survives one bad bit";

    let mut embedded =
        embed_payload_in_blocks(&blocks, &usable_block_indices, payload, "resilient").unwrap();
    flip_embedded_bit(&mut embedded, &usable_block_indices, 0, "resilient");

    let recovered =
        extract_payload_from_blocks(&embedded, &usable_block_indices, "resilient").unwrap();

    assert_eq!(recovered, payload);
}

#[test]
fn textured_blocks_increase_capacity() {
    let blocks = vec![
        sample_blocks(1).remove(0),
        sample_textured_blocks(1).remove(0),
    ];
    let usable_block_indices = sample_usable_block_indices(blocks.len());

    assert!(adaptive_capacity_bits(&blocks, &usable_block_indices) > capacity_bits(blocks.len()));
}

#[test]
fn textured_blocks_round_trip_beyond_base_capacity() {
    let blocks = sample_textured_blocks(64);
    let usable_block_indices = sample_usable_block_indices(blocks.len());
    let base_payload_bytes = capacity_payload_bytes(blocks.len());
    let adaptive_payload_bytes = adaptive_capacity_payload_bytes(&blocks, &usable_block_indices);
    let payload = vec![0x5a; adaptive_payload_bytes];

    assert!(adaptive_payload_bytes > base_payload_bytes);

    let embedded =
        embed_payload_in_blocks(&blocks, &usable_block_indices, &payload, "texture").unwrap();
    let recovered =
        extract_payload_from_blocks(&embedded, &usable_block_indices, "texture").unwrap();

    assert_eq!(recovered, payload);
}
