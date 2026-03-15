use crate::scripts::{bitstream, crypto};

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

pub fn demo_crypto(password: &str, plaintext: &[u8]) -> Vec<u8> {
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