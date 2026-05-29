use std::cmp::min;
use windows::core::HSTRING;
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap, BitmapEncoder};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::{DataWriter, InMemoryRandomAccessStream, DataReader};
use windows::Win32::Graphics::Gdi::*;

/// 使用 WinRT BitmapEncoder 将 SoftwareBitmap 编码为标准的 PNG 二进制字节流，零额外第三方库依赖。
fn encode_software_bitmap_to_png_bytes(bitmap: &SoftwareBitmap) -> Result<Vec<u8>, String> {
    let stream = InMemoryRandomAccessStream::new()
        .map_err(|e| format!("无法创建内存流: {}", e))?;

    let encoder_guid = BitmapEncoder::PngEncoderId()
        .map_err(|e| format!("无法获取 PngEncoderId: {}", e))?;

    let encoder = BitmapEncoder::CreateAsync(encoder_guid, &stream)
        .map_err(|e| format!("无法创建 Png 编码器: {}", e))?
        .get()
        .map_err(|e| format!("无法初始化 Png 编码器: {}", e))?;

    encoder.SetSoftwareBitmap(bitmap)
        .map_err(|e| format!("无法设置 SoftwareBitmap 到编码器: {}", e))?;

    encoder.FlushAsync()
        .map_err(|e| format!("Flush 编码器失败: {}", e))?
        .get()
        .map_err(|e| format!("获取 Flush 编码器结果失败: {}", e))?;

    let size = stream.Size().map_err(|e| format!("无法获取内存流大小: {}", e))? as u32;
    stream.Seek(0).map_err(|e| format!("内存流 Seek 失败: {}", e))?;

    let reader = DataReader::CreateDataReader(&stream)
        .map_err(|e| format!("无法创建 DataReader: {}", e))?;

    reader.LoadAsync(size)
        .map_err(|e| format!("加载数据流失败: {}", e))?
        .get()
        .map_err(|e| format!("获取加载数据流结果失败: {}", e))?;

    let mut bytes = vec![0u8; size as usize];
    reader.ReadBytes(&mut bytes)
        .map_err(|e| format!("从 DataReader 读取字节失败: {}", e))?;

    Ok(bytes)
}

/// 发送截屏 PNG 图片到外部 PaddleOCR HTTP 接口。
fn call_paddleocr_http(bitmap: &SoftwareBitmap, paddleocr_url: &str) -> Result<String, String> {
    let png_bytes = encode_software_bitmap_to_png_bytes(bitmap)?;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("无法创建 HTTP 客户端: {}", e))?;

    let part = reqwest::blocking::multipart::Part::bytes(png_bytes)
        .file_name("screenshot.png")
        .mime_str("image/png")
        .map_err(|e| format!("创建 multipart Part 失败: {}", e))?;

    let form = reqwest::blocking::multipart::Form::new()
        .part("file", part);

    let resp = client.post(paddleocr_url)
        .multipart(form)
        .send()
        .map_err(|e| format!("发送 PaddleOCR HTTP 请求失败: {}", e))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("PaddleOCR 接口返回 HTTP 状态错误: {}", status));
    }

    #[derive(serde::Deserialize)]
    struct PaddleOcrResultItem {
        text: String,
    }

    #[derive(serde::Deserialize)]
    struct PaddleOcrResponse {
        status: String,
        results: Option<Vec<PaddleOcrResultItem>>,
    }

    let ocr_resp: PaddleOcrResponse = resp.json()
        .map_err(|e| format!("解析 PaddleOCR 响应 JSON 失败: {}", e))?;

    if ocr_resp.status != "success" {
        return Err(format!("PaddleOCR 识别失败，接口返回状态: {}", ocr_resp.status));
    }

    let mut recognized_text = String::new();
    if let Some(items) = ocr_resp.results {
        for item in items {
            recognized_text.push_str(&item.text);
        }
    }

    Ok(recognized_text)
}

/// 截取屏幕上的指定矩形区域，并使用指定的 OCR 引擎（Windows 原生或外部 PaddleOCR）进行识别。
/// 
/// 优化特性 (对齐 AutoDrive_ocr 项目的最小边长 = 600 以及最多 5 倍放大规则)：
/// 1. 原生 GDI 像素级极速屏幕截图；
/// 2. DPI 感知与原始像素处理；
/// 3. 如果识别区域短边小于 600 像素，自动使用 GDI StretchBlt (HALFTONE 插值) 进行高清晰度放大，最高放大 5 倍；
/// 4. 无其他多余的对比度拉伸或锐化滤波，保持图像原始色彩；
/// 5. 过滤所有空白字符，输出干净统一的文案以方便做包含匹配。
pub fn ocr_region_sync(x: i32, y: i32, w: i32, h: i32, ocr_engine: &str, paddleocr_url: &str) -> Result<String, String> {
    if w <= 0 || h <= 0 {
        return Err("识别区域的宽度和高度必须大于 0".to_string());
    }

    // 1. 计算缩放因子。如果短边小于 600 像素，进行高清晰度缩放，最高放大 5 倍
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

    let pixel_bytes = unsafe {
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

        // 创建对应大小的兼容位图
        let h_bitmap = CreateCompatibleBitmap(hdc_screen, new_w, new_h);
        if h_bitmap.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(None, hdc_screen);
            return Err("无法创建兼容的位图 (CreateCompatibleBitmap 失败)".to_string());
        }

        // 选择位图进入内存上下文
        let old_obj = SelectObject(hdc_mem, h_bitmap);

        // 如果需要缩放，采用 GDI HALFTONE 插值进行高品质拷贝
        let success = if scale > 1.0 {
            SetStretchBltMode(hdc_mem, HALFTONE);
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
            return Err("拷贝或缩放屏幕像素失败 (Blt 失败)".to_string());
        }

        // 设置 DIB 结构体以读取 BGRA32 格式 data
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
        let mut bytes = vec![0u8; buffer_size];

        let lines_copied = GetDIBits(
            hdc_screen,
            h_bitmap,
            0,
            new_h as u32,
            Some(bytes.as_mut_ptr() as *mut _),
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
        Ok::<Vec<u8>, String>(bytes)
    }?;

    // 4. 将像素载入内存 DataWriter，以输出 WinRT 的 IBuffer
    let data_writer = DataWriter::new().map_err(|e| format!("创建 DataWriter 失败: {}", e))?;
    data_writer.WriteBytes(&pixel_bytes).map_err(|e| format!("写入像素字节失败: {}", e))?;
    let ibuffer = data_writer.DetachBuffer().map_err(|e| format!("分离数据缓冲区失败: {}", e))?;

    // 5. 从 IBuffer 载入并创建 SoftwareBitmap
    let software_bitmap = SoftwareBitmap::CreateCopyFromBuffer(
        &ibuffer,
        BitmapPixelFormat::Bgra8,
        new_w,
        new_h
    ).map_err(|e| format!("创建 SoftwareBitmap 失败: {}", e))?;

    let recognized_text = if ocr_engine == "paddleocr" {
        // 调用外部 PaddleOCR API
        call_paddleocr_http(&software_bitmap, paddleocr_url)?
    } else {
        // 6. 初始化本地 OcrEngine 进行识别
        let lang = Language::CreateLanguage(&HSTRING::from("zh-Hans-CN"))
            .map_err(|e| format!("创建语言包（zh-Hans-CN）失败: {}", e))?;

        let engine = if OcrEngine::IsLanguageSupported(&lang).unwrap_or(false) {
            OcrEngine::TryCreateFromLanguage(&lang)
                .map_err(|e| format!("利用 zh-Hans-CN 初始化 OcrEngine 失败: {}", e))?
        } else {
            // 回退尝试使用系统用户默认语言
            OcrEngine::TryCreateFromUserProfileLanguages()
                .map_err(|e| format!("无法创建用户默认语言 of OcrEngine: {}", e))?
        };

        // 异步识别并在当前后台执行线程中同步等待结果 (Rhai 执行器运行在 thread 中)
        let ocr_result = engine.RecognizeAsync(&software_bitmap)
            .map_err(|e| format!("发起 OCR 识别任务失败: {}", e))?
            .get()
            .map_err(|e| format!("等待 OCR 结果超时或出错: {}", e))?;

        // 7. 解析识别文本，拼接所有文本行
        let mut text = String::new();
        for line in ocr_result.Lines().map_err(|e| format!("解析 OCR 行失败: {}", e))? {
            let line_text = line.Text().map_err(|e| format!("读取 OCR 文本失败: {}", e))?;
            text.push_str(&line_text.to_string());
        }
        text
    };

    // 过滤掉所有空格、换行、制表符等空白字符，输出干净统一的文案以方便做包含匹配
    let clean_text: String = recognized_text.chars().filter(|c| !c.is_whitespace()).collect();
    Ok(clean_text)
}

