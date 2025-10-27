mod cli;
mod image;
mod video;

use anyhow::{Context, Result};
use clap::Parser;
use std::{env, fs::File, path::Path, sync::Arc};
use tokio::task::JoinSet;

use cli::Args;
use image::{compose_thumbnail_grid, save_file};
use video::{extract_pic, get_vid_info};

#[tokio::main]
async fn main() -> Result<()> {
    let start_time = std::time::Instant::now();
    let args = Args::parse();

    let output = args.output.clone().unwrap_or_else(|| {
        let exe_path = env::current_exe().expect("无法获取程序路径");
        let parent_dir = exe_path.parent().expect("无法获取程序目录");
        let video_filename = Path::new(&args.video)
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("无法解析视频文件名");

        format!("{}/{}.jpg", parent_dir.display(), video_filename)
    });

    println!("正在处理视频: {}", args.video);
    println!("输出路径: {}", output);

    let file = File::create(&output).context("无法创建输出文件")?;

    let video_path = Arc::new(args.video);

    // 获取视频信息
    let vid_info = get_vid_info(&video_path).context("无法获取视频信息")?;

    println!(
        "视频信息: {}x{}, 时长: {:.2}秒",
        vid_info.width, vid_info.height, vid_info.duration
    );

    // 计算最终图片的尺寸
    let padding = 10;
    let final_width = vid_info.width * args.row + padding * (args.row + 1);
    let final_height = vid_info.height * args.col + padding * (args.col + 1);

    println!("生成缩略图: {}行 x {}列", args.col, args.row);

    // 计算采样间隔（留10%缓冲避免超出视频时长）
    let total_frames = args.row * args.col;
    let interval = (vid_info.duration / total_frames as f64) * 0.9;

    // 并发提取图片帧
    let mut tasks = JoinSet::new();
    for i in 1..=total_frames {
        let time = ((i as f64) * interval) as u32;
        let video_path = Arc::clone(&video_path);
        tasks.spawn(extract_pic(video_path, time, i));
    }

    println!("正在提取 {} 帧...", total_frames);
    let extract_start = std::time::Instant::now();

    // 收集所有结果
    let mut pics = Vec::new();
    while let Some(result) = tasks.join_next().await {
        let pic = result.context("任务执行失败")?.context("提取图片失败")?;
        pics.push(pic);
    }

    println!(
        "✓ 帧提取完成，耗时: {:.2}秒",
        extract_start.elapsed().as_secs_f64()
    );

    // 按索引排序
    pics.sort_by_key(|(index, _)| *index);

    println!("正在组合图片...");
    let compose_start = std::time::Instant::now();

    // 创建画布并组合图片
    let imgbuf = compose_thumbnail_grid(
        pics,
        &vid_info,
        args.row,
        args.col,
        padding,
        final_width,
        final_height,
    )?;

    println!(
        "✓ 组合完成，耗时: {:.2}秒",
        compose_start.elapsed().as_secs_f64()
    );

    // 保存文件
    let format = output.rsplit('.').next().unwrap_or("jpg").to_lowercase();

    println!("正在保存为 {} 格式...", format);
    let save_start = std::time::Instant::now();
    save_file(args.height, args.width, &format, file, args.quality, imgbuf)?;

    println!(
        "✓ 保存完成，耗时: {:.2}秒",
        save_start.elapsed().as_secs_f64()
    );
    println!(
        "\n🎉 缩略图生成完成！总耗时: {:.2}秒",
        start_time.elapsed().as_secs_f64()
    );
    Ok(())
}
