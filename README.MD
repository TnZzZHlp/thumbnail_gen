# 视频缩略图网格生成器

从视频文件生成缩略图网格预览图的命令行工具。
</br>

## 生成效果预览

![alt text](img/output.jpg)
</br>

## 功能特点

- 支持多种视频格式（mp4、mkv 等）
- 自定义输出图片行列数
- 可设置输出路径
- 支持文件拖放操作

## 依赖要求

- FFmpeg (需要安装并添加到系统环境变量)
- Windows 操作系统

## 使用方法

### 1. 直接拖放使用

直接将视频文件拖放到程序上即可，将在程序所在目录生成 `output.jpg`

### 2. 命令行参数

```bash
thumbnail_gen.exe [视频文件路径] [选项]
```

#### 选项说明:

`-r, --row <数字>` : 每行图片数量 (默认: 7)

`-c, --col <数字>` : 每列图片数量 (默认: 7)

`-o, --output <路径>` : 输出文件路径 (默认: 程序目录下的 output.jpg) 支持 jpeg、png 和 webp 格式

`-q, --quality <数字>` : 输出图片质量 (默认: 75)

`--height <数字>` : 输出图片高度 (默认: 100000)

`--width <数字>` : 输出图片宽度 (默认: 4320)

`-h, --help` : 显示帮助信息

### 使用示例

**基本使用:**

```bash
thumbnail_gen.exe "D:\视频.mp4"
```

**指定行列数:**

```bash
thumbnail_gen.exe -r 3 -c 4 "D:\视频.mp4"
```

**指定输出路径:**

```bash
thumbnail_gen.exe -o "D:\output.jpg" "D:\视频.mp4"
```

**指定生成图片质量**

```bash
thumbnail_gen.exe -q 100 "D:\视频.mp4"
```

**指定生成图片尺寸**

```bash
thumbnail_gen.exe --width 1920 "D:\视频.mp4"
```

**完整示例:**

```bash
thumbnail_gen.exe -r 2 -c 3 -o "D:\output.jpg" "D:\视频.mp4"
```

### 注意事项

**确保已安装 FFmpeg 并添加到环境变量**

**包含空格的路径需要用引号包围**

**输出目录必须已存在**

**建议使用绝对路径**
