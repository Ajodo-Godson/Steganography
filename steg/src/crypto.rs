use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 600_000;

pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

pub fn generate_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

pub fn encrypt_aes_gcm(
    plaintext: &[u8],
    key: &[u8; KEY_LEN],
    nonce: &[u8; NONCE_LEN],
) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new_from_slice(key).expect("invalid AES-256 key length");
    cipher.encrypt(Nonce::from_slice(nonce), plaintext)
}

pub fn decrypt_aes_gcm(
    ciphertext: &[u8],
    key: &[u8; KEY_LEN],
    nonce: &[u8; NONCE_LEN],
) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new_from_slice(key).expect("invalid AES-256 key length");
    cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
}

pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub salt: [u8; SALT_LEN],
    pub nonce: [u8; NONCE_LEN],
}

pub fn encrypt_with_password(
    plaintext: &[u8],
    password: &str,
) -> Result<EncryptedData, aes_gcm::Error> {
    let salt = generate_salt();
    let nonce = generate_nonce();
    let key = derive_key(password, &salt);
    let ciphertext = encrypt_aes_gcm(plaintext, &key, &nonce)?;

    Ok(EncryptedData {
        ciphertext,
        salt,
        nonce,
    })
}

pub fn decrypt_with_password(
    encrypted: &EncryptedData,
    password: &str,
) -> Result<Vec<u8>, aes_gcm::Error> {
    let key = derive_key(password, &encrypted.salt);
    decrypt_aes_gcm(&encrypted.ciphertext, &key, &encrypted.nonce)
}