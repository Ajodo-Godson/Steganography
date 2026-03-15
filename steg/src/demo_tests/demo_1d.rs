use crate::scripts::transform;
use crate::scripts::utils::approx_eq_vec;

pub fn demo_1d_dct() {
    let signal = vec![1.0, 2.0, 3.0, 4.0];
    let dct = transform::forward_dct(&signal);
    let restored = transform::inverse_dct(&dct);

    println!("Original signal: {:?}", signal);
    println!("DCT result: {:?}", dct);
    println!("IDCT result: {:?}", restored);

    assert!(approx_eq_vec(&signal, &restored, 1e-5));
    println!("-----------------------------------");
}