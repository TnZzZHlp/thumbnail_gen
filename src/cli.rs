use clap::Parser;

#[derive(Parser)]
#[clap(name = "thumbnail_gen")]
#[clap(about = "从视频生成缩略图网格", version)]
pub struct Args {
    /// 视频文件路径
    #[clap(value_parser, help = "视频文件路径，支持拖放或命令行参数")]
    pub video: String,

    /// 每行图片数量
    #[clap(short = 'r', long = "row", default_value = "7", value_parser = clap::value_parser!(u32).range(1..=20), help = "每行显示的图片数量，示例：-r 2")]
    pub row: u32,

    /// 每列图片数量
    #[clap(short = 'c', long = "col", default_value = "7", value_parser = clap::value_parser!(u32).range(1..=20), help = "每列显示的图片数量，示例：-c 3")]
    pub col: u32,

    /// 输出路径
    #[clap(
        short = 'o',
        long = "output",
        help = "输出文件路径，默认输出路径为程序同目录，支持jpeg、png和webp格式，示例：-o C:\\output.jpg"
    )]
    pub output: Option<String>,

    /// 生成图片的质量
    #[clap(
        short = 'q',
        long = "quality",
        default_value = "75",
        value_parser = clap::value_parser!(u8).range(1..=100),
        help = "生成图片的质量。仅对jpeg与webp有效。范围 0-100，默认 75，示例：-q 90"
    )]
    pub quality: u8,

    /// 生成图片的高度
    #[clap(
        long = "height",
        default_value = "100000",
        help = "生成图片的高度。图像的宽高比将被保留。图像会被缩放到尽可能大的尺寸，同时确保其尺寸不超过由 width 和 height 定义的边界。示例：--height 7680"
    )]
    pub height: u32,

    /// 生成图片的宽度
    #[clap(
        long = "width",
        default_value = "3840",
        help = "生成图片的宽度。图像的宽高比将被保留。图像会被缩放到尽可能大的尺寸，同时确保其尺寸不超过由 width 和 height 定义的边界。示例：--width 4320"
    )]
    pub width: u32,
}
