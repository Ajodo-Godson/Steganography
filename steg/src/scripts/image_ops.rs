// Luma/Chroma extraction, ndarray chunking, 8x8 blocks
 
use image::{DynamicImage, GrayImage, Luma};
use ndarray::{s, Array2};

pub const BLOCK_SIZE: usize = 8;

pub fn load_image(path: &str) -> image::ImageResult<DynamicImage> {
    image::open(path)
}

pub fn extract_grayscale(img: &DynamicImage) -> GrayImage {
    img.to_luma8()
}

pub fn gray_image_to_matrix(img: &GrayImage) -> Array2<f32> {
    let (width, height) = img.dimensions();
    let mut matrix = Array2::<f32>::zeros((height as usize, width as usize));

    for (x, y, pixel) in img.enumerate_pixels() {
        matrix[(y as usize, x as usize)] = pixel[0] as f32;
    }

    matrix
}

pub fn matrix_to_gray_image(matrix: &Array2<f32>) -> GrayImage {
    let (height, width) = matrix.dim();
    let mut img = GrayImage::new(width as u32, height as u32);

    for y in 0..height {
        for x in 0..width {
            let value = matrix[(y, x)].round().clamp(0.0, 255.0) as u8;
            img.put_pixel(x as u32, y as u32, Luma([value]));
        }
    }

    img
}

pub fn pad_matrix_to_block_size(matrix: &Array2<f32>) -> Array2<f32> {
    let (height, width) = matrix.dim();
    //let padded_height = ((height + BLOCK_SIZE - 1) / BLOCK_SIZE) * BLOCK_SIZE;
    let padded_height = height.div_ceil(BLOCK_SIZE) * BLOCK_SIZE;
    //let padded_width = ((width + BLOCK_SIZE - 1) / BLOCK_SIZE) * BLOCK_SIZE;
    let padded_width = width.div_ceil(BLOCK_SIZE) * BLOCK_SIZE;

    let mut padded = Array2::<f32>::zeros((padded_height, padded_width));
    padded.slice_mut(s![..height, ..width]).assign(matrix);
    padded
}


pub fn split_into_blocks(matrix: &Array2<f32>) -> Vec<Array2<f32>> {
    let padded = pad_matrix_to_block_size(matrix);
    let (height, width) = padded.dim();
    let mut blocks = Vec::new();

    for y in (0..height).step_by(BLOCK_SIZE) {
        for x in (0..width).step_by(BLOCK_SIZE) {
            let block = padded.slice(s![y..y + BLOCK_SIZE, x..x + BLOCK_SIZE]).to_owned();
            blocks.push(block);
        }
    }

    blocks
}