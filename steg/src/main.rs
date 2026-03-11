mod scripts;
use scripts::crypto;
use scripts::transform;

fn approx_eq_vec(vec1: &[f32], vec2: &[f32], epsilon: f32) -> bool {
    if vec1.len() != vec2.len() {
        return false;
    }
    for (a, b) in vec1.iter().zip(vec2.iter()) {
        if (a - b).abs() > epsilon {
            return false;
        }
    }
    true
}


fn main() {

    const SALT_LEN: usize = 16;
    const NONCE_LEN: usize = 12;
    let password = "super_secret_password";
    let plaintext = b"Hello, world!";

    let encrypted = scripts::crypto::encrypt_payload(plaintext, password).unwrap();

    let salt = &encrypted[0..SALT_LEN];
    let nonce = &encrypted[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &encrypted[SALT_LEN + NONCE_LEN..];

    println!("Salt (hex): {}", hex::encode(salt));
    println!("Nonce (hex): {}", hex::encode(nonce));
    println!("Plain password: {}", password);
    println!("Plaintext: {}", String::from_utf8_lossy(plaintext));
    println!("Ciphertext (hex): {}", hex::encode(ciphertext));

    let decrypted = scripts::crypto::decrypt_payload(&encrypted, password).unwrap();
    println!("Decrypted text: {}", String::from_utf8_lossy(&decrypted));

    assert_eq!(plaintext.to_vec(), decrypted);
    println!("Encryption and decryption successful!");


    println!("Encryption and decryption successful!");
    println!("-----------------------------------");
    println!("-----------------------------------");


    // Testing the transforms
    let test_signal = vec![1.0, 2.0, 3.0, 4.0];
    let dct_result = transform::forward_dct(&test_signal);
    let idct_result = transform::inverse_dct(&dct_result);
    
    println!("Original signal: {:?}", test_signal);
    println!("DCT result: {:?}", dct_result);
    println!("IDCT result: {:?}", idct_result);
    assert_eq!(approx_eq_vec(&test_signal, &idct_result, 1e-5), true);
    



}
