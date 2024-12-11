use std::{ env, sync::Arc };

use tokio::task::JoinSet;

static WID_PICS: u32 = 7;
static HEI_PICS: u32 = 7;

#[tokio::main]
async fn main() {
    // 调用ffprobe获取视频信息
    let video_path = Arc::new(env::args().nth(1).expect("no video provided"));

    let vid_info = get_vid_info(&video_path);

    // 计算最终图片的宽度和高度
    let final_width: u32 = vid_info.width * WID_PICS + 10 * (WID_PICS + 1);
    let final_height: u32 = vid_info.height * HEI_PICS + 10 * (HEI_PICS + 1);

    // 计算每隔多少秒取一帧
    let interval = (vid_info.duration / ((WID_PICS * HEI_PICS) as f64)) * 0.9;

    println!("{}", interval);

    // 调用ffmpeg提取图片
    let mut tasks = JoinSet::new();

    for i in 1..=WID_PICS * HEI_PICS {
        let time = ((i as f64) * interval) as u32;
        let video_path = Arc::clone(&video_path);
        tasks.spawn(extract_pic(video_path, time));
    }

    let pics: Vec<Vec<u8>> = tasks.join_all().await.into_iter().collect();

    // 保存图片
    let mut imgbuf = image::ImageBuffer::new(final_width as u32, final_height as u32);

    let mut row = 1;
    let mut col = 1;

    for (i, pic) in pics.iter().enumerate() {
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

        if (i + 1) % (WID_PICS as usize) == 0 {
            row += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    imgbuf.save("output.jpg").unwrap();
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
            "stream=width,height,duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            video_path,
        ])
        .output()
        .expect("ffprobe failed");

    let info_str = String::from_utf8(info.stdout).expect("ffprobe output is not utf8");

    let mut lines = info_str.lines();

    // 分别解析宽度、高度和时长
    let width = lines
        .next()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .expect("无法解析视频宽度");

    let height = lines
        .next()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .expect("无法解析视频高度");

    let duration = lines
        .next()
        .and_then(|s| s.trim().parse::<f64>().ok())
        .expect("无法解析视频时长");

    VidInfo {
        width,
        height,
        duration,
    }
}

// 截取图片
async fn extract_pic(video_path: Arc<String>, time: u32) -> Vec<u8> {
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
            "1", // 改用 -vframes
            "-an",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "pipe:1", // 使用 pipe:1 替代 -
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

    // 检查输出大小
    if pic.stdout.is_empty() {
        println!("警告：FFmpeg 没有输出任何数据");
        panic!("提取图片失败");
    }

    pic.stdout
}
