use super::super::*;
use crate::scripts::image_ops;
use image::{DynamicImage, ImageFormat, Rgb, RgbImage};
use std::io::Cursor;

fn textured_cover(width: u32, height: u32) -> RgbImage {
    let mut img = RgbImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let v = if ((x / 4) + (y / 4)) % 2 == 0 {
                32
            } else {
                224
            };
            img.put_pixel(x, y, Rgb([v, 255 - v, v / 2]));
        }
    }
    img
}

fn embed_payload_image(payload: &[u8], password: &str) -> RgbImage {
    let cover = DynamicImage::ImageRgb8(textured_cover(128, 128));
    let (luma, cb, cr) = image_ops::extract_luma_and_chroma(&cover);
    let (height, width) = luma.dim();
    let blocks = image_ops::split_into_blocks(&luma);
    let usable = image_ops::embeddable_block_indices(height, width);
    let embedded = embed_payload_in_blocks(&blocks, &usable, payload, password).unwrap();
    let embedded_luma = image_ops::merge_blocks(&embedded, height, width);
    image_ops::luma_and_chroma_to_rgb_image(&embedded_luma, &cb, &cr)
}

fn extract_payload_image(img: DynamicImage, password: &str) -> Result<Vec<u8>, String> {
    let (luma, _, _) = image_ops::extract_luma_and_chroma(&img);
    let (height, width) = luma.dim();
    let blocks = image_ops::split_into_blocks(&luma);
    let usable = image_ops::embeddable_block_indices(height, width);
    extract_payload_from_blocks(&blocks, &usable, password)
}

#[test]
fn png_recompression_preserves_payload() {
    let payload = b"lossless image recompression";
    let stego = embed_payload_image(payload, "png");
    let mut encoded = Cursor::new(Vec::new());

    DynamicImage::ImageRgb8(stego)
        .write_to(&mut encoded, ImageFormat::Png)
        .unwrap();

    let decoded = image::load_from_memory_with_format(encoded.get_ref(), ImageFormat::Png).unwrap();
    let recovered = extract_payload_image(decoded, "png").unwrap();

    assert_eq!(recovered, payload);
}

#[test]
#[ignore = "current pixel-domain embedding is not JPEG recompression stable yet"]
fn jpeg_recompression_preserves_payload() {
    let payload = b"lossy image recompression";
    let stego = embed_payload_image(payload, "jpeg");
    let mut encoded = Cursor::new(Vec::new());
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut encoded, 95);

    encoder
        .encode_image(&DynamicImage::ImageRgb8(stego))
        .unwrap();

    let decoded =
        image::load_from_memory_with_format(encoded.get_ref(), ImageFormat::Jpeg).unwrap();
    let recovered = extract_payload_image(decoded, "jpeg").unwrap();

    assert_eq!(recovered, payload);
}
