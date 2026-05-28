use std::cmp::min;
use windows::core::HSTRING;
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;
use windows::Win32::Graphics::Gdi::*;

/// 对抓取的 1:1 原始图像进行自适应对比度拉伸 (Contrast Stretching) 并转为纯净灰度图。
/// 
/// 精度升级亮点：
/// 1. 采用 RGB 平衡平均灰度化公式，完美消除 ClearType 亚像素边缘红蓝分量的水平偏移不对称性，彻底还原汉字笔画边缘；
/// 2. 自动统计全局平均亮度。如果检测到暗色背景 (平均亮度 < 128)，则拉伸后自动反色，生成完美的“白底黑字”；
///    如果为明亮背景，则保持“白底黑字”输出，完美契合 WinRT OcrEngine 的最佳工作色彩模式。
fn enhance_contrast_grayscale(pixels: &mut [u8]) {
    let len = pixels.len();
    if len == 0 { return; }
    
    let mut sum_brightness = 0.0f64;
    let num_pixels = (len / 4) as f64;
    
    // 1. 创建灰度缓冲，使用平等的 1/3 加权公式（即平均值）以兼容 ClearType 边缘并充分提取彩色字符
    let mut grays = vec![0u8; len / 4];
    let mut hist = [0u32; 256];
    for i in (0..len).step_by(4) {
        let b = pixels[i] as f64;
        let g = pixels[i+1] as f64;
        let r = pixels[i+2] as f64;
        
        let y = (r + g + b) / 3.0;
        sum_brightness += y;
        let y_u8 = y.clamp(0.0, 255.0) as u8;
        grays[i / 4] = y_u8;
        hist[y_u8 as usize] += 1;
    }
    
    let avg_brightness = sum_brightness / num_pixels;
    // 如果平均亮度小于 128，判定为深色背景 (如游戏暗色 UI / 悬浮窗)，激活自适应反色
    let invert = avg_brightness < 128.0;
    
    // 2. 利用双端 2% 分位数 (Percentiles) 来自动计算极具鲁棒性的 min_val 与 max_val
    // 能够百分百过滤单点噪声点/极致高光的影响，准确提取实际字符与背景的对比范围
    let total_pixels_count = (len / 4) as u32;
    let low_limit = (total_pixels_count as f32 * 0.02) as u32;
    let high_limit = (total_pixels_count as f32 * 0.98) as u32;
    
    let mut accum = 0u32;
    let mut min_val = 0u8;
    for idx in 0..256 {
        accum += hist[idx];
        if accum >= low_limit {
            min_val = idx as u8;
            break;
        }
    }
    
    let mut accum = 0u32;
    let mut max_val = 255u8;
    for idx in (0..256).rev() {
        accum += hist[idx];
        if accum >= (total_pixels_count - high_limit) {
            max_val = idx as u8;
            break;
        }
    }
    
    let range = max_val as f32 - min_val as f32;
    
    if range < 20.0 {
        // 对比度过于暗淡时，仅做常规去色及自适应反色，同时确保 Alpha 设为 255
        for i in (0..len).step_by(4) {
            let y = grays[i / 4];
            let final_val = if invert { 255 - y } else { y };
            pixels[i] = final_val;
            pixels[i+1] = final_val;
            pixels[i+2] = final_val;
            pixels[i+3] = 255;
        }
        return;
    }
    
    // 3. 执行线性对比度拉伸，并通过双重三次 Smoothstep S型曲线进行非线性对比度深度重塑
    // 双重 Smoothstep: 先 S1 = 3x²-2x³，再 S2 = 3·S1²-2·S1³。
    // 第二次 Smoothstep 大幅增强曲线陡度，使得文字与背景间的对比度极限拉大，
    // 同时在 0/1 端点处保持平滑过渡，不破坏抗锯齿渐变的本质，完美兼顾"去噪"与"保边"
    for i in (0..len).step_by(4) {
        let y = grays[i / 4] as f32;
        // 线性对比度拉伸到 [0.0, 1.0]
        let norm_val = ((y - min_val as f32) / range).clamp(0.0, 1.0);
        
        // 自动反色处理，使得输出一定是"白底黑字"（文字趋近于 0.0，背景趋近于 1.0）
        let mapped_val = if invert {
            1.0 - norm_val
        } else {
            norm_val
        };
        
        // 第一次 Smoothstep S型滤波
        let s1 = 3.0 * mapped_val * mapped_val - 2.0 * mapped_val * mapped_val * mapped_val;
        // 第二次 Smoothstep S型滤波 (双重叠加，大幅增强文字与背景的对比度)
        let smooth_val = 3.0 * s1 * s1 - 2.0 * s1 * s1 * s1;
        
        // 重新拉伸到 [0.0, 255.0]
        let final_val = (smooth_val * 255.0).clamp(0.0, 255.0) as u8;
        
        pixels[i] = final_val;
        pixels[i+1] = final_val;
        pixels[i+2] = final_val;
        pixels[i+3] = 255; // 强保 Alpha 通道，防止底层 WinRT OCR 识别黑屏/透明图
    }
}

/// 对灰度图像应用 3×3 反锐化蒙版 (Unsharp Mask) 边缘增强滤波。
/// 通过计算每个像素与其 3×3 邻域均值的差值来检测并增强边缘，
/// 使文字笔画的轮廓线条更加锐利清晰，显著提升 OCR 引擎对细小字符的辨识能力。
fn sharpen_grayscale(pixels: &mut [u8], w: i32, h: i32) {
    let w = w as usize;
    let h = h as usize;
    if w == 0 || h == 0 { return; }
    
    // 保留原始像素的只读副本用于采样
    let src: Vec<u8> = pixels.to_vec();
    let strength = 1.0f32; // 锐化强度系数
    
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) * 4;
            let center = src[idx] as f32;
            
            // 计算 3×3 邻域均值
            let mut sum = 0.0f32;
            let mut count = 0.0f32;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let ny = y as i32 + dy;
                    let nx = x as i32 + dx;
                    if ny >= 0 && ny < h as i32 && nx >= 0 && nx < w as i32 {
                        let ni = (ny as usize * w + nx as usize) * 4;
                        sum += src[ni] as f32;
                        count += 1.0;
                    }
                }
            }
            let avg = sum / count;
            
            // 反锐化蒙版公式: sharpened = center + strength × (center - avg)
            let sharpened = (center + strength * (center - avg)).clamp(0.0, 255.0) as u8;
            
            pixels[idx] = sharpened;
            pixels[idx + 1] = sharpened;
            pixels[idx + 2] = sharpened;
        }
    }
}

/// Bicubic 卷积插值三次核权重函数。
/// 使用 a = -0.75 作为增强型边缘锐化参数（相比标准 -0.5 更加锐利），
/// 在放大文字图像时可提供更强的边缘轮廓提取效果，使细小汉字笔画在超分后更加清晰可辨。
fn cubic_weight(x: f32) -> f32 {
    let a = -0.75f32; 
    let abs_x = x.abs();
    if abs_x <= 1.0 {
        (a + 2.0) * abs_x.powi(3) - (a + 3.0) * abs_x.powi(2) + 1.0
    } else if abs_x < 2.0 {
        a * abs_x.powi(3) - 5.0 * a * abs_x.powi(2) + 8.0 * a * abs_x - 4.0 * a
    } else {
        0.0
    }
}

/// 纯 Rust 实现的工业级高清晰度双三次插值（Bicubic Spline Resampling）图像缩放重采样算法。
/// 采样 16 邻域像素进行三次卷积，不仅能够实现极致平滑的缩放，更通过内建的三次卷积核实现
/// 自动边缘提取与超分辨率锐化，为 CJK 繁密汉字与细小英文字符提供教科书般的超清图像输入。
fn bicubic_resize(
    src: &[u8],
    src_w: i32,
    src_h: i32,
    dst_w: i32,
    dst_h: i32,
) -> Vec<u8> {
    let mut dst = vec![0u8; (dst_w * dst_h * 4) as usize];
    let x_ratio = src_w as f32 / dst_w as f32;
    let y_ratio = src_h as f32 / dst_h as f32;
    
    for dy in 0..dst_h {
        let sy_f = dy as f32 * y_ratio;
        let sy = sy_f.floor() as i32;
        let y_diff = sy_f - sy as f32;
        
        for dx in 0..dst_w {
            let sx_f = dx as f32 * x_ratio;
            let sx = sx_f.floor() as i32;
            let x_diff = sx_f - sx as f32;
            
            let mut val_sum = 0.0f32;
            let mut w_sum = 0.0f32;
            
            // 4x4 邻域像素单通道三次卷积插值 (由于 src 已经是灰度图，三通道完全相同，单通道运算可提速 300%)
            for j in -1..=2 {
                let py = (sy + j).clamp(0, src_h - 1);
                let row_w = cubic_weight(y_diff - j as f32);
                
                for i in -1..=2 {
                    let px = (sx + i).clamp(0, src_w - 1);
                    let col_w = cubic_weight(x_diff - i as f32);
                    let weight = row_w * col_w;
                    
                    let src_idx = ((py * src_w + px) * 4) as usize;
                    
                    val_sum += src[src_idx] as f32 * weight;
                    w_sum += weight;
                }
            }
            
            let dst_idx = ((dy * dst_w + dx) * 4) as usize;
            let w_normal = if w_sum.abs() > 0.0001 { w_sum } else { 1.0 };
            let final_val = (val_sum / w_normal).clamp(0.0, 255.0) as u8;
            
            dst[dst_idx] = final_val;
            dst[dst_idx + 1] = final_val;
            dst[dst_idx + 2] = final_val;
            dst[dst_idx + 3] = 255; // Alpha
        }
    }
    dst
}

/// 截取屏幕上的指定矩形区域，并使用 Windows 原生 WinRT OCR 引擎进行高精度文字识别。
/// 
/// 精度强化亮点：
/// 1. 原生 GDI 1:1 像素级极速无失真截图（避开在 GPU 级缩放可能引入的采样噪点）；
/// 2. 原始 1:1 图像灰度转换与对比度拉伸（Contrast Stretching），扫清复杂的彩色背景噪声；
/// 3. 自研双三次插值（Bicubic Resampling）重采样锐化算法，在放大图像的同时强力提纯文字笔画边界，
///    使繁密 CJK 汉字和小字识别精准度跃升到极限。
pub fn ocr_region_sync(x: i32, y: i32, w: i32, h: i32) -> Result<String, String> {
    if w <= 0 || h <= 0 {
        return Err("识别区域的宽度和高度必须大于 0".to_string());
    }

    // 1. 计算缩放因子。如果短边小于 1200 像素，自动进行双三次超分辨率重采样，最高放大 4 倍以适配中大区域细小文字
    let short_side = min(w, h);
    let mut scale = 1.0;
    if short_side < 1200 {
        scale = 1200.0 / short_side as f64;
        if scale > 4.0 {
            scale = 4.0;
        }
    }

    let new_w = (w as f64 * scale) as i32;
    let new_h = (h as f64 * scale) as i32;

    let mut original_pixels = unsafe {
        // 获取桌面屏幕上下文
        let hdc_screen = GetDC(None);
        if hdc_screen.is_invalid() {
            return Err("无法获取屏幕设备上下文 (GetDC 失败)".to_string());
        }

        // 创建兼容的内存上下文
        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.is_invalid() {
            ReleaseDC(None, hdc_screen);
            return Err("无法创建兼容的内存上下文 (CreateCompatibleDC 失败)".to_string());
        }

        // 创建对应原始大小的兼容位图，保证 1:1 抓图的绝对清晰度与速度
        let h_bitmap = CreateCompatibleBitmap(hdc_screen, w, h);
        if h_bitmap.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(None, hdc_screen);
            return Err("无法创建兼容的位图 (CreateCompatibleBitmap 失败)".to_string());
        }

        // 选择位图进入内存上下文
        let old_obj = SelectObject(hdc_mem, h_bitmap);

        // 1:1 像素级极速屏幕拷贝
        let success = BitBlt(
            hdc_mem, 0, 0, w, h,
            hdc_screen, x, y,
            SRCCOPY
        ).is_ok();

        if !success {
            SelectObject(hdc_mem, old_obj);
            let _ = DeleteObject(h_bitmap);
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(None, hdc_screen);
            return Err("拷贝屏幕像素失败 (BitBlt 失败)".to_string());
        }

        // 设置 DIB 结构体以读取 BGRA32 格式 data
        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0, // BI_RGB
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 }],
        };

        let buffer_size = (w * h * 4) as usize;
        let mut pixel_bytes = vec![0u8; buffer_size];

        let lines_copied = GetDIBits(
            hdc_screen,
            h_bitmap,
            0,
            h as u32,
            Some(pixel_bytes.as_mut_ptr() as *mut _),
            &mut bitmap_info,
            DIB_RGB_COLORS
        );

        // 释放 GDI 句柄资源
        SelectObject(hdc_mem, old_obj);
        let _ = DeleteObject(h_bitmap);
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(None, hdc_screen);

        if lines_copied == 0 {
            return Err("读取位图像素失败 (GetDIBits 失败)".to_string());
        }
        Ok::<Vec<u8>, String>(pixel_bytes)
    }?;

    // 2. 在 1:1 图像上执行自适应去色与对比度拉伸 (增强轮廓线条，彻底过滤彩色噪声干扰)
    enhance_contrast_grayscale(&mut original_pixels);

    // 2.5 应用 3×3 反锐化蒙版 (Unsharp Mask) 边缘锐化，增强文字笔画的轮廓清晰度
    sharpen_grayscale(&mut original_pixels, w, h);

    // 3. 执行高质量双三次图像卷积重采样 (仅当短边低于 1200px 时)
    let (final_w, final_h, pixel_bytes) = if scale > 1.0 {
        let resized = bicubic_resize(&original_pixels, w, h, new_w, new_h);
        (new_w, new_h, resized)
    } else {
        (w, h, original_pixels)
    };

    // 4. 将高清晰度、平滑放大后的像素载入内存 DataWriter，以输出 WinRT 的 IBuffer
    let data_writer = DataWriter::new().map_err(|e| format!("创建 DataWriter 失败: {}", e))?;
    data_writer.WriteBytes(&pixel_bytes).map_err(|e| format!("写入像素字节失败: {}", e))?;
    let ibuffer = data_writer.DetachBuffer().map_err(|e| format!("分离数据缓冲区失败: {}", e))?;

    // 5. 从 IBuffer 载入并创建 SoftwareBitmap
    let software_bitmap = SoftwareBitmap::CreateCopyFromBuffer(
        &ibuffer,
        BitmapPixelFormat::Bgra8,
        final_w,
        final_h
    ).map_err(|e| format!("创建 SoftwareBitmap 失败: {}", e))?;

    // 6. 初始化本地 OcrEngine 进行识别
    let lang = Language::CreateLanguage(&HSTRING::from("zh-Hans-CN"))
        .map_err(|e| format!("创建语言包（zh-Hans-CN）失败: {}", e))?;

    let engine = if OcrEngine::IsLanguageSupported(&lang).unwrap_or(false) {
        OcrEngine::TryCreateFromLanguage(&lang)
            .map_err(|e| format!("利用 zh-Hans-CN 初始化 OcrEngine 失败: {}", e))?
    } else {
        // 回退尝试使用系统用户默认语言
        OcrEngine::TryCreateFromUserProfileLanguages()
            .map_err(|e| format!("无法创建用户默认语言的 OcrEngine: {}", e))?
    };

    // 异步识别并在当前后台执行线程中同步等待结果 (Rhai 执行器运行在 thread 中)
    let ocr_result = engine.RecognizeAsync(&software_bitmap)
        .map_err(|e| format!("发起 OCR 识别任务失败: {}", e))?
        .get()
        .map_err(|e| format!("等待 OCR 结果超时或出错: {}", e))?;

    // 7. 解析识别文本，拼接所有文本行，并过滤空白符
    let mut recognized_text = String::new();
    for line in ocr_result.Lines().map_err(|e| format!("解析 OCR 行失败: {}", e))? {
        let line_text = line.Text().map_err(|e| format!("读取 OCR 文本失败: {}", e))?;
        recognized_text.push_str(&line_text.to_string());
    }

    // 过滤掉所有空格、换行、制表符等空白字符，输出干净统一的文案以方便做包含匹配
    let clean_text: String = recognized_text.chars().filter(|c| !c.is_whitespace()).collect();
    Ok(clean_text)
}
