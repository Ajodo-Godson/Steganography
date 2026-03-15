mod scripts;

use clap::Parser;
use ndarray::Array2;
use scripts::cli::Cli;
use scripts::{bitstream, crypto, image_ops, stego, transform};

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

fn approx_eq_vec(left: &[f32], right: &[f32], epsilon: f32) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .all(|(a, b)| (a - b).abs() <= epsilon)
}

fn approx_eq_array2(left: &Array2<f32>, right: &Array2<f32>, epsilon: f32) -> bool {
    left.dim() == right.dim()
        && left
            .indexed_iter()
            .all(|((row, col), value)| (*value - right[(row, col)]).abs() <= epsilon)
}

fn demo_crypto(password: &str, plaintext: &[u8]) -> Vec<u8> {
    let encrypted = crypto::encrypt_payload(plaintext, password).unwrap();

    let salt = &encrypted[0..SALT_LEN];
    let nonce = &encrypted[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &encrypted[SALT_LEN + NONCE_LEN..];

    println!("Salt bytes: {:?}", salt);
    println!("Nonce bytes: {:?}", nonce);
    println!("Ciphertext bytes: {:?}", ciphertext);

    println!("Salt bits: {:?}", bitstream::bytes_to_bits(salt));
    println!("Nonce bits: {:?}", bitstream::bytes_to_bits(nonce));
    println!("Ciphertext bits: {:?}", bitstream::bytes_to_bits(ciphertext));

    let decrypted = crypto::decrypt_payload(&encrypted, password).unwrap();
    println!("Decrypted text: {}", String::from_utf8_lossy(&decrypted));

    assert_eq!(plaintext, decrypted.as_slice());
    println!("Encryption and decryption successful!");
    println!("-----------------------------------");

    encrypted
}

fn demo_1d_dct() {
    let signal = vec![1.0, 2.0, 3.0, 4.0];
    let dct = transform::forward_dct(&signal);
    let restored = transform::inverse_dct(&dct);

    println!("Original signal: {:?}", signal);
    println!("DCT result: {:?}", dct);
    println!("IDCT result: {:?}", restored);

    assert!(approx_eq_vec(&signal, &restored, 1e-5));
    println!("-----------------------------------");
}

fn demo_2d_dct() {
    let block = Array2::from_shape_vec(
        (8, 8),
        vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 3.0,
            4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 5.0,
            6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0,
            7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0,
            15.0,
        ],
    )
    .unwrap();

    let dct_block = transform::forward_dct_2d_block(&block);
    let restored_block = transform::inverse_dct_2d_block(&dct_block);

    println!("Original block:\n{:?}", block);
    println!("DCT block:\n{:?}", dct_block);
    println!("Restored block:\n{:?}", restored_block);

    assert!(approx_eq_array2(&block, &restored_block, 1e-3));
    println!("-----------------------------------");
}

fn demo_image_and_stego(password: &str, encrypted: &[u8]) {
    std::fs::create_dir_all("output").unwrap();

    let img = image_ops::load_image("data/cat.jpg").unwrap();
    let gray = image_ops::extract_grayscale(&img);
    let matrix = image_ops::gray_image_to_matrix(&gray);
    let (height, width) = matrix.dim();

    println!("Image size: {}x{}", width, height);

    let blocks = image_ops::split_into_blocks(&matrix);
    println!("Number of blocks: {}", blocks.len());
    println!("Block size: {}x{}", image_ops::BLOCK_SIZE, image_ops::BLOCK_SIZE);

    let rebuilt = image_ops::merge_blocks(&blocks, height, width);
    assert!(approx_eq_array2(&matrix, &rebuilt, 1e-5));

    gray.save("output/cat_gray.png").unwrap();
    println!("Saved grayscale image to output/cat_gray.png");

    let embedded_blocks = stego::embed_payload_in_blocks(&blocks, encrypted).unwrap();

    let embedded_matrix = image_ops::merge_blocks(&embedded_blocks, height, width);
    let embedded_image = image_ops::matrix_to_gray_image(&embedded_matrix);


    embedded_image.save("output/cat_stego.png").unwrap();
    println!("Saved stego image to output/cat_stego.png");

   
    let stego_img = image_ops::load_image("output/cat_stego.png").unwrap();
    let stego_gray = image_ops::extract_grayscale(&stego_img);
    let stego_matrix = image_ops::gray_image_to_matrix(&stego_gray);
    let stego_blocks = image_ops::split_into_blocks(&stego_matrix);

    let extracted_encrypted = stego::extract_payload_from_blocks(&stego_blocks).unwrap();
    let decrypted = crypto::decrypt_payload(&extracted_encrypted, password).unwrap();

    println!("Recovered stego text: {}", String::from_utf8_lossy(&decrypted));
    assert_eq!(decrypted, b"Hello, world!");
    println!("Stego round-trip successful");
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        scripts::cli::Commands::Demo => {
            let password = "super_secret_password";
            let plaintext = b"Hello, world!";

            let encrypted = demo_crypto(password, plaintext);
            demo_1d_dct();
            demo_2d_dct();
            demo_image_and_stego(password, &encrypted);
        }
        scripts::cli::Commands::Embed {
            input,
            output,
            password,
            message,
        } => {
            std::fs::create_dir_all("output").ok();

            let img = image_ops::load_image(&input).expect("Failed to load input image");
            let gray = image_ops::extract_grayscale(&img);
            let matrix = image_ops::gray_image_to_matrix(&gray);
            let (height, width) = matrix.dim();
            let blocks = image_ops::split_into_blocks(&matrix);

            let encrypted =
                crypto::encrypt_payload(message.as_bytes(), &password).expect("Encryption failed");

            let bits_needed = 32 + encrypted.len() * 8;
            if bits_needed > blocks.len() {
                panic!(
                    "Payload too large for image capacity: need {} bits, have {}",
                    bits_needed,
                    blocks.len()
                );
            }

            let embedded_blocks =
                stego::embed_payload_in_blocks(&blocks, &encrypted).expect("Embedding failed");

            
            let embedded_matrix = image_ops::merge_blocks(&embedded_blocks, height, width);
            let embedded_image = image_ops::matrix_to_gray_image(&embedded_matrix);

            
            embedded_image
                .save(&output)
                .expect("Failed to save output image");

            println!("Embed successful: {}", output);
        }
        scripts::cli::Commands::Extract { input, password } => {
            let stego_img = image_ops::load_image(&input).expect("Failed to load stego image");
            let stego_gray = image_ops::extract_grayscale(&stego_img);
            let stego_matrix = image_ops::gray_image_to_matrix(&stego_gray);
            let stego_blocks = image_ops::split_into_blocks(&stego_matrix);

            
            let encrypted =
                stego::extract_payload_from_blocks(&stego_blocks).expect("Extraction failed");

            let decrypted =
                crypto::decrypt_payload(&encrypted, &password).expect("Decryption failed");

            println!("{}", String::from_utf8_lossy(&decrypted));
        }
    }
}