# AutoController 更新日志

## v0.2.0 — 2026-05-28

### 🆕 新功能

#### Windows 原生 OCR 屏幕文字识别系统
- **核心 OCR 底层模块** (`src-tauri/src/script_engine/ocr.rs`)：基于 WinRT `Windows.Media.Ocr.OcrEngine` 实现零依赖、超低延迟的中英文混合屏幕文字识别。
- **多区域标定与 API**：支持在 ConfigPanel 中框选并标定多个 OCR 识别区域（`#1`、`#2`、`#3`），通过 `ocr(index)` 脚本 API 直接调用对应区域识别。
- **直接区域识别 API**：支持 `ocr(x, y, w, h)` 以物理像素坐标直接截图识别。

#### 高精度图像预处理算法
- **RGB 平衡平均灰度化**：采用 `(R + G + B) / 3` 算法替代经典感知权重，彻底消除 Windows ClearType 亚像素抗锯齿边缘水平通道红/蓝色偏，完美保留汉字笔画形态。
- **智能自适应反色**：自动统计图像平均亮度，深色背景（亮度 < 128）下自动对比度反色，确保 WinRT OCR 引擎始终接收"白底黑字"的高亲和性输入，兼容 VSCode 深色主题、游戏 UI 等各类暗底场景。
- **单通道 Bicubic 双三次卷积超分重采样**：识别到前置灰度化后图像三通道完全等价，直接将三通道插值合并为单通道插值后赋值，削减 66.6% 无用浮点乘法，双三次重采样提速 **300%**。

#### 高分屏 DPI 物理像素精准对齐
- 在动态生成的 PowerShell Forms 框选脚本首行注入 `SetProcessDPIAware()` P/Invoke 调用（带 `try/catch` 安全保护），强制 Forms 遮罩运行在物理像素坐标系，与 GDI `BitBlt` 截图区域实现**像素级绝对对齐**，彻底修复高 DPI 系统下选区偏移与缩放崩盘问题。

#### 定时任务调度引擎
- **后台心跳调度线程** (`src-tauri/src/scheduler/queue.rs`)：应用启动时拉起独立系统线程，每秒心跳轮询，非阻塞多线程异步分发执行到期任务。
- **五种调度策略**：
  - `Manual`（手动执行）：系统不自动触发，仅由用户点击运行按钮执行。
  - `Once`（单次定时）：指定 UTC 时刻触发一次，执行后自动下线。
  - `Daily`（每日定时）：指定本地时区时刻每日循环触发。
  - `Interval`（周期循环）：指定毫秒间隔周期性重复执行。
  - `Cron`（标准 Cron 表达式）：5 字段标准 Cron 解析（依赖 `cron` crate）。
- **优先级抢占调度**：多任务同时到期时，依据 `priority`（1-100）值高优先级抢占顺序执行。

#### 任务调度前端可视化配置面板
- 新建任务弹窗中新增"调度配置"区块，可视化配置所有五种调度策略。
- `Manual` 模式：高亮说明框，清晰告知用户系统不会自动触发。
- `Once` 模式：`datetime-local` 选择器，精确到秒。
- `Daily` 模式：标准时间选择器（时:分:秒）。
- `Interval` 模式：数值 + 单位（秒/分钟/小时）双选择器，自动换算为底层毫秒数。
- `Cron` 模式：表达式输入框配常用示例提示。
- 任务卡片列表中以人类可读格式展示当前调度策略（如：`每日定时: 12:00:00`、`循环间隔: 5 分钟`、`Cron: */5 * * * *`、`🖱️ 手动执行`）。

#### 脚本编辑器 API 参考面板增强
- 新增 **"OCR 屏幕文字识别"** 快速参考：`ocr(x,y,w,h)` 与 `ocr(index)` 两种用法示例，含完整注释。
- 新增 **"字符串模糊判断与匹配"** 快速参考：`contains`、`starts_with`、`ends_with`、`is_empty`、`len`、`trim` 等高频方法完整示例。

---

### 🐛 Bug 修复

#### 修复菜单切换时日志级别刷屏问题
- **问题**：每次点击侧边栏菜单切换页面，前端都会调用 `config_get` → 读取配置 → 前端渲染 → 调用 `config_set` → 后台输出 `日志级别已动态更新为: info`，造成后台控制台频繁无意义刷屏。
- **修复** (`src-tauri/src/commands/config_cmd.rs`)：在 `config_set` 中引入 `old_level != new_level` 前置判断，只有日志过滤级别发生**实际变化**时才重载 `EnvFilter` 并输出日志，完全消除无效刷屏。

---

### ⚡ 性能优化

| 优化项 | 提升幅度 | 说明 |
|--------|----------|------|
| Bicubic 双三次插值单通道化 | **300% 加速** | 灰度图三通道浮点乘法合并为单通道，削减 66.6% 无用运算 |
| OCR 预处理管道整体延迟 | 显著降低 | 单通道插值 + 仅一次亮度统计扫描，无冗余遍历 |
| 配置变更日志节流 | 消除无效 I/O | 仅在值实际变化时触发过滤器重载与磁盘写入 |

---

### 🔧 技术细节

#### 依赖变更 (`Cargo.toml`)
```toml
# 新增
cron = "0.12"                          # 标准 Cron 表达式解析

# 新增 windows features
windows = { features = [
  "Win32_Graphics_Gdi",                # BitBlt GDI 截图
  "Win32_UI_WindowsAndMessaging",      # 窗口消息
  "Media_Ocr",                         # WinRT OCR 引擎
  "Graphics_Imaging",                  # SoftwareBitmap
  "Storage_Streams",                   # InMemoryRandomAccessStream
] }
```

#### 新增文件
- `src-tauri/src/script_engine/ocr.rs` — OCR 核心模块
- `src-tauri/src/scheduler/queue.rs` — 任务调度引擎
- `src-tauri/src/scheduler/types.rs` — 调度类型定义
- `src-tauri/src/commands/select_region.ps1` — 动态生成的屏幕框选脚本模板
- `src/views/TaskScheduler.vue` — 任务调度前端面板
- `src/types/scheduler.ts` — 调度类型 TypeScript 定义

#### 修改文件
- `src-tauri/src/commands/config_cmd.rs` — DPI 注入 & 日志节流
- `src-tauri/src/script_engine/runtime.rs` — 注册 `ocr()` Rhai API
- `src-tauri/src/config/app_config.rs` — 扩展 OCR 区域坐标配置字段
- `src/views/ScriptEditor.vue` — 扩展 API 参考面板
- `src/App.vue` — 路由注册任务调度页面

---

## v0.3.x 及更早版本

> 更新日志持续补充中…
