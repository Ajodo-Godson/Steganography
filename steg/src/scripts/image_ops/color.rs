use image::{DynamicImage, Rgb, RgbImage};
use ndarray::Array2;

fn clamp_to_u8(value: f32) -> u8 {
    value.round().clamp(0.0, 255.0) as u8
}

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

        luma[(y as usize, x as usize)] = 0.299 * r + 0.587 * g + 0.114 * b;
        cb[(y as usize, x as usize)] = 128.0 - 0.168_736 * r - 0.331_264 * g + 0.5 * b;
        cr[(y as usize, x as usize)] = 128.0 + 0.5 * r - 0.418_688 * g - 0.081_312 * b;
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
