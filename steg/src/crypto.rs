// AES-GCM encryption, decryption, and nonce generation

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce, Key 
};
use rand::RngCore;
use std::convert::TryInto;



// The encryption key can be generated randomly:
let key = Aes256Gcm::generate_key(OsRng);

// Transformed from a byte array:
let key: &[u8; 32] = &[42; 32];
let key: &Key<Aes256Gcm> = key.into();

// Note that you can get byte array from slice using the `TryInto` trait:
let key: &[u8] = &[42; 32];
let key: [u8; 32] = key.try_into()?;

// Alternatively, the key can be transformed directly from a byte slice
// (panicks on length mismatch):
let key = Key::<Aes256Gcm>::from_slice(key);

let cipher = Aes256Gcm::new(&key);
let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
let ciphertext = cipher.encrypt(&nonce, b"plaintext message".as_ref())?;
let plaintext = cipher.decrypt(&nonce, ciphertext.as_ref())?;
assert_eq!(&plaintext, b"plaintext message");



pub fn create_password(password: &str) -> &str{
    // Create password and salt it
    random_salt = rand
    let salted_password = format!("{}{}", password, "some_random_salt");

}

pub fn encrypt_aes_gcm(plaintext: &[u8], key: &[u8], nonce: &[u8]) -> Vec<u8> {
    // Implement AES-GCM encryption logic here
    // Return the ciphertext as a vector of bytes
}

pub fn decrypt_aes_gcm(ciphertext: &[u8], key: &[u8], nonce: &[u8]) -> Vec<u8> {
    // Implement AES-GCM decryption logic here
    // Return the decrypted plaintext as a vector of bytes
}

pub fn generate_nonce() -> Vec<u8> {
    // Implement nonce generation logic here
    // Return the generated nonce as a vector of bytes
}

