mod demo_tests;
mod scripts;

use clap::Parser;
use scripts::cli::Cli;
use scripts::{crypto, payload, utils};

use crate::demo_tests::demo_1d::demo_1d_dct;
use crate::demo_tests::demo_2d::demo_2d_dct;
use crate::demo_tests::demo_crypto::demo_crypto;
use crate::demo_tests::demo_image_and_stego::demo_image_and_stego;

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
            utils::embed_bytes_into_image(&input, &output, &encrypted, &password)
                .expect("Embedding failed");
            println!("Embed successful: {}", output);
        }

        scripts::cli::Commands::Extract { input, password } => {
            let encrypted =
                utils::extract_bytes_from_image(&input, &password).expect("Extraction failed");
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
            utils::embed_bytes_into_image(&input, &output, &encrypted, &password)
                .expect("Embedding failed");

            println!("Embed-file successful: {} -> {}", secret_file, output);
        }

        scripts::cli::Commands::ExtractFile {
            input,
            password,
            output,
        } => {
            let encrypted =
                utils::extract_bytes_from_image(&input, &password).expect("Extraction failed");
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
