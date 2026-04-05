use crate::scripts::{crypto, image_ops, stego, utils};

pub fn demo_image_and_stego(password: &str, encrypted: &[u8]) {
    std::fs::create_dir_all("output").unwrap();

    let img = image_ops::load_image("data/cat.jpg").unwrap();
    let gray = image_ops::extract_grayscale(&img);
    let matrix = image_ops::gray_image_to_matrix(&gray);
    let (height, width) = matrix.dim();

    let blocks = image_ops::split_into_blocks(&matrix);
    let rebuilt = image_ops::merge_blocks(&blocks, height, width);
    assert!(utils::approx_eq_array2(&matrix, &rebuilt, 1e-5));

    gray.save("output/cat_gray.png").unwrap();

    let embedded_blocks = stego::embed_payload_in_blocks(&blocks, encrypted, password).unwrap();
    let embedded_matrix = image_ops::merge_blocks(&embedded_blocks, height, width);
    let embedded_image = image_ops::matrix_to_gray_image(&embedded_matrix);
    embedded_image.save("output/cat_stego.png").unwrap();

    let extracted_encrypted =
        utils::extract_bytes_from_image("output/cat_stego.png", password).unwrap();
    let decrypted = crypto::decrypt_payload(&extracted_encrypted, password).unwrap();

    println!(
        "Recovered stego text: {}",
        String::from_utf8_lossy(&decrypted)
    );
}
