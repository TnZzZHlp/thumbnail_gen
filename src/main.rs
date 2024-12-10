extern crate ffmpeg_next as ffmpeg;

use ffmpeg::format::{ input, Pixel };
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{ context::Context, flag::Flags };
use ffmpeg::util::frame::video::Video;
use std::env;

static WID_PICS: u64 = 4;
static HEI_PICS: u64 = 4;

fn main() -> Result<(), ffmpeg::Error> {
    println!("开始运行");
    ffmpeg::init().unwrap();

    let mut pics: Vec<Box<[u8]>> = Vec::new();
    let mut final_height = 0;
    let mut final_width = 0;
    let mut pic_height = 0;
    let mut pic_width = 0;

    if let Ok(mut ictx) = input(&env::args().nth(1).expect("Cannot open file.")) {
        let input = ictx.streams().best(Type::Video).ok_or(ffmpeg::Error::StreamNotFound)?;
        let video_stream_index = input.index();

        let context_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?;
        let mut decoder = context_decoder.decoder().video()?;

        let mut scaler = Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR
        )?;

        let interval =
            (ictx.streams().best(ffmpeg::media::Type::Video).expect("没有视频流").frames() as u64) /
            (WID_PICS * HEI_PICS);

        let mut frame_index = 1;

        let mut receive_and_process_decoded_frames = |
            decoder: &mut ffmpeg::decoder::Video
        | -> Result<(), ffmpeg::Error> {
            let mut decoded = Video::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                if frame_index % interval == 0 {
                    println!("Decoded frame {}", frame_index);
                    let mut rgb_frame = Video::empty();
                    scaler.run(&decoded, &mut rgb_frame)?;
                    pics.push(rgb_frame.data(0).to_vec().into_boxed_slice());
                }
                frame_index += 1;
            }
            final_height = decoder.height() * (HEI_PICS as u32) + ((HEI_PICS as u32) + 1) * 10;
            final_width = decoder.width() * (WID_PICS as u32) + ((WID_PICS as u32) + 1) * 10;

            pic_height = decoder.height();
            pic_width = decoder.width();

            Ok(())
        };

        for (stream, mut packet) in ictx.packets() {
            if stream.index() == video_stream_index {
                packet.set_position(200000);
                decoder.send_packet(&packet)?;
                receive_and_process_decoded_frames(&mut decoder)?;
            }
        }
        decoder.send_eof()?;
        receive_and_process_decoded_frames(&mut decoder)?;
    }

    // 生成缩略图
    let mut imgbuf = image::ImageBuffer::new(final_width, final_height);

    let mut row = 1;
    let mut col = 1;

    for (i, pic) in pics.iter().enumerate() {
        // 计算当前图片的位置
        let x = col * 10 + (col - 1) * pic_width;
        let y = row * 10 + (row - 1) * pic_height;

        // 直接在原图上操作
        for py in 0..pic_height {
            for px in 0..pic_width {
                let base_index = (py * pic_width + px) * 3;
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

    Ok(())
}
