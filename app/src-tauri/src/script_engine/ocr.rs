use std::cmp::min;
use std::sync::OnceLock;
use parking_lot::Mutex;
use paddle_ocr_rs::ocr_lite::OcrLite;
use windows::core::HSTRING;
use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;
use windows::Win32::Graphics::Gdi::*;

pub static OCR_ENGINE: OnceLock<Mutex<OcrLite>> = OnceLock::new();

fn strip_unc_prefix(path: std::path::PathBuf) -> std::path::PathBuf {
    use std::path::{Component, Prefix};

    let mut components = path.components();
    match components.next() {
        Some(Component::Prefix(prefix_component)) => {
            match prefix_component.kind() {
                Prefix::VerbatimDisk(disk) => {
                    let mut new_path = std::path::PathBuf::from(format!("{}:", disk as char));
                    new_path.push(components.as_path());
                    new_path
                }
                Prefix::VerbatimUNC(server, share) => {
                    let mut new_path = std::path::PathBuf::from(r"\\");
                    new_path.push(server);
                    new_path.push(share);
                    new_path.push(components.as_path());
                    new_path
                }
                _ => path,
            }
        }
        _ => path,
    }
}

/// 初始化并获取全局缓存的 PaddleOCR 推理引擎实例
pub fn get_or_init_ocr(app_handle: &tauri::AppHandle) -> Result<&Mutex<OcrLite>, String> {
    if let Some(engine) = OCR_ENGINE.get() {
        return Ok(engine);
    }

    use tauri::Manager;
    use tauri::path::BaseDirectory;

    // ★ 关键步骤：在首次初始化前设置 ORT_DYLIB_PATH 环境变量，
    // 使 ort crate 的 load-dynamic 特性能在运行时找到 onnxruntime.dll，
    // 从而完全绕过静态链接（解决 MSVC 14.35 工具链兼容性问题）。
    if std::env::var("ORT_DYLIB_PATH").is_err() {
        // 优先尝试 Tauri 资源路径（打包后生效）
        if let Ok(dll_path) = app_handle.path().resolve("resources/onnxruntime/onnxruntime.dll", BaseDirectory::Resource) {
            let dll_path = strip_unc_prefix(dll_path);
            if dll_path.exists() {
                tracing::info!("设置 ORT_DYLIB_PATH = {:?}", dll_path);
                std::env::set_var("ORT_DYLIB_PATH", &dll_path);
            }
        }
        // 开发模式回退：直接使用项目 resources 目录中的 DLL
        if std::env::var("ORT_DYLIB_PATH").is_err() {
            let dev_dll = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("resources")
                .join("onnxruntime")
                .join("onnxruntime.dll");
            if dev_dll.exists() {
                tracing::info!("(开发模式) 设置 ORT_DYLIB_PATH = {:?}", dev_dll);
                std::env::set_var("ORT_DYLIB_PATH", &dev_dll);
            }
        }
    }

    let mut det_path = strip_unc_prefix(
        app_handle
            .path()
            .resolve("resources/ocr_models/det/det_model.onnx", BaseDirectory::Resource)
            .map_err(|e| format!("无法解析检测模型路径: {}", e))?
    );


    let mut cls_path = strip_unc_prefix(
        app_handle
            .path()
            .resolve("resources/ocr_models/cls/cls_model.onnx", BaseDirectory::Resource)
            .map_err(|e| format!("无法解析分类模型路径: {}", e))?
    );


    let mut rec_path = strip_unc_prefix(
        app_handle
            .path()
            .resolve("resources/ocr_models/rec/rec_model.onnx", BaseDirectory::Resource)
            .map_err(|e| format!("无法解析识别模型路径: {}", e))?
    );


    tracing::info!("正在初始化本地 PaddleOCR V4 引擎...\n检测模型: {:?}\n分类模型: {:?}\n识别模型: {:?}", det_path, cls_path, rec_path);

    if !det_path.exists() || !cls_path.exists() || !rec_path.exists() {
        return Err("OCR 模型文件不存在，请确保模型已部署在 resources/ocr_models 目录中".to_string());
    }

    let mut ocr = OcrLite::new();
    ocr.init_models(
        &det_path.to_string_lossy(),
        &cls_path.to_string_lossy(),
        &rec_path.to_string_lossy(),
        4, // 限制内部并行线程数为 4，杜绝多核 CPU 线程暴涨卡顿
    ).map_err(|e| format!("初始化 PaddleOCR 模型失败: {:?}", e))?;

    let _ = OCR_ENGINE.set(Mutex::new(ocr));

    Ok(OCR_ENGINE.get().unwrap())
}

/// 将截屏 BGRA 像素转换为 RgbImage 并调用本地 PaddleOCR 推理接口
fn call_paddleocr_native(
    pixel_bytes: Vec<u8>,
    w: i32,
    h: i32,
    app_handle: &tauri::AppHandle,
) -> Result<String, String> {
    // 1. 无缝将 BGRA32 字节流提取并转换为 RgbImage，零临时文件 I/O，极其快速
    let mut rgb_bytes = Vec::with_capacity((w * h * 3) as usize);
    for chunk in pixel_bytes.chunks_exact(4) {
        let b = chunk[0];
        let g = chunk[1];
        let r = chunk[2];
        rgb_bytes.push(r);
        rgb_bytes.push(g);
        rgb_bytes.push(b);
    }

    let rgb_img = image::RgbImage::from_raw(w as u32, h as u32, rgb_bytes)
        .ok_or_else(|| "无法从原始字节创建 RgbImage".to_string())?;

    // 2. 线程安全调用全局共享的 OcrLite 推理会话
    let ocr_mutex = get_or_init_ocr(app_handle)?;
    let mut ocr = ocr_mutex.lock();

    // 3. 执行识别推理（angle_classification: false, rotate: false）
    let res = ocr.detect(
        &rgb_img,
        50,    // padding
        1024,  // max_side_len
        0.5,   // box_score_thresh
        0.3,   // box_thresh
        1.6,   // unclip_ratio
        false, // do angle classification (针对常规水平文本，不需要多余的角度纠正)
        false, // do rotate
    ).map_err(|e| format!("PaddleOCR 本地推理失败: {:?}", e))?;

    // 4. 提取和拼接识别出来的所有文本块
    let mut text = String::new();
    for block in res.text_blocks {
        text.push_str(&block.text);
    }

    Ok(text)
}

/// 截取屏幕上的指定矩形区域，并使用指定的 OCR 引擎进行识别。
pub fn ocr_region_sync(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    ocr_engine: &str,
    _paddleocr_url: &str,
    app_handle: Option<&tauri::AppHandle>,
) -> Result<String, String> {
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

    let recognized_text = if ocr_engine == "paddleocr" {
        // 调用本地原生 PaddleOCR 引擎进行识别
        let handle = app_handle.ok_or_else(|| "本地 PaddleOCR 推理需要有效的 AppHandle 传入".to_string())?;
        call_paddleocr_native(pixel_bytes, new_w, new_h, handle)?
    } else {
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

