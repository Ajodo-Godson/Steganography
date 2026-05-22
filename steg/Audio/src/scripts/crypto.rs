use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const PBKDF2_ITERATIONS: u32 = 600_000;

fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

fn generate_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_LEN] {
    let mut key = [0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

fn encrypt_aes_gcm(
    plaintext: &[u8],
    key: &[u8; KEY_LEN],
    nonce: &[u8; NONCE_LEN],
) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| aes_gcm::Error)?;
    cipher.encrypt(Nonce::from_slice(nonce), plaintext)
}

fn decrypt_aes_gcm(
    ciphertext: &[u8],
    key: &[u8; KEY_LEN],
    nonce: &[u8; NONCE_LEN],
) -> Result<Vec<u8>, aes_gcm::Error> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| aes_gcm::Error)?;
    cipher.decrypt(Nonce::from_slice(nonce), ciphertext)
}

/// Encrypts `payload` with a password-derived key. The returned blob is
/// `[salt (16)] [nonce (12)] [ciphertext]` — self-contained for decryption.
pub fn encrypt_payload(payload: &[u8], password: &str) -> Result<Vec<u8>, aes_gcm::Error> {
    let salt = generate_salt();
    let nonce = generate_nonce();
    let key = derive_key(password, &salt);
    let ciphertext = encrypt_aes_gcm(payload, &key, &nonce)?;

    let mut packed = Vec::with_capacity(SALT_LEN + NONCE_LEN + ciphertext.len());
    packed.extend_from_slice(&salt);
    packed.extend_from_slice(&nonce);
    packed.extend_from_slice(&ciphertext);
    Ok(packed)
}

/// Decrypts a blob produced by [`encrypt_payload`].
pub fn decrypt_payload(packed: &[u8], password: &str) -> Result<Vec<u8>, aes_gcm::Error> {
    if packed.len() < SALT_LEN + NONCE_LEN {
        return Err(aes_gcm::Error);
    }

    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&packed[..SALT_LEN]);

    let mut nonce = [0u8; NONCE_LEN];
    nonce.copy_from_slice(&packed[SALT_LEN..SALT_LEN + NONCE_LEN]);

    let ciphertext = &packed[SALT_LEN + NONCE_LEN..];
    let key = derive_key(password, &salt);
    decrypt_aes_gcm(ciphertext, &key, &nonce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_round_trip() {
        let payload = b"secret audio payload";
        let password = "hunter2";

        let encrypted = encrypt_payload(payload, password).unwrap();
        let decrypted = decrypt_payload(&encrypted, password).unwrap();

        assert_eq!(decrypted, payload);
    }

    #[test]
    fn wrong_password_fails_decryption() {
        let encrypted = encrypt_payload(b"data", "correct").unwrap();
        assert!(decrypt_payload(&encrypted, "wrong").is_err());
    }

    #[test]
    fn truncated_blob_fails_decryption() {
        let blob = vec![0u8; SALT_LEN + NONCE_LEN - 1];
        assert!(decrypt_payload(&blob, "pw").is_err());
    }

    #[test]
    fn each_encryption_produces_unique_ciphertext() {
        let payload = b"same data";
        let a = encrypt_payload(payload, "pw").unwrap();
        let b = encrypt_payload(payload, "pw").unwrap();
        assert_ne!(a, b);
    }
}
