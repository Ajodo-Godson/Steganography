use image::{DynamicImage, GrayImage, Luma, Rgb, RgbImage};
use ndarray::{Array2, s};

pub const BLOCK_SIZE: usize = 8;

pub fn load_image(path: &str) -> image::ImageResult<DynamicImage> {
    image::open(path)
}

fn clamp_to_u8(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

// GrayScale Image Operations
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

// LUMA CHROMA

pub fn extract_luma_and_chroma(img: &DynamicImage) -> (Array2<f32>, Array2<f32>, Array2<f32>) {
    let rgb = img.to_rgb8();
    let (width, height) = rgb.dimensions();

    let mut luma = Array2::<f32>::zeros((height as usize, width as usize));
    let mut cb = Array2::<f32>::zeros((height as usize, width as usize));
    let mut cr = Array2::<f32>::zeros((height as usize, width as usize));

    for (x, y, pixel) in rgb.enumerate_pixels() {
        let r = pixel[0] as f32;
        let g = pixel[1] as f32;
        let b = pixel[2] as f32;

        let yy = 0.299 * r + 0.587 * g + 0.114 * b;
        let cbv = 128.0 - 0.168_736 * r - 0.331_264 * g + 0.5 * b;
        let crv = 128.0 + 0.5 * r - 0.418_688 * g - 0.081_312 * b;

        luma[(y as usize, x as usize)] = yy;
        cb[(y as usize, x as usize)] = cbv;
        cr[(y as usize, x as usize)] = crv;
    }

    (luma, cb, cr)
}

pub fn luma_and_chroma_to_rgb_image(
    luma: &Array2<f32>,
    cb: &Array2<f32>,
    cr: &Array2<f32>,
) -> RgbImage {
    assert_eq!(luma.dim(), cb.dim(), "Cb must match luma dimensions");
    assert_eq!(luma.dim(), cr.dim(), "Cr must match luma dimensions");

    let (height, width) = luma.dim();
    let mut img = RgbImage::new(width as u32, height as u32);

    for y in 0..height {
        for x in 0..width {
            let yy = luma[(y, x)];
            let cb_shift = cb[(y, x)] - 128.0;
            let cr_shift = cr[(y, x)] - 128.0;

            let r = yy + 1.402 * cr_shift;
            let g = yy - 0.344_136 * cb_shift - 0.714_136 * cr_shift;
            let b = yy + 1.772 * cb_shift;

            img.put_pixel(
                x as u32,
                y as u32,
                Rgb([clamp_to_u8(r), clamp_to_u8(g), clamp_to_u8(b)]),
            );
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
            let block = padded
                .slice(s![y..y + BLOCK_SIZE, x..x + BLOCK_SIZE])
                .to_owned();
            blocks.push(block);
        }
    }

    blocks
}

pub fn merge_blocks(blocks: &[Array2<f32>], height: usize, width: usize) -> Array2<f32> {
    let padded_height = height.div_ceil(BLOCK_SIZE) * BLOCK_SIZE;
    let padded_width = width.div_ceil(BLOCK_SIZE) * BLOCK_SIZE;
    let mut padded = Array2::<f32>::zeros((padded_height, padded_width));

    let mut block_index = 0;
    for y in (0..padded_height).step_by(BLOCK_SIZE) {
        for x in (0..padded_width).step_by(BLOCK_SIZE) {
            if block_index < blocks.len() {
                let block = &blocks[block_index];
                padded
                    .slice_mut(s![y..y + BLOCK_SIZE, x..x + BLOCK_SIZE])
                    .assign(block);
                block_index += 1;
            }
        }
    }

    padded.slice(s![..height, ..width]).to_owned()
}

pub fn embeddable_block_indices(height: usize, width: usize) -> Vec<usize> {
    let padded_block_cols = width.div_ceil(BLOCK_SIZE);
    let full_block_rows = height / BLOCK_SIZE;
    let full_block_cols = width / BLOCK_SIZE;

    let mut indices = Vec::with_capacity(full_block_rows * full_block_cols);
    for block_row in 0..full_block_rows {
        for block_col in 0..full_block_cols {
            indices.push(block_row * padded_block_cols + block_col);
        }
    }

    indices
}
