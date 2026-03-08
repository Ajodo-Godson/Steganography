//    The Forward and Inverse DCT math using rustdct
use rustdct::DctPlanner;


pub fn e_k(k: u32) -> f32 {
    if k == 0 {
        1.0 / (2.0 as f32).sqrt()
    } else {
        1.0
    }
}

pub fn forward_dct(input: &[f32]) -> Vec<f32> {
    let len = input.len();
    let mut planner = DctPlanner::new();
    let dct = planner.plan_dct2(len);

    let mut buffer = input.to_vec();
    dct.process_dct2(&mut buffer);
    buffer
}

// pub fn inverse_dct(input_dct: &[f32]) -> Vec<f32>{
//     let len = input_dct.len();
//     let scalar = 2.0 / len as f32;
//     println!("Scalar for normalization: {}", scalar);
//     println!("Input DCT values: {:?}", input_dct);

//     let x_n = input_dct.iter().map(|&x| x / scalar).collect::<Vec<f32>>();
//     x_n


//     // let x_n = (2.0 / len as f32) * input_dct.iter().enumerate().map(|(k, &X_k)| {
//     //     let e_k = e_k(k as f32);
//     //     e_k * X_k * (0..len).map(|n| {
//     //         (std::f32::consts::PI * k as f32 * (2.0 * n as f32 + 1.0) / (2.0 * len as f32)).cos()
//     //     }).sum::<f32>()
//     // }).collect::<Vec<f32>>();
//     // x_n

// }



pub fn inverse_dct(input_dct: &[f32]) -> Vec<f32> {
    let len = input_dct.len();
    let mut planner = DctPlanner::new();

    let idct = planner.plan_dct3(len); // dct3 is the inverse of dct2

    let mut buffer = input_dct.to_vec();
    
    idct.process_dct3(&mut buffer);


    let scale = 2.0 * len as f32;
    for value in &mut buffer {
        *value /= scale;
    }

    buffer
}

