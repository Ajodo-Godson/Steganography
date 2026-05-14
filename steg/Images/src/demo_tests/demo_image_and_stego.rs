use crate::scripts::{crypto, image_ops, stego, utils};

pub fn demo_image_and_stego(password: &str, encrypted: &[u8]) {
    std::fs::create_dir_all("output").unwrap();

    let img = image_ops::load_image("data/cat.jpg").unwrap();
    let (luma, cb, cr) = image_ops::extract_luma_and_chroma(&img);
    let (height, width) = luma.dim();

    let blocks = image_ops::split_into_blocks(&luma);
    let usable_block_indices = image_ops::embeddable_block_indices(height, width);
    let rebuilt = image_ops::merge_blocks(&blocks, height, width);
    assert!(utils::approx_eq_array2(&luma, &rebuilt, 1e-5));

    image_ops::matrix_to_gray_image(&luma)
        .save("output/cat_luma.png")
        .unwrap();

    let embedded_blocks =
        stego::embed_payload_in_blocks(&blocks, &usable_block_indices, encrypted, password)
            .unwrap();
    let embedded_luma = image_ops::merge_blocks(&embedded_blocks, height, width);
    let embedded_image = image_ops::luma_and_chroma_to_rgb_image(&embedded_luma, &cb, &cr);
    embedded_image.save("output/cat_stego.png").unwrap();

    let extracted_encrypted =
        utils::extract_bytes_from_image("output/cat_stego.png", password).unwrap();
    let decrypted = crypto::decrypt_payload(&extracted_encrypted, password).unwrap();

    println!(
        "Recovered stego text: {}",
        String::from_utf8_lossy(&decrypted)
    );
}
