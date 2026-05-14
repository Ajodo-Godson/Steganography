mod blocks;
mod color;

#[allow(unused_imports)]
pub use blocks::{
    BLOCK_SIZE, embeddable_block_indices, merge_blocks, pad_matrix_to_block_size, split_into_blocks,
};
pub use color::{extract_luma_and_chroma, luma_and_chroma_to_rgb_image};

use image::{DynamicImage, GrayImage, Luma};
use ndarray::Array2;

pub fn load_image(path: &str) -> image::ImageResult<DynamicImage> {
    image::open(path)
}

#[allow(dead_code)]
pub fn extract_grayscale(img: &DynamicImage) -> GrayImage {
    img.to_luma8()
}

#[allow(dead_code)]
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
