pub mod crypto;



fn main() {
    let plaintext = b"Hello, world!";
    let password = "super_secret_password";

    // Encrypt the plaintext
    let encrypted = crypto::encrypt_with_password(plaintext, password).unwrap();

    // Decrypt the ciphertext
    let decrypted = crypto::decrypt_with_password(&encrypted, password).unwrap();

    print("Plain password: {}", password);
    print("Plaintext: {}", String::from_utf8_lossy(plaintext));
    print("Ciphertext (hex): {}", hex::encode(&encrypted.ciphertext));
    print("Decrypted text: {}", String::from_utf8_lossy(&decrypted));

    assert_eq!(plaintext.to_vec(), decrypted);
}
