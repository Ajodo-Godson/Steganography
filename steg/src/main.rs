mod scripts;

use clap::Parser;
use ndarray::Array2;
use scripts::cli::Cli;
use scripts::{bitstream, crypto, image_ops, payload, stego, transform};
use scripts::utils::{approx_eq_array2, approx_eq_vec};

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

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