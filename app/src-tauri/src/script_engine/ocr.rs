#[derive(Debug, Clone, serde::Serialize)]
pub struct OcrRegionResult {
    pub text: String,
    pub engine: String,
    pub profile: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub scaled_w: i32,
    pub scaled_h: i32,
    pub scale: f64,
    pub capture_ms: u128,
    pub infer_ms: u128,
    pub total_ms: u128,
    pub inverted: bool,
}

#[cfg(target_os = "windows")]
mod win_impl {
    use super::OcrRegionResult;
    use paddle_ocr_rs::ocr_lite::OcrLite;
    use parking_lot::Mutex;
    use std::cmp::{max, min};
    use std::ffi::c_void;
    use std::sync::OnceLock;
    use std::time::Instant;
    use windows::core::HSTRING;
    use windows::Globalization::Language;
    use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
    use windows::Media::Ocr::OcrEngine;
    use windows::Storage::Streams::DataWriter;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::Graphics::Gdi::*;

    pub static OCR_ENGINE: OnceLock<Mutex<OcrLite>> = OnceLock::new();
    static OCR_INIT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    static WINRT_OCR_ENGINE: OnceLock<OcrEngine> = OnceLock::new();
    static WINRT_OCR_INIT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    #[derive(Clone, Copy)]
    struct OcrRuntimeProfile {
        name: &'static str,
        target_short_side: f64,
        max_scale: f64,
        capture_max_side: f64,
        paddle_padding: u32,
        paddle_max_side_len: u32,
        box_score_thresh: f32,
        box_thresh: f32,
        unclip_ratio: f32,
        do_angle: bool,
    }

    fn runtime_profile(profile: &str) -> OcrRuntimeProfile {
        match profile {
            "fast" => OcrRuntimeProfile {
                name: "fast",
                target_short_side: 480.0,
                max_scale: 4.0,
                capture_max_side: 768.0,
                paddle_padding: 32,
                paddle_max_side_len: 768,
                box_score_thresh: 0.55,
                box_thresh: 0.35,
                unclip_ratio: 1.5,
                do_angle: false,
            },
            "accurate" => OcrRuntimeProfile {
                name: "accurate",
                target_short_side: 760.0,
                max_scale: 6.0,
                capture_max_side: 1280.0,
                paddle_padding: 64,
                paddle_max_side_len: 1280,
                box_score_thresh: 0.35,
                box_thresh: 0.25,
                unclip_ratio: 1.8,
                do_angle: false,
            },
            _ => OcrRuntimeProfile {
                name: "balanced",
                target_short_side: 600.0,
                max_scale: 5.0,
                capture_max_side: 1024.0,
                paddle_padding: 50,
                paddle_max_side_len: 1024,
                box_score_thresh: 0.5,
                box_thresh: 0.3,
                unclip_ratio: 1.6,
                do_angle: false,
            },
        }
    }

    fn strip_unc_prefix(path: std::path::PathBuf) -> std::path::PathBuf {
        use std::path::{Component, Prefix};

        let mut components = path.components();
        match components.next() {
            Some(Component::Prefix(prefix_component)) => match prefix_component.kind() {
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
            },
            _ => path,
        }
    }

    /// 初始化并获取全局缓存的 PaddleOCR 推理引擎实例
    pub fn get_or_init_ocr(app_handle: &tauri::AppHandle) -> Result<&Mutex<OcrLite>, String> {
        if let Some(engine) = OCR_ENGINE.get() {
            return Ok(engine);
        }

        let _init_guard = OCR_INIT_LOCK.get_or_init(|| Mutex::new(())).lock();
        if let Some(engine) = OCR_ENGINE.get() {
            return Ok(engine);
        }

        use tauri::path::BaseDirectory;
        use tauri::Manager;

        // ★ 关键步骤：在首次初始化前设置 ORT_DYLIB_PATH 环境变量，
        // 使 ort crate 的 load-dynamic 特性能在运行时找到 onnxruntime.dll，
        // 从而完全绕过静态链接（解决 MSVC 14.35 工具链兼容性问题）。
        if std::env::var("ORT_DYLIB_PATH").is_err() {
            // 优先尝试 Tauri 资源路径（打包后生效）
            if let Ok(dll_path) = app_handle.path().resolve(
                "resources/onnxruntime/onnxruntime.dll",
                BaseDirectory::Resource,
            ) {
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

        let det_path = strip_unc_prefix(
            app_handle
                .path()
                .resolve(
                    "resources/ocr_models/det/det_model.onnx",
                    BaseDirectory::Resource,
                )
                .map_err(|e| format!("无法解析检测模型路径: {}", e))?,
        );

        let cls_path = strip_unc_prefix(
            app_handle
                .path()
                .resolve(
                    "resources/ocr_models/cls/cls_model.onnx",
                    BaseDirectory::Resource,
                )
                .map_err(|e| format!("无法解析分类模型路径: {}", e))?,
        );

        let rec_path = strip_unc_prefix(
            app_handle
                .path()
                .resolve(
                    "resources/ocr_models/rec/rec_model.onnx",
                    BaseDirectory::Resource,
                )
                .map_err(|e| format!("无法解析识别模型路径: {}", e))?,
        );

        tracing::info!(
            "正在初始化本地 PaddleOCR V4 引擎...\n检测模型: {:?}\n分类模型: {:?}\n识别模型: {:?}",
            det_path,
            cls_path,
            rec_path
        );

        if !det_path.exists() || !cls_path.exists() || !rec_path.exists() {
            return Err(
                "OCR 模型文件不存在，请确保模型已部署在 resources/ocr_models 目录中".to_string(),
            );
        }

        let mut ocr = OcrLite::new();
        ocr.init_models(
            &det_path.to_string_lossy(),
            &cls_path.to_string_lossy(),
            &rec_path.to_string_lossy(),
            4, // 限制内部并行线程数为 4，杜绝多核 CPU 线程暴涨卡顿
        )
        .map_err(|e| format!("初始化 PaddleOCR 模型失败: {:?}", e))?;

        let _ = OCR_ENGINE.set(Mutex::new(ocr));

        Ok(OCR_ENGINE.get().unwrap())
    }

    fn get_or_init_winrt_ocr_engine() -> Result<&'static OcrEngine, String> {
        if let Some(engine) = WINRT_OCR_ENGINE.get() {
            return Ok(engine);
        }

        let _init_guard = WINRT_OCR_INIT_LOCK.get_or_init(|| Mutex::new(())).lock();
        if let Some(engine) = WINRT_OCR_ENGINE.get() {
            return Ok(engine);
        }

        let lang = Language::CreateLanguage(&HSTRING::from("zh-Hans-CN"))
            .map_err(|e| format!("创建语言包（zh-Hans-CN）失败: {}", e))?;

        let engine = if OcrEngine::IsLanguageSupported(&lang).unwrap_or(false) {
            OcrEngine::TryCreateFromLanguage(&lang)
                .map_err(|e| format!("利用 zh-Hans-CN 初始化 OcrEngine 失败: {}", e))?
        } else {
            OcrEngine::TryCreateFromUserProfileLanguages()
                .map_err(|e| format!("无法创建用户默认语言 of OcrEngine: {}", e))?
        };

        let _ = WINRT_OCR_ENGINE.set(engine);
        Ok(WINRT_OCR_ENGINE.get().unwrap())
    }

    pub fn preheat_ocr_engine(
        app_handle: &tauri::AppHandle,
        ocr_engine: &str,
    ) -> Result<(), String> {
        let started_at = Instant::now();
        if ocr_engine == "winocr" {
            let _ = get_or_init_winrt_ocr_engine()?;
        } else {
            let _ = get_or_init_ocr(app_handle)?;
        }

        tracing::info!(
            target: "ocr",
            engine = %ocr_engine,
            total_ms = started_at.elapsed().as_millis(),
            "OCR engine preheated",
        );
        Ok(())
    }

    /// 将截屏 BGRA 像素转换为 RgbImage 并调用本地 PaddleOCR 推理接口
    fn call_paddleocr_native(
        pixel_bytes: &[u8],
        w: i32,
        h: i32,
        profile: &OcrRuntimeProfile,
        app_handle: &tauri::AppHandle,
    ) -> Result<String, String> {
        // 1. 无缝将 BGRA32 字节流提取并转换为 RgbImage，零临时文件 I/O，极其快速
        let rgb_len = (w as usize)
            .checked_mul(h as usize)
            .and_then(|px| px.checked_mul(3))
            .ok_or_else(|| "OCR RGB 像素缓冲区大小溢出".to_string())?;
        let mut rgb_bytes = Vec::with_capacity(rgb_len);
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
        let res = ocr
            .detect(
                &rgb_img,
                profile.paddle_padding,
                profile.paddle_max_side_len,
                profile.box_score_thresh,
                profile.box_thresh,
                profile.unclip_ratio,
                profile.do_angle,
                false,
            )
            .map_err(|e| format!("PaddleOCR 本地推理失败: {:?}", e))?;

        // 4. 提取和拼接识别出来的所有文本块
        let mut text = String::new();
        for block in res.text_blocks {
            text.push_str(&block.text);
        }

        Ok(text)
    }

    fn compute_capture_scale(w: i32, h: i32, profile: &OcrRuntimeProfile) -> f64 {
        let short_side = min(w, h) as f64;
        let long_side = max(w, h) as f64;
        let mut scale = if short_side < profile.target_short_side {
            profile.target_short_side / short_side
        } else {
            1.0
        };

        if scale > profile.max_scale {
            scale = profile.max_scale;
        }

        if long_side * scale > profile.capture_max_side {
            scale = profile.capture_max_side / long_side;
        }

        if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        }
    }

    fn capture_region_bgra(
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        scale: f64,
    ) -> Result<(Vec<u8>, i32, i32), String> {
        let new_w = ((w as f64 * scale).round().max(1.0)) as i32;
        let new_h = ((h as f64 * scale).round().max(1.0)) as i32;

        unsafe {
            let hdc_screen = GetDC(None);
            if hdc_screen.is_invalid() {
                return Err("无法获取屏幕设备上下文 (GetDC 失败)".to_string());
            }

            let hdc_mem = CreateCompatibleDC(hdc_screen);
            if hdc_mem.is_invalid() {
                ReleaseDC(None, hdc_screen);
                return Err("无法创建兼容的内存上下文 (CreateCompatibleDC 失败)".to_string());
            }

            let mut bitmap_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: new_w,
                    biHeight: -new_h,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    biSizeImage: 0,
                    biXPelsPerMeter: 0,
                    biYPelsPerMeter: 0,
                    biClrUsed: 0,
                    biClrImportant: 0,
                },
                bmiColors: [RGBQUAD {
                    rgbBlue: 0,
                    rgbGreen: 0,
                    rgbRed: 0,
                    rgbReserved: 0,
                }],
            };

            let mut dib_bits: *mut c_void = std::ptr::null_mut();
            let h_bitmap = match CreateDIBSection(
                hdc_screen,
                &bitmap_info,
                DIB_RGB_COLORS,
                &mut dib_bits,
                HANDLE::default(),
                0,
            ) {
                Ok(bitmap) => bitmap,
                Err(e) => {
                    let _ = DeleteDC(hdc_mem);
                    ReleaseDC(None, hdc_screen);
                    return Err(format!("无法创建 DIBSection 位图: {}", e));
                }
            };

            if dib_bits.is_null() {
                let _ = DeleteObject(h_bitmap);
                let _ = DeleteDC(hdc_mem);
                ReleaseDC(None, hdc_screen);
                return Err("DIBSection 未返回有效像素缓冲区".to_string());
            }

            let old_obj = SelectObject(hdc_mem, h_bitmap);
            let should_stretch = new_w != w || new_h != h;

            let success = if should_stretch {
                SetStretchBltMode(hdc_mem, HALFTONE);
                StretchBlt(hdc_mem, 0, 0, new_w, new_h, hdc_screen, x, y, w, h, SRCCOPY).as_bool()
            } else {
                BitBlt(hdc_mem, 0, 0, new_w, new_h, hdc_screen, x, y, SRCCOPY).is_ok()
            };

            let result = if success {
                let buffer_size = (new_w as usize)
                    .checked_mul(new_h as usize)
                    .and_then(|px| px.checked_mul(4))
                    .ok_or_else(|| "截图区域过大，像素缓冲区大小溢出".to_string())?;
                let bytes = std::slice::from_raw_parts(dib_bits as *const u8, buffer_size).to_vec();
                Ok((bytes, new_w, new_h))
            } else {
                Err("拷贝或缩放屏幕像素失败 (Blt 失败)".to_string())
            };

            SelectObject(hdc_mem, old_obj);
            let _ = DeleteObject(h_bitmap);
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(None, hdc_screen);

            result
        }
    }

    fn normalize_winrt_pixels(pixel_bytes: &mut [u8]) -> bool {
        if pixel_bytes.is_empty() {
            return false;
        }

        let mut luminance_sum = 0u64;
        let mut count = 0u64;
        for chunk in pixel_bytes.chunks_exact(4) {
            let b = chunk[0] as u64;
            let g = chunk[1] as u64;
            let r = chunk[2] as u64;
            luminance_sum += (r * 299 + g * 587 + b * 114) / 1000;
            count += 1;
        }

        if count == 0 || luminance_sum / count >= 128 {
            return false;
        }

        for chunk in pixel_bytes.chunks_exact_mut(4) {
            chunk[0] = 255u8.saturating_sub(chunk[0]);
            chunk[1] = 255u8.saturating_sub(chunk[1]);
            chunk[2] = 255u8.saturating_sub(chunk[2]);
        }
        true
    }

    /// 截取屏幕上的指定矩形区域，并使用指定的 OCR 引擎进行识别。
    pub fn ocr_region_detailed_sync(
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        ocr_engine: &str,
        ocr_profile: &str,
        _paddleocr_url: &str,
        app_handle: Option<&tauri::AppHandle>,
    ) -> Result<OcrRegionResult, String> {
        let total_started_at = Instant::now();

        if w <= 0 || h <= 0 {
            return Err("识别区域的宽度和高度必须大于 0".to_string());
        }

        let profile = runtime_profile(ocr_profile);
        let scale = compute_capture_scale(w, h, &profile);
        let capture_started_at = Instant::now();
        let (mut pixel_bytes, new_w, new_h) = capture_region_bgra(x, y, w, h, scale)?;
        let capture_ms = capture_started_at.elapsed().as_millis();

        let infer_started_at = Instant::now();
        let mut inverted = false;
        let recognized_text = if ocr_engine == "paddleocr" {
            // 调用本地原生 PaddleOCR 引擎进行识别
            let handle = app_handle
                .ok_or_else(|| "本地 PaddleOCR 推理需要有效的 AppHandle 传入".to_string())?;
            call_paddleocr_native(&pixel_bytes, new_w, new_h, &profile, handle)?
        } else {
            inverted = normalize_winrt_pixels(&mut pixel_bytes);

            // 4. 将像素载入内存 DataWriter，以输出 WinRT 的 IBuffer
            let data_writer =
                DataWriter::new().map_err(|e| format!("创建 DataWriter 失败: {}", e))?;
            data_writer
                .WriteBytes(&pixel_bytes)
                .map_err(|e| format!("写入像素字节失败: {}", e))?;
            let ibuffer = data_writer
                .DetachBuffer()
                .map_err(|e| format!("分离数据缓冲区失败: {}", e))?;

            // 5. 从 IBuffer 载入并创建 SoftwareBitmap
            let software_bitmap = SoftwareBitmap::CreateCopyFromBuffer(
                &ibuffer,
                BitmapPixelFormat::Bgra8,
                new_w,
                new_h,
            )
            .map_err(|e| format!("创建 SoftwareBitmap 失败: {}", e))?;

            // 6. 复用本地 OcrEngine，避免每次识别重复初始化语言与引擎实例
            let engine = get_or_init_winrt_ocr_engine()?;

            // 异步识别并在当前后台执行线程中同步等待结果 (Rhai 执行器运行在 thread 中)
            let ocr_result = engine
                .RecognizeAsync(&software_bitmap)
                .map_err(|e| format!("发起 OCR 识别任务失败: {}", e))?
                .get()
                .map_err(|e| format!("等待 OCR 结果超时或出错: {}", e))?;

            // 7. 解析识别文本，拼接所有文本行
            let mut text = String::new();
            for line in ocr_result
                .Lines()
                .map_err(|e| format!("解析 OCR 行失败: {}", e))?
            {
                let line_text = line
                    .Text()
                    .map_err(|e| format!("读取 OCR 文本失败: {}", e))?;
                text.push_str(&line_text.to_string());
            }
            text
        };
        let infer_ms = infer_started_at.elapsed().as_millis();

        // 过滤掉所有空格、换行、制表符等空白字符，输出干净统一的文案以方便做包含匹配
        let clean_text: String = recognized_text
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        tracing::debug!(
            target: "ocr",
            engine = %ocr_engine,
            x,
            y,
            w,
            h,
            scaled_w = new_w,
            scaled_h = new_h,
            scale,
            capture_ms,
            infer_ms,
            total_ms = total_started_at.elapsed().as_millis(),
            text_len = clean_text.chars().count(),
            "OCR region recognition completed",
        );
        Ok(OcrRegionResult {
            text: clean_text,
            engine: ocr_engine.to_string(),
            profile: profile.name.to_string(),
            x,
            y,
            w,
            h,
            scaled_w: new_w,
            scaled_h: new_h,
            scale,
            capture_ms,
            infer_ms,
            total_ms: total_started_at.elapsed().as_millis(),
            inverted,
        })
    }

    pub fn ocr_region_sync(
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        ocr_engine: &str,
        ocr_profile: &str,
        paddleocr_url: &str,
        app_handle: Option<&tauri::AppHandle>,
    ) -> Result<String, String> {
        ocr_region_detailed_sync(
            x,
            y,
            w,
            h,
            ocr_engine,
            ocr_profile,
            paddleocr_url,
            app_handle,
        )
        .map(|result| result.text)
    }
}

#[cfg(target_os = "windows")]
pub use win_impl::{ocr_region_detailed_sync, ocr_region_sync, preheat_ocr_engine};

#[cfg(not(target_os = "windows"))]
pub fn ocr_region_sync(
    _x: i32,
    _y: i32,
    _w: i32,
    _h: i32,
    _ocr_engine: &str,
    _ocr_profile: &str,
    _paddleocr_url: &str,
    _app_handle: Option<&tauri::AppHandle>,
) -> Result<String, String> {
    Err("OCR is only supported on Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn ocr_region_detailed_sync(
    _x: i32,
    _y: i32,
    _w: i32,
    _h: i32,
    _ocr_engine: &str,
    _ocr_profile: &str,
    _paddleocr_url: &str,
    _app_handle: Option<&tauri::AppHandle>,
) -> Result<OcrRegionResult, String> {
    Err("OCR is only supported on Windows".to_string())
}

#[cfg(not(target_os = "windows"))]
pub fn preheat_ocr_engine(_app_handle: &tauri::AppHandle, _ocr_engine: &str) -> Result<(), String> {
    Err("OCR is only supported on Windows".to_string())
}
