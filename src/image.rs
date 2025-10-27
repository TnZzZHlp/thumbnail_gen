use anyhow::{Context, Result};
use image::{DynamicImage, ImageBuffer};
use std::fs::File;

use crate::video::VidInfo;

/// 保存图片到文件
pub fn save_file(
    height: u32,
    width: u32,
    format: &str,
    file: File,
    quality: u8,
    img: ImageBuffer<image::Rgb<u8>, Vec<u8>>,
) -> Result<()> {
    let img = DynamicImage::from(img).resize(width, height, image::imageops::FilterType::Triangle);

    match format {
        "jpeg" | "jpg" => {
            img.write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(
                file, quality,
            ))
            .context("保存 JPEG 文件失败")?;
        }
        "png" => {
            img.write_with_encoder(image::codecs::png::PngEncoder::new_with_quality(
                file,
                image::codecs::png::CompressionType::Best,
                image::codecs::png::FilterType::NoFilter,
            ))
            .context("保存 PNG 文件失败")?;
        }
        "webp" => {
            img.write_with_encoder(image::codecs::webp::WebPEncoder::new_lossless(file))
                .context("保存 WebP 文件失败")?;
        }
        _ => anyhow::bail!("不支持的格式: {}", format),
    }

    Ok(())
}

/// 将提取的图片帧组合成网格
pub fn compose_thumbnail_grid(
    pics: Vec<(u32, Vec<u8>)>,
    vid_info: &VidInfo,
    row_count: u32,
    _col_count: u32,
    padding: u32,
    final_width: u32,
    final_height: u32,
) -> Result<ImageBuffer<image::Rgb<u8>, Vec<u8>>> {
    let mut imgbuf = ImageBuffer::new(final_width, final_height);

    let mut current_row = 0;
    let mut current_col = 0;

    for (_, pic_data) in pics.iter() {
        // 计算当前图片的位置
        let x = padding + current_col * (vid_info.width + padding);
        let y = padding + current_row * (vid_info.height + padding);

        // 将图片数据复制到画布
        for py in 0..vid_info.height {
            for px in 0..vid_info.width {
                let base_index = ((py * vid_info.width + px) * 3) as usize;

                if base_index + 2 < pic_data.len() {
                    let r = pic_data[base_index];
                    let g = pic_data[base_index + 1];
                    let b = pic_data[base_index + 2];
                    imgbuf.put_pixel(x + px, y + py, image::Rgb([r, g, b]));
                }
            }
        }

        // 移动到下一个位置
        current_col += 1;
        if current_col >= row_count {
            current_col = 0;
            current_row += 1;
        }
    }

    Ok(imgbuf)
}
