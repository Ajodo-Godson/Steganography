use ndarray::Array2;

use crate::scripts::{image_ops, stego};

pub fn approx_eq_vec(left: &[f32], right: &[f32], epsilon: f32) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .all(|(a, b)| (a - b).abs() <= epsilon)
}

pub fn approx_eq_array2(left: &Array2<f32>, right: &Array2<f32>, epsilon: f32) -> bool {
    left.dim() == right.dim()
        && left
            .indexed_iter()
            .all(|((r, c), v)| (*v - right[(r, c)]).abs() <= epsilon)
}

pub fn embed_bytes_into_image(
    input: &str,
    output: &str,
    encrypted: &[u8],
    password: &str,
) -> Result<(), String> {
    let img = image_ops::load_image(input).map_err(|e| e.to_string())?;
    let gray = image_ops::extract_grayscale(&img);
    let matrix = image_ops::gray_image_to_matrix(&gray);
    let (height, width) = matrix.dim();
    let blocks = image_ops::split_into_blocks(&matrix);

    let available_bits = stego::capacity_bits(blocks.len());
    let bits_needed = 32 + encrypted.len() * 8;
    let max_payload_bytes = stego::capacity_payload_bytes(blocks.len());

    println!("Image: {}x{}", width, height);
    println!("Blocks: {}", blocks.len());
    println!("Bits/block: {}", stego::bits_per_block());
    println!(
        "Capacity: {} bits (~{} bytes payload)",
        available_bits, max_payload_bytes
    );
    println!("Required: {} bits", bits_needed);

    if bits_needed > available_bits {
        return Err(format!(
            "Payload too large for cover image: need {} bits, have {} bits",
            bits_needed, available_bits
        ));
    }

    let embedded_blocks = stego::embed_payload_in_blocks(&blocks, encrypted, password)?;
    let embedded_matrix = image_ops::merge_blocks(&embedded_blocks, height, width);
    let embedded_image = image_ops::matrix_to_gray_image(&embedded_matrix);

    embedded_image.save(output).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn extract_bytes_from_image(input: &str, password: &str) -> Result<Vec<u8>, String> {
    let stego_img = image_ops::load_image(input).map_err(|e| e.to_string())?;
    let stego_gray = image_ops::extract_grayscale(&stego_img);
    let stego_matrix = image_ops::gray_image_to_matrix(&stego_gray);
    let stego_blocks = image_ops::split_into_blocks(&stego_matrix);

    stego::extract_payload_from_blocks(&stego_blocks, password)
}
