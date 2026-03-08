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
    let test_signal = vec![1.0, 2.0, 3.0, 4.0];
    let dct_result = transform::forward_dct(&test_signal);
    let idct_result = transform::inverse_dct(&dct_result);
    
    println!("Original signal: {:?}", test_signal);
    println!("DCT result: {:?}", dct_result);
    println!("IDCT result: {:?}", idct_result);
    assert_eq!(test_signal, idct_result);



}
