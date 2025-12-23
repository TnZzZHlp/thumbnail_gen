use serde_json::Value;
use std::sync::Arc;

pub struct VidInfo {
    pub width: u32,
    pub height: u32,
    pub duration: f64,
}

pub fn get_vid_info(video_path: &str) -> VidInfo {
    // 调用ffprobe获取视频信息
    let info = std::process::Command::new("ffprobe")
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
            tracing::error!("Error parsing ffprobe output: {}", e);
            tracing::error!("ffprobe output: {}", info_str);
            panic!("ffprobe failed");
        }
    };

    let width = json["streams"][0]["width"]
        .as_u64()
        .expect("无法解析视频宽度") as u32;
    let height = json["streams"][0]["height"]
        .as_u64()
        .expect("无法解析视频高度") as u32;
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
pub fn extract_pic(
    video_path: Arc<String>,
    time: u32,
    index: u32,
    width: u32,
    height: u32,
) -> (u32, Vec<u8>) {
    tracing::info!("提取第 {} 秒的图片", time);

    let pic = std::process::Command::new("ffmpeg")
        .args([
            "-ss",
            &time.to_string(),
            "-noaccurate_seek",
            "-i",
            &video_path,
            "-vframes",
            "1",
            "-vf",
            &format!("scale={}:{}", width, height),
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
        tracing::error!("FFmpeg error: {}", error);
        panic!("FFmpeg 执行失败");
    }

    (index, pic.stdout)
}
