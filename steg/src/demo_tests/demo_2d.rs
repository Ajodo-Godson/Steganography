mod scripts; 
use ndarray::Array2;
use scripts::transform;
use scripts::utils::{approx_eq_vec, approx_eq_array2};


const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;

fn demo_2d_dct() {
    let block = Array2::from_shape_vec(
        (8, 8),
        vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 3.0,
            4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 5.0,
            6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0,
            7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0,
            15.0,
        ],
    )
    .unwrap();

    let dct_block = transform::forward_dct_2d_block(&block);
    let restored_block = transform::inverse_dct_2d_block(&dct_block);

    println!("Original block:\n{:?}", block);
    println!("DCT block:\n{:?}", dct_block);
    println!("Restored block:\n{:?}", restored_block);

    assert!(approx_eq_array2(&block, &restored_block, 1e-3));
    println!("-----------------------------------");
}