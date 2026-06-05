# AutoController 更新日志

## v0.7.2 — 2026-06-05

### 🆕 新功能与脚本 API

- **全新全局实时遥测脚本 API (`get_telemetry`)**：新增 `get_telemetry()` Rhai 脚本绑定，使用户能在脚本运行时随时获取当前车辆的全局内存实时遥测数据。返回的 Map 对象包含 `car_name` (车辆名)、`speed_kmh` (千米/小时速度)、`current_engine_rpm` (当前转速)、`gear` (挡位)、`is_race_on` (是否在比赛中) 等 16 项关键数据。
- **Monaco 智能联想与 API 快速手册同步**：在脚本编辑器中，为 `get_telemetry()` 注册了专属的 IntelliSense 自动补齐建议和精美 Markdown Hover 悬浮气泡文档；同步在脚本编辑器右侧 API 参考面板中新增「FH6 游戏遥测数据」常用字段速查块。

### 🚀 优化与增强

- **Rhai 脚本 `log` 支持打印复杂结构**：优化了脚本中 `log(...)` 方法的传参逻辑。当传入 Dynamic 类型的 Map 或 Array 时，Rust 后端会自动将其反序列化并格式化为标准 JSON 字符串输出，不再受限于只能打印纯字符串，同时在 Rust 后端依赖中开启了 `rhai` 的 `serde` 特性支持。
- **日志查看面板 UI 体验调优**：拓宽日志 Module 列至 `160px` 以减少常见模块名称的截断，溢出时支持 ellipsis 截断并提供 hover `title` 完整展示；对日志 Message 消息列应用了 `break-all` 和 `whitespace-pre-wrap` 换行样式，避免长文本和带换行符的调试日志撑破布局。

### 🐛 Bug 修复

- **修复脚本编辑器 Monaco 联想、查找、折叠失效 bug**：修复了在特定场景下 Monaco 编辑器搜索查找（Find）、代码折叠（Folding）、括号匹配（Bracket Matching）以及智能联想（IntelliSense Suggest）/ Hover 气泡不生效的 bug。
- **修复 HMR 内存泄漏与重复气泡**：在 `CodeEditor.vue` 销毁及重载时，增加了对 completion 和 hover 注册器 disposable 的主动销毁逻辑，杜绝了前端热更新（HMR）开发时因重复注册导致的联想项/悬浮提示框多重堆叠与内存占用。

---

## v0.7.1 — 2026-06-04

### 🚀 优化与增强

- **Rhai 脚本运行时大文件重构**：将原本庞大的 `runtime.rs` 进行了精细的模块化拆分。分离出虚拟手柄动作 API 绑定模块 `controller_bindings.rs`、OCR 识别 API 绑定模块 `ocr_bindings.rs`、单脚本执行管理模块 `execution.rs` 以及多脚本序列调度模块 `sequence.rs`，极大提升了代码可读性与后期维护效率。
- **引入统一的无锁 `CancellationToken`**：采用基于原子变量 `AtomicBool` 的低延迟、线程安全取消令牌机制，替代了以往每次检测均需获取全局 executions 互斥锁的哈希表查询方案，彻底消除了脚本执行及高频 `sleep` 循环期间的锁竞争开销。
- **遥测数据库集中存储**：优化了 SQLite 遥测数据库的保存路径，由原先的独立 `fh6-tel` 目录迁移至应用配置数据主目录下的 `autocontroller/forza-fh6/sessions.db`，实现了遥测数据与应用核心配置文件的一体化集中保存与备份。

---

## v0.7.0 — 2026-06-04

### 🆕 新功能

#### 相对 v0.6.0：新增全局 UI 系统与 Forza 遥测模块

- **全局 UI 骨架（新增）**：新增 `PageShell`、`PageHeader`、`StatusBanner`、`EmptyState` 等共享布局组件，并为普通、信息、成功、警告、危险状态补齐语义化颜色 token。设备监控、宏控制、脚本编辑器、任务调度、参数配置、通知配置、防失焦与日志查看等核心页面接入统一标题区、状态提示和空状态样式。
- **统一工具栏与折叠侧栏（新增）**：脚本编辑器新增脚本列表折叠 rail 模式，并统一「物理手柄宏录制」「新建」「保存」「运行」等高频操作的按钮尺寸、图标/文案对齐与工具栏节奏，让高密度编辑场景保留更多可用空间。
- **Forza 遥测中心（新增）**：新增独立的 Forza 遥测页面，支持接收游戏 Data Out UDP 数据并展示实时速度、转速、挡位、圈速、姿态、G 值、油门/刹车/转向输入和轮胎状态。
- **遥测历史会话与回放（新增）**：新增遥测会话记录、SQLite 本地存储、圈速统计、会话收藏/重命名/删除、历史数据回放，以及速度、转速、驾驶输入曲线和圈速分析视图。
- **遥测设置与引导（新增）**：新增遥测端口、英里/公里单位、轮胎温度阈值、自动记录等配置入口，并在使用说明中补充 Data Out 开启方式、默认 UDP 端口、连接状态、历史会话与回放路径。

### 🔧 体验与文档调整

- **ViGEmBus 说明更新为内置驱动流程**：将旧版“手动下载安装 ViGEmBus / 复制 DLL”说明调整为当前内置运行组件方案，强调首次启动自动检测、设备监控页热重连与管理员权限排障路径。
- **OCR 使用说明补全**：新增 OCR 使用说明，覆盖识别引擎选择、标定区创建、`ocr()` / `ocr(index)` / `ocr(x, y, w, h)` 三类调用方式。
- **通知与文件导入细节收敛**：全局 toast 关闭按钮调整至右上角，避免关闭控件出现在提示框左上角造成阅读干扰；隐藏配置页导入用原生文件输入控件，只保留统一的「导入」按钮触发文件选择。

### 🐛 Bug 修复

- **Profile 导入控件重复显示**：修复参数配置页 Profile 管理区域中「导入」按钮与原生「选择文件」控件同时显示的问题。
- **提示条关闭按钮位置异常**：修复全局提示条关闭按钮默认出现在左上角、与中文提示阅读路径冲突的问题。
- **按钮图标文字未水平对齐**：修复按钮内图标和文字基线不一致导致顶部工具栏观感跳动的问题。
- **Forza 历史会话抽屉关闭按钮去重**：修复 Forza 遥测页面历史会话右侧抽屉顶部同时出现两个关闭按钮的问题，保留标题栏内的关闭按钮，减少视觉干扰。
- **会话详情弹框默认宽度覆盖问题**：修复底层弹框组件默认 `sm:max-w-sm` 样式导致会话详情弹窗在部分视口下偏窄的问题。

---

## v0.6.0 — 2026-06-01

### 🚀 优化与增强

#### 四大硬核联动：系统级防杀毒排障、代码智能补全、数据统一备份与驱动热自愈

- **Windows Defender 一键信任排除（物理级防隔离）**：针对注入进程（NoFocusLoss）极易被杀软拦截的痛点，新增一键信任排除功能。在管理员权限下，一键静默拉起隐藏的 PowerShell 窗口并执行 `Add-MpPreference -ExclusionPath`，自动将软件运行根目录添加至 Windows Defender 排除项，化解物理查杀困扰。
- **Monaco Rhai 智能提示与 Hover 气泡**：极大打磨了 IDE 级脚本开发体验。扩展注册了 `monaco.languages.registerHoverProvider`，在光标悬浮至 `press`, `release`, `set_thumb`, `set_trigger`, `sleep`, `ocr`, `log` 等核心 API 上时，将弹出以精美 Markdown 排版的参数说明、中文详细介绍与标准示例代码；同时开启了编辑器悬浮气泡原生支持，并为 `ocr()`、`ocr(index)`、`ocr(x,y,w,h)` 提供了高容错自动补齐 Snippet。
- **配置数据统一备份与免权限安全导出（一键备份恢复）**：将本地 `config.json`, `macros.json`, `scripts.json`, `tasks.json` 四大核心数据打包为统一的 JSON 结构，并采用 HTML5 Blob 机制实现在前端触发本地下载，完美绕开复杂的本地磁盘目录安全读写受限屏障。支持使用 `FileReader` 配合后端进行反序列化强制解包，一键对全部自定义脚本、宏动作、定时任务进行百分百安全覆盖导入与状态同步自愈。
- **ViGEmBus 内核驱动多维诊断与一键热重连自愈**：深度精细化处理了手柄底层驱动在应用初始化时的连接状态，当前端捕捉到驱动连接断开（`connected === false`）且 DLL 本地存在时，会自动在监控状态条中显示 **「尝试热重连并激活驱动」** 精致操作按钮。一键点击后静默拉起后端重连，并智能识别之前已处于激活状态的手柄并重新系统级挂载，实现驱动级完美修复，免去重启应用的低效体验！

---

## v0.5.0 — 2026-06-01
### 🚀 优化与增强
#### 虚拟手柄UI重构与 Rust 原生绑定 PaddleOCR 离线集成
- **设备监控页手柄全新视觉微调**：全面重构了设备监控面板中的虚拟手柄交互组件。采用现代化高对比度流线型动效，精准映射手柄各按键的实时输入状态与线性摇杆位移。优化了多设备并存时的排版紧凑度，大幅降低视觉噪音，让硬件状态监控更加直观、丝滑。

- **PaddleOCR Rust 原生绑定与完全本地化内置** ：将 PaddleOCR 核心识别引擎通过 Rust 底层原生绑定（Binding）直接无缝嵌入主程序中，拒绝臃肿的 Python 环境或外部进程调用。配合内置的轻量化预测模型，实现全离线、零网络依赖的图文识别。

内存级超低延迟画面检测：得益于 Rust 与 C++ 动态库的高效互操作性，截图数据无需经过磁盘 I/O 或复杂的序列化传输，直接在内存中完成像素流传递与 OCR 解析。在大幅降低 CPU 与内存开销的同时，为后续的自动化脚本与画面高频状态检测提供了毫秒级的底层性能支撑。


## v0.4.0 — 2026-05-30

### 🆕 新功能

#### 跨进程 DLL 注入与物理隔离防止游戏失焦功能
- **防止游戏失去焦点（No Focus Loss）**：采用底层跨进程 DLL 注入技术，对目标窗口的失去激活消息（如 `WM_ACTIVATEAPP`、`WM_NCACTIVATE`）进行精准欺骗与拦截。**即使切屏至后台、查看网页或多屏协同操作，游戏在后台也能像在前台活动一样持续渲染、保持满帧运行并正常播放声音**，完美适配各类游戏后台挂机。
- **子进程物理隔离高安全注入** (`app/src-tauri/src/system/injector.rs`)：核心注入脏活由完全隔离的独立子进程 `injector.exe` 在后台隐蔽拉起并动态执行（在 Windows 平台启用 `CREATE_NO_WINDOW` 标志强力隐藏命令行黑框），主程序进程不触碰任何敏感的跨进程注入 Win32 API，**彻底杜绝主程序报毒、被安全杀软直接查杀隔离或导致主程序崩溃等严重安全隐患**。
- **Windows 原生管理员权限智能检测**：集成智能检测机制，在用户进入防止失焦界面时，通过绿色/橙黄色主题高对比度提示条展示当前程序的运行权限上下文。针对绝大多数大型 3D 游戏或需要高权限运行的联机平台（Steam/Wegame等），在前端显著引导用户右键选择「以管理员身份运行」本软件，大幅提升注入成功率。
- **一键极简折叠式页面排版**：设计默认单行隐藏的卡片展示，点击 `展开详情` 和 `收起` 按钮辅以平滑过渡动效。在不占用日常屏幕视觉空间的前提下，确保高危安全防封警告与重启使用指南的随手触达。
- **交互式功能指南与避坑指南**：新增手风琴式滑出抽屉面板，系统化归纳挂机前置要素。特别补充了“部分游戏在配置窗口化或无边框窗口化后，必须**重启游戏一次**渲染设置才会生效”的使用踩坑警示。
- **终端脏日志重定向优化**：将普通用户运行 `check_is_admin` 时 Windows 自带的 `net session` 抛出的 `Access is denied` 与 `System error 5` 等安全访问拒绝的控制台错误流重定向至 `Stdio::null()` 废弃，**净化开发及运行日志环境**。

---

## v0.3.0 — 2026-05-29

### 🆕 新功能

#### 智能屏幕 OCR 双引擎系统与配置集成
- **PaddleOCR (HTTP API) 引擎支持**：重构了 OCR 执行底座，新增对外部 PaddleOCR HTTP 接口调用支持。
- **极速内存 PNG 编码转换** (`src-tauri/src/script_engine/ocr.rs`)：深度使用 Windows 原生 WinRT `BitmapEncoder` 和 `InMemoryRandomAccessStream` 实现内存中 `SoftwareBitmap` 到标准 PNG 字节流的高性能转码，实现**零三方依赖、零临时文件读写**，避免因引入庞大三方图像处理库造成打包体积剧增。
- **HTTP Multipart 上传与结果聚合**：引入 `reqwest` 同步 `multipart` 文件表单流分发机制，并对多行识别结果进行高效整合与去空字符过滤，保证不同引擎在 Rhai 脚本层 API 行为的完美一致。
- **OCR 自动化配置面板升级** (`src/views/ConfigPanel.vue`)：前端“OCR 自动化配置”页面重构。新增 OCR 引擎下拉框切换及 PaddleOCR URL 地址输入功能，集成了平滑过渡动效、Pinia 全局状态响应式绑定及秒级后台持久化写入。
- **PaddleOCR Docker 容器部署镜像**：额外为用户提供了高度精炼的 Windows 下 PaddleOCR x86 容器镜像以支持一键部署本地 OCR 服务：
  ```bash
  crpi-a1liy20beodq2bdl.cn-beijing.personal.cr.aliyuncs.com/bujic/win-paddleocr-x86:latest
  ```

#### 多通道智能消息通知系统
- **四种主流通知通道集成**：新增消息通知参数配置板块，深度集成 **飞书 Webhook (Feishu)**、**Server酱 (ServerChan)**、**Server酱三代 (ServerChan3)** 及 **Telegram Bot**。
- 支持在脚本启动、运行成功、异常出错、停止或者任意自定义脚本节点执行高可靠的消息远程分发通知，挂机进度与设备状态完全掌控。

---

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
