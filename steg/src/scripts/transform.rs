//    The Forward and Inverse DCT math using rustdct
use rustdct::DctPlanner;
use ndarray::Array2;


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



pub fn forward_dct_2d_block(block: &Array2<f32>) -> Array2<f32> {
    assert_eq!(block.dim(), (8, 8), "forward_dct_2d_block expects an 8x8 block");

    let mut planner = DctPlanner::new();
    let dct2 = planner.plan_dct2(8);

    let mut result = block.clone();

    // DCT on rows
    for row in 0..8 {
        let mut row_data = result.row(row).to_vec();
        dct2.process_dct2(&mut row_data);
        for col in 0..8 {
            result[(row, col)] = row_data[col];
        }
    }

    // DCT on columns
    for col in 0..8 {
        let mut col_data = (0..8).map(|row| result[(row, col)]).collect::<Vec<_>>();
        dct2.process_dct2(&mut col_data);
        for row in 0..8 {
            result[(row, col)] = col_data[row];
        }
    }

    result
}

pub fn inverse_dct_2d_block(block: &Array2<f32>) -> Array2<f32> {
    assert_eq!(block.dim(), (8, 8), "inverse_dct_2d_block expects an 8x8 block");

    let mut planner = DctPlanner::new();
    let dct3 = planner.plan_dct3(8);

    let mut result = block.clone();

    // IDCT on rows
    for row in 0..8 {
        let mut row_data = result.row(row).to_vec();
        dct3.process_dct3(&mut row_data);

        let scale = 2.0 / 8.0;
        for value in &mut row_data {
            *value *= scale;
        }

        for col in 0..8 {
            result[(row, col)] = row_data[col];
        }
    }

    // IDCT on columns
    for col in 0..8 {
        let mut col_data = (0..8).map(|row| result[(row, col)]).collect::<Vec<_>>();
        dct3.process_dct3(&mut col_data);

        let scale = 2.0 / 8.0;
        for value in &mut col_data {
            *value *= scale;
        }

        for row in 0..8 {
            result[(row, col)] = col_data[row];
        }
    }

    result
}