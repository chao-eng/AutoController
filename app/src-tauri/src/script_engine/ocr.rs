use std::cmp::min;
use windows::core::HSTRING;
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;
use windows::Win32::Graphics::Gdi::*;

/// 截取屏幕上的指定矩形区域，并使用 Windows 原生 WinRT OCR 引擎进行文字识别。
/// 
/// 支持自动 DPI 缩放与 GDI 级高性能双线性/插值半色调（HALFTONE）图像放大，
/// 当识别区域短边小于 600px 时，自动放大以提高 OCR 识别率。
pub fn ocr_region_sync(x: i32, y: i32, w: i32, h: i32) -> Result<String, String> {
    if w <= 0 || h <= 0 {
        return Err("识别区域的宽度和高度必须大于 0".to_string());
    }

    // 1. 计算缩放因子。如果短边小于 600 像素，自动进行 GDI 硬件级拉伸，最高放大 5 倍
    let short_side = min(w, h);
    let mut scale = 1.0;
    if short_side < 600 {
        scale = 600.0 / short_side as f64;
        if scale > 5.0 {
            scale = 5.0;
        }
    }

    let new_w = (w as f64 * scale) as i32;
    let new_h = (h as f64 * scale) as i32;

    unsafe {
        // 获取桌面屏幕上下文 (GetDC is inside Win32::Graphics::Gdi)
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

        // 创建对应大小的兼容位图
        let h_bitmap = CreateCompatibleBitmap(hdc_screen, new_w, new_h);
        if h_bitmap.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(None, hdc_screen);
            return Err("无法创建兼容的位图 (CreateCompatibleBitmap 失败)".to_string());
        }

        // 选择位图进入内存上下文
        let old_obj = SelectObject(hdc_mem, h_bitmap);

        // 如果需要放大，开启高质量的 HALFTONE 插值拷贝
        let success = if scale > 1.0 {
            SetStretchBltMode(hdc_mem, HALFTONE);
            let _ = SetBrushOrgEx(hdc_mem, 0, 0, None);
            StretchBlt(
                hdc_mem, 0, 0, new_w, new_h,
                hdc_screen, x, y, w, h,
                SRCCOPY
            ).as_bool()
        } else {
            BitBlt(
                hdc_mem, 0, 0, w, h,
                hdc_screen, x, y,
                SRCCOPY
            ).is_ok()
        };

        if !success {
            SelectObject(hdc_mem, old_obj);
            let _ = DeleteObject(h_bitmap);
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(None, hdc_screen);
            return Err("拷贝屏幕像素失败 (BitBlt/StretchBlt 失败)".to_string());
        }

        // 设置 DIB 结构体以读取 BGRA32 格式数据 (biHeight 为负数表示 top-down DIB)
        let mut bitmap_info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: new_w,
                biHeight: -new_h,
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

        let buffer_size = (new_w * new_h * 4) as usize;
        let mut pixel_bytes = vec![0u8; buffer_size];

        let lines_copied = GetDIBits(
            hdc_screen,
            h_bitmap,
            0,
            new_h as u32,
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

        // 2. 将像素数据载入内存 DataWriter，以输出 WinRT 的 IBuffer
        let data_writer = DataWriter::new().map_err(|e| format!("创建 DataWriter 失败: {}", e))?;
        data_writer.WriteBytes(&pixel_bytes).map_err(|e| format!("写入像素字节失败: {}", e))?;
        let ibuffer = data_writer.DetachBuffer().map_err(|e| format!("分离数据缓冲区失败: {}", e))?;

        // 3. 从 IBuffer 载入并创建 SoftwareBitmap
        let software_bitmap = SoftwareBitmap::CreateCopyFromBuffer(
            &ibuffer,
            BitmapPixelFormat::Bgra8,
            new_w,
            new_h
        ).map_err(|e| format!("创建 SoftwareBitmap 失败: {}", e))?;

        // 4. 初始化本地 OcrEngine 进行识别
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

        // 5. 解析识别文本，拼接所有文本行，并过滤空白符
        let mut recognized_text = String::new();
        for line in ocr_result.Lines().map_err(|e| format!("解析 OCR 行失败: {}", e))? {
            let line_text = line.Text().map_err(|e| format!("读取 OCR 文本失败: {}", e))?;
            recognized_text.push_str(&line_text.to_string());
        }

        // 过滤掉所有空格、换行、制表符等空白字符，输出干净统一的文案以方便做正则/ contains 匹配
        let clean_text: String = recognized_text.chars().filter(|c| !c.is_whitespace()).collect();
        Ok(clean_text)
    }
}
