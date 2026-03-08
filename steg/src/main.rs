mod scripts;
use scripts::crypto;
use scripts::transform;


fn main() {
    let plaintext = b"Hello, world!";
    let password = "super_secret_password";

    // Encrypt the plaintext
    let encrypted = crypto::encrypt_with_password(plaintext, password).unwrap();

    // Decrypt the ciphertext
    let decrypted = crypto::decrypt_with_password(&encrypted, password).unwrap();

    println!("Salt (hex): {}", hex::encode(&encrypted.salt));
    println!("Nonce (hex): {}", hex::encode(&encrypted.nonce));
    println!("Plain password: {}", password);
    println!("Plaintext: {}", String::from_utf8_lossy(plaintext));
    println!("Ciphertext (hex): {}", hex::encode(&encrypted.ciphertext));
    println!("Decrypted text: {}", String::from_utf8_lossy(&decrypted));

    assert_eq!(plaintext.to_vec(), decrypted);

    // Testing the transforms
    transform::demo_transform();
    transform::demo_inverse_transform();



}
