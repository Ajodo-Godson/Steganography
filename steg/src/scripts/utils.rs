fn approx_eq_vec(left: &[f32], right: &[f32], epsilon: f32) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .all(|(a, b)| (a - b).abs() <= epsilon)
}

fn approx_eq_array2(left: &Array2<f32>, right: &Array2<f32>, epsilon: f32) -> bool {
    left.dim() == right.dim()
        && left
            .indexed_iter()
            .all(|((r, c), v)| (*v - right[(r, c)]).abs() <= epsilon)
}