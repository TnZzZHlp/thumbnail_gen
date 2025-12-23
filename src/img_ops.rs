use image::{DynamicImage, ImageBuffer};
use std::fs::File;

pub fn save_file(
    height: u32,
    width: u32,
    format: &str,
    file: File,
    quality: u8,
    img: ImageBuffer<image::Rgb<u8>, Vec<u8>>,
) {
    let img = DynamicImage::from(img).resize(width, height, image::imageops::FilterType::Triangle);

    match format {
        "jpeg" | "jpg" => {
            img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(
                file, quality,
            ))
            .unwrap();
        }
        "png" => {
            img.write_with_encoder(image::codecs::png::PngEncoder::new_with_quality(
                file,
                image::codecs::png::CompressionType::Best,
                image::codecs::png::FilterType::NoFilter,
            ))
            .unwrap();
        }
        "webp" => {
            img.write_with_encoder(image::codecs::webp::WebPEncoder::new_lossless(file))
                .unwrap();
        }
        _ => {
            tracing::error!("不支持的格式: {}", format);
            panic!("不支持的格式");
        }
    }
}
