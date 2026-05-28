use crate::config::{AppConfig, AppConfigManager};

#[tauri::command]
pub fn config_get(manager: tauri::State<'_, AppConfigManager>) -> AppConfig {
    manager.get()
}

#[tauri::command]
pub fn config_set(
    manager: tauri::State<'_, AppConfigManager>,
    reload_handle: tauri::State<'_, tracing_subscriber::reload::Handle<tracing_subscriber::EnvFilter, tracing_subscriber::Registry>>,
    new_config: AppConfig,
) -> Result<(), String> {
    let old_level = manager.get().log_level.clone();
    let new_level = new_config.log_level.clone();
    manager.set(new_config);

    // 仅当日志级别实际发生变化时，才进行重载与日志记录，杜绝页面切换/点击菜单时的刷屏现象
    if old_level != new_level {
        if let Ok(new_filter) = tracing_subscriber::EnvFilter::try_new(format!("{},tao=error", new_level)) {
            if let Err(e) = reload_handle.reload(new_filter) {
                tracing::warn!(error = %e, "动态更新日志级别过滤失败");
            } else {
                tracing::info!("日志级别已动态更新为: {}", new_level);
            }
        }
    }

    Ok(())
}

#[derive(Clone, serde::Serialize)]
pub struct OcrRegionSavedPayload {
    pub index: usize,
    pub region: crate::config::OcrRegion,
    pub regions: Vec<crate::config::OcrRegion>,
}

#[tauri::command]
pub fn open_ocr_viewfinder(
    app_handle: tauri::AppHandle,
    manager: tauri::State<'_, AppConfigManager>,
    index: Option<usize>,
) -> Result<(), String> {
    use std::process::Command;
    use std::os::windows::process::CommandExt;
    use tauri::Emitter;

    // 计算当前是要操作的 1-based 序号 (从 #1 开始)
    let u_idx = match index {
        Some(idx) => {
            if idx == 0 {
                1
            } else {
                idx
            }
        }
        None => {
            // 如果未指定序号，则默认追加到当前标定区域列表的末尾 (即 len + 1)
            manager.get().ocr_regions.len() + 1
        }
    };

    // 将 1-based 序号动态拼接到提示语中，使用户感知极为精细清晰
    let title_text = format!("🎯 标定 OCR 识别区域 #{} (鼠标左键拖拽框选，按 ESC 取消)", u_idx);

    // 核心安全：将完整的 Windows Forms 极光绿框选脚本作为静态字符串嵌入 Rust
    let script_content = format!(r#"
# select_region.ps1

# 强制开启 DPI 感知以获取物理 1:1 坐标，彻底杜绝高分屏缩放下的定位坐标偏移问题
try {{
    Add-Type -TypeDefinition @"
    using System;
    using System.Runtime.InteropServices;
    public class DpiUtil {{
        [DllImport("user32.dll")]
        public static extern bool SetProcessDPIAware();
    }}
"@
    [DpiUtil]::SetProcessDPIAware()
}} catch {{}}

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
$bounds = [System.Windows.Forms.SystemInformation]::VirtualScreen
$form = New-Object System.Windows.Forms.Form
$form.Text = 'AutoController Region Selector'
$form.FormBorderStyle = 'None'
$form.StartPosition = 'Manual'
$form.Location = New-Object System.Drawing.Point $bounds.Left, $bounds.Top
$form.Size = New-Object System.Drawing.Size $bounds.Width, $bounds.Height
$form.TopMost = $true
$form.Opacity = 0.35
$form.BackColor = 'Black'
$form.Cursor = [System.Windows.Forms.Cursors]::Cross
$form.KeyPreview = $true
$form.ShowInTaskbar = $false
$form.GetType().GetMethod('SetStyle', [System.Reflection.BindingFlags]::NonPublic -bor [System.Reflection.BindingFlags]::Instance).Invoke($form, @([System.Windows.Forms.ControlStyles]::OptimizedDoubleBuffer -bor [System.Windows.Forms.ControlStyles]::AllPaintingInWmPaint -bor [System.Windows.Forms.ControlStyles]::UserPaint, $true))
$tip = New-Object System.Windows.Forms.Label
$tip.AutoSize = $true
$tip.ForeColor = [System.Drawing.Color]::FromArgb(34, 197, 94)
$tip.BackColor = [System.Drawing.Color]::Transparent
$tip.Font = New-Object System.Drawing.Font 'Microsoft YaHei', 20, ([System.Drawing.FontStyle]::Bold)
$tip.Text = "{}"
$tip.Location = New-Object System.Drawing.Point 80, 80
$form.Controls.Add($tip)
$liveCoord = New-Object System.Windows.Forms.Label
$liveCoord.AutoSize = $true
$liveCoord.ForeColor = [System.Drawing.Color]::Yellow
$liveCoord.BackColor = [System.Drawing.Color]::Transparent
$liveCoord.Font = New-Object System.Drawing.Font 'Consolas', 18
$liveCoord.Location = New-Object System.Drawing.Point 80, 140
$form.Controls.Add($liveCoord)
$script:startPt = $null
$script:rect = New-Object System.Drawing.Rectangle 0, 0, 0, 0
$script:result = $null
$form.add_MouseDown({{
    param($s, $e)
    if ($e.Button -eq 'Left') {{
        $script:startPt = $e.Location
        $script:rect = New-Object System.Drawing.Rectangle $e.X, $e.Y, 0, 0
    }}
}})
$form.add_MouseMove({{
    param($s, $e)
    if ($script:startPt) {{
        $x = [Math]::Min($script:startPt.X, $e.X)
        $y = [Math]::Min($script:startPt.Y, $e.Y)
        $w = [Math]::Abs($e.X - $script:startPt.X)
        $h = [Math]::Abs($e.Y - $script:startPt.Y)
        $script:rect = New-Object System.Drawing.Rectangle $x, $y, $w, $h
        $screenX = $bounds.Left + $x
        $screenY = $bounds.Top + $y
        $liveCoord.Text = "起始坐标: ($screenX, $screenY)  宽: $w px  高: $h px"
        $form.Invalidate()
    }}
}})
$form.add_MouseUp({{
    param($s, $e)
    if ($script:startPt -and $script:rect.Width -gt 10 -and $script:rect.Height -gt 10) {{
        $script:result = $script:rect
        $form.Close()
    }}
    $script:startPt = $null
}})
$form.add_Paint({{
    param($s, $e)
    if ($script:rect.Width -gt 0 -and $script:rect.Height -gt 0) {{
        $pen = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(34, 197, 94)), 3
        $pen.DashStyle = [System.Drawing.Drawing2D.DashStyle]::Dash
        $e.Graphics.DrawRectangle($pen, $script:rect)
        $brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(30, 34, 197, 94))
        $e.Graphics.FillRectangle($brush, $script:rect)
        $pen.Dispose(); $brush.Dispose()
    }}
}})
$form.add_KeyDown({{
    param($s, $e)
    if ($e.KeyCode -eq 'Escape') {{ $script:result = $null; $form.Close() }}
}})
[void]$form.ShowDialog()
$form.Dispose()
if ($script:result) {{
    $x = $bounds.Left + $script:result.X
    $y = $bounds.Top + $script:result.Y
    Write-Output "RESULT:$x,$y,$($script:result.Width),$($script:result.Height)"
}} else {{
    Write-Output "CANCELLED"
}}
"#, title_text);

    // 获取 AppConfigManager 克隆以传递到后台进程
    let config_manager = manager.inner().clone();
    
    // 在后台线程异步启动，不阻塞 Tauri 渲染主线程，确保极佳的用户体验与极速响应
    std::thread::spawn(move || {
        let temp_dir = std::env::temp_dir();
        let script_path = temp_dir.join("autocontroller_select_region.ps1");
        
        if let Err(e) = std::fs::write(&script_path, script_content) {
            tracing::error!(error = %e, "写入临时标定脚本失败");
            return;
        }

        // 启动 PowerShell 执行 native Forms 标定
        let mut cmd = Command::new("powershell");
        cmd.arg("-NoProfile")
           .arg("-ExecutionPolicy")
           .arg("Bypass")
           .arg("-File")
           .arg(&script_path);

        // 在 Windows 平台下强力隐藏后台终端小黑框
        #[cfg(windows)]
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW

        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                tracing::error!(error = %e, "运行原生标定工具进程失败");
                let _ = std::fs::remove_file(&script_path);
                return;
            }
        };

        // 无论如何，清理临时脚本文件以保持操作系统干净
        let _ = std::fs::remove_file(&script_path);

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("RESULT:") {
                let parts: Vec<&str> = line.trim_start_matches("RESULT:").split(',').collect();
                if parts.len() == 4 {
                    if let (Ok(x), Ok(y), Ok(w), Ok(h)) = (
                        parts[0].parse::<i32>(),
                        parts[1].parse::<i32>(),
                        parts[2].parse::<i32>(),
                        parts[3].parse::<i32>(),
                    ) {
                        tracing::info!("原生标定 #{} 成功，收到坐标: x={}, y={}, w={}, h={}", u_idx, x, y, w, h);
                        
                        // 1. 同步保存配置到全局 TOML 配置文件
                        let mut config = config_manager.get();
                        let vec_idx = u_idx - 1;
                        let new_region = crate::config::OcrRegion { x, y, w, h };

                        if vec_idx < config.ocr_regions.len() {
                            // 重新标定已有区域
                            config.ocr_regions[vec_idx] = new_region;
                        } else {
                            // 扩展并填充直到 vec_idx
                            while config.ocr_regions.len() < vec_idx {
                                // 填充占位符区域，保证数组连续性
                                config.ocr_regions.push(crate::config::OcrRegion { x: 0, y: 0, w: 0, h: 0 });
                            }
                            config.ocr_regions.push(new_region);
                        }

                        // 顺便把老字段 ocr_region 同步更新为首个区域，保证完美向前兼容！
                        if vec_idx == 0 {
                            config.ocr_region = Some(crate::config::OcrRegion { x, y, w, h });
                        }

                        config_manager.set(config.clone());
                        tracing::info!("标定坐标 #{} 已持久化保存", u_idx);

                        // 2. 发送全局事件通知前端卡片刷新与 Toast 浮现
                        let region_payload = OcrRegionSavedPayload {
                            index: u_idx,
                            region: crate::config::OcrRegion { x, y, w, h },
                            regions: config.ocr_regions.clone(),
                        };
                        if let Err(e) = app_handle.emit("ocr-region-saved", region_payload) {
                            tracing::error!(error = %e, "发送 ocr-region-saved 事件失败");
                        } else {
                            tracing::info!("ocr-region-saved 事件已成功广播，更新序号 #{}", u_idx);
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

#[tauri::command]
pub fn save_ocr_region(
    app_handle: tauri::AppHandle,
    manager: tauri::State<'_, AppConfigManager>,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> Result<(), String> {
    use tauri::Emitter;
    
    // 1. 读取、更新并持久化本地 TOML 配置
    let mut config = manager.get();
    let region = crate::config::OcrRegion { x, y, w, h };
    config.ocr_region = Some(region.clone());
    manager.set(config);
    
    // 2. 全局广播 'ocr-region-saved' 事件，让主窗口能响应式刷新其 UI 状态与坐标展示
    let _ = app_handle.emit("ocr-region-saved", region);
    
    Ok(())
}
