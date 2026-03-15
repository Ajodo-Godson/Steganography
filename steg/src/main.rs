mod scripts;

use clap::Parser;
use ndarray::Array2;
use scripts::cli::Cli;
use scripts::{bitstream, crypto, image_ops, payload, stego, transform};

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
            .all(|((r, c), v)| (*v - right[(r, c)]).abs() <= epsilon)
}

fn embed_bytes_into_image(input: &str, output: &str, encrypted: &[u8]) -> Result<(), String> {
    let img = image_ops::load_image(input).map_err(|e| e.to_string())?;
    let gray = image_ops::extract_grayscale(&img);
    let matrix = image_ops::gray_image_to_matrix(&gray);
    let (height, width) = matrix.dim();
    let blocks = image_ops::split_into_blocks(&matrix);

    let bits_needed = 32 + encrypted.len() * 8;
    if bits_needed > blocks.len() {
        return Err(format!(
            "Payload too large for cover image: need {} bits, have {}",
            bits_needed,
            blocks.len()
        ));
    }

    let embedded_blocks = stego::embed_payload_in_blocks(&blocks, encrypted)?;
    let embedded_matrix = image_ops::merge_blocks(&embedded_blocks, height, width);
    let embedded_image = image_ops::matrix_to_gray_image(&embedded_matrix);

    embedded_image.save(output).map_err(|e| e.to_string())?;
    Ok(())
}

fn extract_bytes_from_image(input: &str) -> Result<Vec<u8>, String> {
    let stego_img = image_ops::load_image(input).map_err(|e| e.to_string())?;
    let stego_gray = image_ops::extract_grayscale(&stego_img);
    let stego_matrix = image_ops::gray_image_to_matrix(&stego_gray);
    let stego_blocks = image_ops::split_into_blocks(&stego_matrix);

    stego::extract_payload_from_blocks(&stego_blocks)
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

    let blocks = image_ops::split_into_blocks(&matrix);
    let rebuilt = image_ops::merge_blocks(&blocks, height, width);
    assert!(approx_eq_array2(&matrix, &rebuilt, 1e-5));

    gray.save("output/cat_gray.png").unwrap();

    let embedded_blocks = stego::embed_payload_in_blocks(&blocks, encrypted).unwrap();
    let embedded_matrix = image_ops::merge_blocks(&embedded_blocks, height, width);
    let embedded_image = image_ops::matrix_to_gray_image(&embedded_matrix);
    embedded_image.save("output/cat_stego.png").unwrap();

    let extracted_encrypted = extract_bytes_from_image("output/cat_stego.png").unwrap();
    let decrypted = crypto::decrypt_payload(&extracted_encrypted, password).unwrap();

    println!("Recovered stego text: {}", String::from_utf8_lossy(&decrypted));
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
            let packed = payload::encode_text(&message).expect("Failed to pack text payload");
            let encrypted = crypto::encrypt_payload(&packed, &password).expect("Encryption failed");
            embed_bytes_into_image(&input, &output, &encrypted).expect("Embedding failed");
            println!("Embed successful: {}", output);
        }

        scripts::cli::Commands::Extract { input, password } => {
            let encrypted = extract_bytes_from_image(&input).expect("Extraction failed");
            let packed = crypto::decrypt_payload(&encrypted, &password).expect("Decryption failed");

            match payload::decode_payload(&packed).expect("Invalid payload") {
                payload::Payload::Text(text) => println!("{}", text),
                payload::Payload::File { .. } => {
                    eprintln!("Payload is a file. Use `extract-file`.");
                }
            }
        }

        scripts::cli::Commands::EmbedFile {
            input,
            output,
            password,
            secret_file,
        } => {
            let bytes = std::fs::read(&secret_file).expect("Failed to read secret file");
            let name = std::path::Path::new(&secret_file)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("secret.bin");

            let packed = payload::encode_file(name, &bytes).expect("Failed to pack file payload");
            let encrypted = crypto::encrypt_payload(&packed, &password).expect("Encryption failed");
            embed_bytes_into_image(&input, &output, &encrypted).expect("Embedding failed");

            println!("Embed-file successful: {} -> {}", secret_file, output);
        }

        scripts::cli::Commands::ExtractFile {
            input,
            password,
            output,
        } => {
            let encrypted = extract_bytes_from_image(&input).expect("Extraction failed");
            let packed = crypto::decrypt_payload(&encrypted, &password).expect("Decryption failed");

            match payload::decode_payload(&packed).expect("Invalid payload") {
                payload::Payload::File { name, bytes } => {
                    let out_path = output.unwrap_or(name);
                    std::fs::write(&out_path, &bytes).expect("Failed to write extracted file");
                    println!("Extract-file successful: {}", out_path);
                }
                payload::Payload::Text(text) => {
                    eprintln!("Payload is text, not file:\n{}", text);
                }
            }
        }
    }
}