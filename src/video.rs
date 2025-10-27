use anyhow::{Context, Result};
use serde_json::Value;
use std::sync::Arc;

pub struct VidInfo {
    pub width: u32,
    pub height: u32,
    pub duration: f64,
}

/// 获取视频信息（宽度、高度、时长）
pub fn get_vid_info(video_path: &str) -> Result<VidInfo> {
    let output = std::process::Command::new("ffprobe")
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
        .context("无法执行 ffprobe，请确保已安装 FFmpeg 并添加到系统路径")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("ffprobe 执行失败: {}", error);
    }

    let info_str = String::from_utf8(output.stdout).context("ffprobe 输出不是有效的 UTF-8 编码")?;

    let json: Value = serde_json::from_str(&info_str).context("无法解析 ffprobe 输出")?;

    let width = json["streams"][0]["width"]
        .as_u64()
        .context("无法解析视频宽度")? as u32;

    let height = json["streams"][0]["height"]
        .as_u64()
        .context("无法解析视频高度")? as u32;

    let duration = json["format"]["duration"]
        .as_str()
        .context("无法解析视频时长")?
        .parse::<f64>()
        .context("视频时长格式错误")?;

    Ok(VidInfo {
        width,
        height,
        duration,
    })
}

/// 从视频中提取指定时间的帧
pub async fn extract_pic(video_path: Arc<String>, time: u32, index: u32) -> Result<(u32, Vec<u8>)> {
    let output = std::process::Command::new("ffmpeg")
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
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .output()
        .context("无法执行 ffmpeg")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("FFmpeg 执行失败（帧 {}）: {}", index, error);
    }

    Ok((index, output.stdout))
}
