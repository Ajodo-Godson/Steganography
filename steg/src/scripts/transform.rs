//    The Forward and Inverse DCT math using rustdct
use rustdct::DctPlanner;


pub fn forward_dct(input: &[f32]) -> Vec<f32> {
    let len = input.len();
    let mut planner = DctPlanner::new();
    let dct = planner.plan_dct2(len);

    let mut buffer = input.to_vec();
    dct.process_dct2(&mut buffer);
    buffer
}



pub fn inverse_dct(input_dct: &[f32]) -> Vec<f32> {
    let len = input_dct.len();
    let mut planner = DctPlanner::new();

    let idct = planner.plan_dct3(len); // dct3 is the inverse of dct2

    let mut buffer = input_dct.to_vec();
    
    idct.process_dct3(&mut buffer);


    let scale = 2.0 / len as f32;
    for value in &mut buffer {
        *value *= scale;
    }

    buffer
}

