mod scripts;
use scripts::crypto;
use scripts::transform;
use scripts::image_ops;
use ndarray::Array2;
use scripts::bitstream;


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
fn approx_eq_array2(a: &Array2<f32>, b: &Array2<f32>, epsilon: f32) -> bool {
    if a.dim() != b.dim() {
        return false;
    }

    for ((row, col), value) in a.indexed_iter() {
        if (*value - b[(row, col)]).abs() > epsilon {
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

    let encrypted = crypto::encrypt_payload(plaintext, password).unwrap();
    let bits = bitstream::bytes_to_bits(&encrypted);

    let salt = &encrypted[0..SALT_LEN];
    let nonce = &encrypted[SALT_LEN..SALT_LEN + NONCE_LEN];
    let ciphertext = &encrypted[SALT_LEN + NONCE_LEN..];

    // println!("Salt (hex): {}", hex::encode(salt));
    // println!("Nonce (hex): {}", hex::encode(nonce));
    // println!("Plain password: {}", password);
    // println!("Plaintext: {}", String::from_utf8_lossy(plaintext));
    // println!("Ciphertext (hex): {}", hex::encode(ciphertext));
    // println!("Encrypted payload in bits: {:?}", bits);
    println!("Salt bytes: {:?}", salt);
    println!("Nonce bytes: {:?}", nonce);
    println!("Ciphertext bytes: {:?}", ciphertext);

    println!("Salt bits: {:?}", bitstream::bytes_to_bits(salt));
    println!("Nonce bits: {:?}", bitstream::bytes_to_bits(nonce));
    println!("Ciphertext bits: {:?}", bitstream::bytes_to_bits(ciphertext));

    let decrypted = crypto::decrypt_payload(&encrypted, password).unwrap();
    println!("Decrypted text: {}", String::from_utf8_lossy(&decrypted));


    assert_eq!(plaintext.to_vec(), decrypted);
    println!("Encryption and decryption successful!");

    
    println!("-----------------------------------");


    // Testing the transforms
    let test_signal = vec![1.0, 2.0, 3.0, 4.0];
    let dct_result = transform::forward_dct(&test_signal);
    let idct_result = transform::inverse_dct(&dct_result);
    
    println!("Original signal: {:?}", test_signal);
    println!("DCT result: {:?}", dct_result);
    println!("IDCT result: {:?}", idct_result);
    assert_eq!(approx_eq_vec(&test_signal, &idct_result, 1e-5), true);


    println!("-----------------------------------");

    let block = Array2::from_shape_vec(
        (8, 8),
        vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0,
            2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0,
            3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0,
            4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0,
            5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0,
            7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0,
            8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
        ],
    )
    .unwrap();

    let dct_block = transform::forward_dct_2d_block(&block);
    let restored_block = transform::inverse_dct_2d_block(&dct_block);

    println!("Original block:\n{:?}", block);
    println!("DCT block:\n{:?}", dct_block);
    println!("Restored block:\n{:?}", restored_block);

    assert!(approx_eq_array2(&block, &restored_block, 1e-3));


    let img = image_ops::load_image("data/cat.jpg").unwrap();
    let gray = image_ops::extract_grayscale(&img);
    let matrix = image_ops::gray_image_to_matrix(&gray);
    let (height, width) = matrix.dim();

    println!("Image size: {}x{}", width, height);

    let blocks = image_ops::split_into_blocks(&matrix);
    println!("Number of blocks: {}", blocks.len()); 
    println!("Block size: {}x{}", image_ops::BLOCK_SIZE, image_ops::BLOCK_SIZE);

    let rebuilt = image_ops::merge_blocks(&blocks, height, width);
    assert!(approx_eq_array2(&matrix, &rebuilt, 1e-5));



    

    gray.save("output/cat_gray.png").unwrap();
    println!("Saved grayscale image to output/cat_gray.png");




}

