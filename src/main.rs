use clap::Parser;
use rayon::prelude::*;
use std::{env, path::Path, sync::Arc};

mod ffmpeg;
mod img_ops;

#[derive(Parser)]
#[clap(name = "thumbnail_gen")]
#[clap(about = "从视频生成缩略图网格")]
struct Args {
    /// 视频文件路径
    #[clap(value_parser, help = "视频文件路径，支持拖放或命令行参数")]
    video: String,

    /// 每行图片数量
    #[clap(
        short = 'r',
        long = "row",
        default_value = "7",
        help = "每行显示的图片数量，示例：-r 2"
    )]
    row: u32,

    /// 每列图片数量
    #[clap(
        short = 'c',
        long = "col",
        default_value = "7",
        help = "每列显示的图片数量，示例：-c 3"
    )]
    col: u32,

    /// 输出路径
    #[clap(
        short = 'o',
        long = "output",
        help = "输出文件路径，默认输出路径为程序同目录，支持jpeg、png和webp格式，示例：-o C:\\output.jpg"
    )]
    output: Option<String>,

    /// 生成图片的质量
    #[clap(
        short = 'q',
        long = "quality",
        default_value = "75",
        help = "生成图片的质量。仅对jpeg与webp有效。范围 0-100，默认 100，示例：-q 90"
    )]
    quality: u8,

    /// 生成图片的高度
    #[clap(
        long = "height",
        default_value = "100000",
        help = "生成图片的高度。图像的宽高比将被保留。图像会被缩放到尽可能大的尺寸，同时确保其尺寸不超过由 width 和 height 定义的边界。示例：--height 7680"
    )]
    height: u32,

    /// 生成图片的宽度
    #[clap(
        long = "width",
        default_value = "3840",
        help = "生成图片的宽度。图像的宽高比将被保留。图像会被缩放到尽可能大的尺寸，同时确保其尺寸不超过由 width 和 height 定义的边界。示例：--width 4320"
    )]
    width: u32,
}

fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let output = args.output.unwrap_or_else(|| {
        format!(
            "{}/{}.jpg",
            env::current_exe().unwrap().parent().unwrap().display(),
            Path::new(args.video.as_str())
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
        )
    });

    let file = std::fs::File::create(&output).unwrap();

    let wid_pics = args.row;
    let hei_pics = args.col;

    // 调用ffprobe获取视频信息
    let video_path = Arc::new(args.video);

    let vid_info = ffmpeg::get_vid_info(&video_path);

    // 计算缩放比例
    let full_width = vid_info.width * wid_pics + 10 * (wid_pics + 1);
    let full_height = vid_info.height * hei_pics + 10 * (hei_pics + 1);

    let scale_w = args.width as f64 / full_width as f64;
    let scale_h = args.height as f64 / full_height as f64;
    let scale = scale_w.min(scale_h).min(1.0); // 不放大

    let thumb_width = (vid_info.width as f64 * scale) as u32;
    let thumb_height = (vid_info.height as f64 * scale) as u32;
    let padding = (10.0 * scale) as u32;

    // 计算最终图片的宽度和高度
    let final_width: u32 = thumb_width * wid_pics + padding * (wid_pics + 1);
    let final_height: u32 = thumb_height * hei_pics + padding * (hei_pics + 1);

    // 计算每隔多少秒取一帧
    let interval = (vid_info.duration / ((wid_pics * hei_pics) as f64)) * 0.9;

    // 调用ffmpeg提取图片
    let pics: Vec<(u32, Vec<u8>)> = (1..=wid_pics * hei_pics)
        .into_par_iter()
        .map(|i| {
            let time = ((i as f64) * interval) as u32;
            let video_path = Arc::clone(&video_path);
            ffmpeg::extract_pic(video_path, time, i, thumb_width, thumb_height)
        })
        .collect();

    // 按照索引排序
    let mut pics = pics;
    pics.sort_by_key(|(index, _)| *index);

    // 保存图片
    let mut imgbuf = vec![0u8; (final_width * final_height * 3) as usize];

    // 使用 rayon 并行处理图片合成
    imgbuf
        .par_chunks_mut((final_width * 3) as usize)
        .enumerate()
        .for_each(|(y, row_slice)| {
            let y = y as u32;
            // 填充背景色 (白色)
            for pixel in row_slice.chunks_mut(3) {
                pixel[0] = 255;
                pixel[1] = 255;
                pixel[2] = 255;
            }

            // 计算当前行属于哪一行缩略图
            // y = row_idx * (thumb_height + padding) + padding + local_y
            // 但是我们需要反过来，给定 y，判断它是否在某个缩略图的垂直范围内
            // 每一行缩略图的高度区间:
            // Start: row_idx * (thumb_height + padding) + padding
            // End: Start + thumb_height

            let row_total_height = thumb_height + padding;
            if y < padding {
                return; // 顶部 padding
            }
            let effective_y = y - padding;
            let row_idx = effective_y / row_total_height;
            let local_y = effective_y % row_total_height;

            if local_y >= thumb_height {
                return; // 缩略图下方的 padding
            }

            if row_idx >= hei_pics {
                return;
            }

            // 这一行包含 wid_pics 个缩略图
            // 遍历这一行的所有缩略图
            for col_idx in 0..wid_pics {
                let pic_idx = (row_idx * wid_pics + col_idx) as usize;
                if pic_idx >= pics.len() {
                    continue;
                }

                let (_, pic_data) = &pics[pic_idx];

                // 计算该缩略图在当前行的水平位置
                let x_start = padding + col_idx * (thumb_width + padding);
                let x_end = x_start + thumb_width;

                // 复制像素数据
                // pic_data 是 thumb_width * thumb_height * 3
                // 我们需要 pic_data 中第 local_y 行的数据
                let src_start = (local_y * thumb_width * 3) as usize;
                let src_end = src_start + (thumb_width * 3) as usize;

                if src_end <= pic_data.len() {
                    let src_row = &pic_data[src_start..src_end];
                    let dest_start = (x_start * 3) as usize;
                    let dest_end = (x_end * 3) as usize;

                    if dest_end <= row_slice.len() {
                        row_slice[dest_start..dest_end].copy_from_slice(src_row);
                    }
                }
            }
        });

    let imgbuf = image::ImageBuffer::from_raw(final_width, final_height, imgbuf).unwrap();

    // 保存图片
    let format = output.split('.').next_back().unwrap().to_lowercase();

    // 已经缩放过了，这里传入 final_height/width 避免再次缩放
    img_ops::save_file(
        final_height,
        final_width,
        &format,
        file,
        args.quality,
        imgbuf,
    );
}
