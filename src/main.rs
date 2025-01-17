use std::{ env, fs::File, path::Path, sync::Arc };
use image::{ DynamicImage, ImageBuffer };
use serde_json::Value;
use tokio::task::JoinSet;
use clap::Parser;

#[derive(Parser)]
#[clap(name = "thumbnail_gen")]
#[clap(about = "从视频生成缩略图网格")]
struct Args {
    /// 视频文件路径
    #[clap(value_parser, help = "视频文件路径，支持拖放或命令行参数")]
    video: String,

    /// 每行图片数量
    #[clap(short = 'r', long = "row", default_value = "7", help = "每行显示的图片数量，示例：-r 2")]
    row: u32,

    /// 每列图片数量
    #[clap(short = 'c', long = "col", default_value = "7", help = "每列显示的图片数量，示例：-c 3")]
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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let output = args.output.unwrap_or_else(||
        format!(
            "{}/{}.jpg",
            env::current_exe().unwrap().parent().unwrap().display(),
            Path::new(args.video.as_str()).file_name().unwrap().to_str().unwrap()
        )
    );

    let file = std::fs::File::create(&output).unwrap();

    let wid_pics = args.row;
    let hei_pics = args.col;

    // 调用ffprobe获取视频信息
    let video_path = Arc::new(args.video);

    let vid_info = get_vid_info(&video_path);

    // 计算最终图片的宽度和高度
    let final_width: u32 = vid_info.width * wid_pics + 10 * (wid_pics + 1);
    let final_height: u32 = vid_info.height * hei_pics + 10 * (hei_pics + 1);

    // 计算每隔多少秒取一帧
    let interval = (vid_info.duration / ((wid_pics * hei_pics) as f64)) * 0.9;

    // 调用ffmpeg提取图片
    let mut tasks = JoinSet::new();

    for i in 1..=wid_pics * hei_pics {
        let time = ((i as f64) * interval) as u32;
        let video_path = Arc::clone(&video_path);
        tasks.spawn(extract_pic(video_path, time, i));
    }

    let pics: Vec<(u32, Vec<u8>)> = tasks.join_all().await.into_iter().collect();

    // 按照索引排序
    let mut pics = pics;
    pics.sort_by_key(|(index, _)| *index);

    // 保存图片
    let mut imgbuf = image::ImageBuffer::new(final_width as u32, final_height as u32);

    let mut row = 1;
    let mut col = 1;

    for (i, (_, pic)) in pics.iter().enumerate() {
        // 计算当前图片的位置
        let x = col * 10 + (col - 1) * vid_info.width;
        let y = row * 10 + (row - 1) * vid_info.height;

        // 直接在原图上操作
        for py in 0..vid_info.height {
            for px in 0..vid_info.width {
                let base_index = (py * vid_info.width + px) * 3;
                let r = pic[base_index as usize];
                let g = pic[(base_index as usize) + 1];
                let b = pic[(base_index as usize) + 2];

                imgbuf.put_pixel(x + px, y + py, image::Rgb([r, g, b]));
            }
        }

        if (i + 1) % (wid_pics as usize) == 0 {
            row += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    // 保存图片
    let format = output.split('.').last().unwrap().to_lowercase();

    save_file(args.height, args.width, &format, file, args.quality, imgbuf);
}

struct VidInfo {
    width: u32,
    height: u32,
    duration: f64,
}

fn get_vid_info(video_path: &str) -> VidInfo {
    // 调用ffprobe获取视频信息
    let info = std::process::Command
        ::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-show_format",
            "-of",
            "json",
            video_path,
        ])
        .output()
        .expect("ffprobe failed");

    let info_str = String::from_utf8(info.stdout).expect("ffprobe output is not utf8");

    let json: Value = match serde_json::from_str(&info_str) {
        Ok(json) => json,
        Err(e) => {
            println!("Error parsing ffprobe output: {}", e);
            println!("ffprobe output: {}", info_str);
            panic!("ffprobe failed");
        }
    };

    let width = json["streams"][0]["width"].as_u64().expect("无法解析视频宽度") as u32;
    let height = json["streams"][0]["height"].as_u64().expect("无法解析视频高度") as u32;
    let duration = json["format"]["duration"]
        .as_str()
        .expect("无法解析视频时长")
        .parse::<f64>()
        .expect("无法解析视频时长");

    VidInfo {
        width,
        height,
        duration,
    }
}

// 截取图片
async fn extract_pic(video_path: Arc<String>, time: u32, index: u32) -> (u32, Vec<u8>) {
    println!("提取第 {} 秒的图片", time);

    let pic = std::process::Command
        ::new("ffmpeg")
        .args([
            "-ss",
            &time.to_string(),
            "-noaccurate_seek",
            "-i",
            &video_path,
            "-vframes",
            "1",
            "-an",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "pipe:1",
        ])
        .stderr(std::process::Stdio::piped()) // 捕获错误输出
        .stdout(std::process::Stdio::piped()) // 确保捕获标准输出
        .output()
        .expect("ffmpeg failed");

    // 添加错误检查
    if !pic.status.success() {
        let error = String::from_utf8_lossy(&pic.stderr);
        println!("FFmpeg error: {}", error);
        panic!("FFmpeg 执行失败");
    }

    (index, pic.stdout)
}

fn save_file(
    height: u32,
    width: u32,
    format: &str,
    file: File,
    quality: u8,
    img: ImageBuffer<image::Rgb<u8>, Vec<u8>>
) {
    let img = DynamicImage::from(img).resize(width, height, image::imageops::FilterType::Triangle);

    match format {
        "jpeg" | "jpg" => {
            img.write_with_encoder(
                image::codecs::jpeg::JpegEncoder::new_with_quality(file, quality)
            ).unwrap();
        }
        "png" => {
            img.write_with_encoder(
                image::codecs::png::PngEncoder::new_with_quality(
                    file,
                    image::codecs::png::CompressionType::Best,
                    image::codecs::png::FilterType::NoFilter
                )
            ).unwrap();
        }
        "webp" => {
            img.write_with_encoder(image::codecs::webp::WebPEncoder::new_lossless(file)).unwrap();
        }
        _ => panic!("不支持的格式"),
    }
}
