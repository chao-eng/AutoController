# AutoController - 高性能游戏挂机工具 设计文档

> 版本: 1.0 | 日期: 2026-05-25 | 状态: 初始设计

---

## 一、项目概述

### 1.1 项目名称
AutoController - 专业级游戏手柄模拟挂机工具

### 1.2 项目定位
基于 Tauri 2.0 + Vue 3 + ViGEmBus 驱动的高性能 Windows 游戏挂机工具，提供专业级手柄输入模拟、宏录制回放、脚本编程和任务调度能力。

### 1.3 目标用户
- 需要长时间挂机的游戏玩家
- 游戏测试与自动化开发人员
- 需要批量手柄模拟的测试场景

### 1.4 核心价值
- 低延迟（<5ms）、高精度（100Hz+）的手柄输入模拟
- 直观的可视化操作界面，零代码门槛
- 强大的脚本编程能力，满足复杂自动化需求
- 轻量高效，CPU<5%，内存<100MB

---

## 二、技术架构

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri 2.0 Application                 │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │              Vue 3 Frontend (WebView)              │  │
│  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌────────┐  │  │
│  │  │设备  │ │宏控制│ │脚本  │ │参数  │ │任务    │  │  │
│  │  │监控  │ │中心  │ │编辑器│ │配置  │ │调度    │  │  │
│  │  └──────┘ └──────┘ └──────┘ └──────┘ └────────┘  │  │
│  └──────────────────────┬─────────────────────────────┘  │
│                         │ Tauri IPC                      │
│  ┌──────────────────────┴─────────────────────────────┐  │
│  │              Rust Backend (Core)                    │  │
│  │  ┌──────────┐ ┌──────────┐ ┌───────────────────┐  │  │
│  │  │ViGEmBus  │ │宏引擎    │ │脚本引擎(V8/QuickJS)│  │  │
│  │  │手柄管理  │ │录制/回放 │ │JS/TS执行         │  │  │
│  │  └──────────┘ └──────────┘ └───────────────────┘  │  │
│  │  ┌──────────┐ ┌──────────┐ ┌───────────────────┐  │  │
│  │  │任务调度器│ │日志系统  │ │配置/Profile管理   │  │  │
│  │  └──────────┘ └──────────┘ └───────────────────┘  │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  ┌────────────────────────────────────────────────────┐  │
│  │              System Integration                     │  │
│  │  系统托盘 │ 进程监控 │ 自启动 │ 更新服务          │  │
│  └────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────┐
│   ViGEmBus Driver   │
│  (Kernel Level)     │
│  Xbox 360           │
└─────────────────────┘
```

### 2.2 技术栈选型

| 层级 | 技术 | 版本 | 用途 |
|------|------|------|------|
| 桌面框架 | Tauri | 2.x | 应用壳、IPC、系统集成 |
| 前端框架 | Vue 3 | 3.4+ | 响应式UI |
| 前端语言 | TypeScript | 5.x | 类型安全 |
| 构建工具 | Vite | 5.x | 快速构建 |
| 状态管理 | Pinia | 2.x | 全局状态 |
| UI框架 | 自定义组件 | - | 基于设计系统 |
| 图标 | Lucide Vue | - | SVG图标 |
| 后端语言 | Rust | 1.95+ | 核心逻辑 |
| 手柄驱动 | ViGEmBus | 1.17+ | 虚拟手柄 |
| 脚本引擎 | QuickJS | - | JS/TS脚本执行 |
| 序列化 | serde | 1.x | Rust数据序列化 |
| 日志 | tracing | 0.1.x | 结构化日志 |

### 2.3 多线程架构

```
Main Thread (UI)
  ├── WebView渲染线程 (Tauri管理)
  └── IPC消息分发

Dedicated Threads:
  ├── Input Simulation Thread (手柄输入模拟, 100Hz循环)
  │     └── ViGEmBus设备通信
  ├── Macro Engine Thread (宏录制/回放)
  │     └── 时间戳精确调度
  ├── Script Engine Thread (脚本执行)
  │     └── QuickJS运行时
  ├── Task Scheduler Thread (任务调度)
  │     └── 定时器/日历触发
  └── Log Writer Thread (异步日志写入)
        └── 文件/缓冲区写入
```

线程间通信使用 `tokio::sync::mpsc` 和 `std::sync::Arc<Mutex<>>` 模式，确保无锁读取和高效写入。

---

## 三、模块详细设计

### 3.1 手柄模拟系统

#### 3.1.1 设备抽象层

```rust
// 核心枚举定义
// 移除了 ControllerType，仅支持 Xbox 360

struct ControllerState {
    buttons: u16,           // 按键位图
    left_thumb_x: i16,      // 左摇杆X轴 (-32768 ~ 32767)
    left_thumb_y: i16,      // 左摇杆Y轴
    right_thumb_x: i16,     // 右摇杆X轴
    right_thumb_y: i16,     // 右摇杆Y轴
    left_trigger: u8,       // 左扳机 (0~255)
    right_trigger: u8,      // 右扳机 (0~255)
}

// 按键映射
enum Button {
    A, B, X, Y,
    LB, RB, LT, RT,
    Back, Start, Guide,
    LeftThumb, RightThumb,
    DPadUp, DPadDown, DPadLeft, DPadRight,
}
```

#### 3.1.2 ViGEmBus集成

- 使用 `vigem-client` Rust crate 与 ViGEmBus 驱动通信
- 每个虚拟设备通过 `VigemClient::create()` 创建
- 输入报告通过 `X360Report` 提交
- 100Hz输入循环：10ms间隔定时提交报告
- 延迟优化：使用 `spin_loop` 提示和精确定时器

#### 3.1.3 多设备管理

- 最多支持8个虚拟手柄同时运行
- 每个设备独立线程，互不干扰
- 设备状态通过共享内存 + 读写锁同步
- 设备热插拔：运行时动态创建/销毁

### 3.2 宏录制与回放系统

#### 3.2.1 数据结构

```rust
struct MacroEvent {
    timestamp_ms: u64,          // 相对于录制开始的毫秒时间戳
    device_id: u8,              // 设备ID
    event_type: MacroEventType, // 事件类型
}

enum MacroEventType {
    ButtonPress(Button),
    ButtonRelease(Button),
    ThumbMove(ThumbAxis, f32),  // 0.0 ~ 1.0
    TriggerMove(TriggerSide, f32), // 0.0 ~ 1.0
}

struct Macro {
    id: String,
    name: String,
    created_at: DateTime<Utc>,
    total_duration_ms: u64,
    events: Vec<MacroEvent>,
}
```

#### 3.2.2 录制功能

- 通过全局键盘钩子（可选快捷键）触发录制开始/停止
- 录制期间捕获所有虚拟手柄状态变更
- 毫秒级精度时间戳
- 支持无限时长录制（事件流式写入磁盘）
- 录制过程中实时显示波形预览

#### 3.2.3 回放功能

- 速度调节：50%~200%，通过时间戳缩放实现
- 循环次数：1~无限次
- 断点续播：记录回放位置，支持暂停后从断点继续
- 回放精度：基于高精度定时器 `quanta::Instant`

### 3.3 脚本引擎

#### 3.3.1 引擎选型

采用 QuickJS 嵌入式引擎：
- 轻量级，内存占用低
- 支持ES2020语法
- 可通过 Rust FFI 绑定自定义API
- 编译为 WASM 可选，增强安全性

#### 3.3.2 脚本API设计

```typescript
// 手柄控制API
interface ControllerAPI {
  press(deviceId: number, button: Button): void;
  release(deviceId: number, button: Button): void;
  setThumb(deviceId: number, thumb: ThumbAxis, value: number): void;
  setTrigger(deviceId: number, trigger: TriggerSide, value: number): void;
  getState(deviceId: number): ControllerState;
}

// 延时与等待
interface TimingAPI {
  sleep(ms: number): Promise<void>;
  waitUntil(condition: () => boolean, timeout?: number): Promise<boolean>;
}

// 游戏事件监听
interface EventAPI {
  onGameStart(callback: () => void): void;
  onGameExit(callback: () => void): void;
  onProcessChange(callback: (pid: number) => void): void;
}

// 日志API
interface LogAPI {
  info(msg: string): void;
  warn(msg: string): void;
  error(msg: string): void;
}
```

#### 3.3.3 脚本编辑器

- 基于 Monaco Editor（VS Code同款编辑器）
- 语法高亮：JavaScript/TypeScript
- 代码补全：自定义API提示
- 实时错误检测
- 代码片段库：常用操作模板
- 调试支持：断点、单步执行、变量查看

### 3.4 任务调度系统

#### 3.4.1 调度类型

```rust
enum ScheduleType {
    Once(DateTime<Utc>),                    // 单次执行
    Daily { time: NaiveTime },              // 每日定时
    Interval { duration: Duration },        // 间隔执行
    Cron { expression: String },            // Cron表达式
}

struct ScheduledTask {
    id: String,
    name: String,
    schedule: ScheduleType,
    action: TaskAction,
    priority: u8,           // 0=最低, 255=最高
    enabled: bool,
    last_run: Option<DateTime<Utc>>,
    next_run: Option<DateTime<Utc>>,
}
```

#### 3.4.2 冲突解决

- 同一设备同一时刻只执行一个任务
- 高优先级任务可抢占低优先级任务
- 被抢占任务自动进入等待队列
- 任务依赖：支持任务间的前置条件检查

### 3.5 系统集成

#### 3.5.1 系统托盘

- 最小化到托盘运行
- 托盘菜单：显示/隐藏、暂停/恢复、退出
- 托盘图标状态指示（运行中/暂停/错误）
- 气泡通知：任务完成、错误告警

#### 3.5.2 进程监控

- 监控目标游戏进程是否运行
- 游戏启动时自动切换到高性能模式
- 游戏退出时自动降低采样率
- 支持配置多个游戏进程名

### 3.6 日志系统

#### 3.6.1 日志分级

| 级别 | 用途 | 示例 |
|------|------|------|
| TRACE | 详细调试 | 输入报告内容 |
| DEBUG | 调试信息 | 设备状态变更 |
| INFO | 关键操作 | 任务启动/完成 |
| WARN | 警告 | 性能降级 |
| ERROR | 错误 | 设备连接失败 |

#### 3.6.2 日志存储

- 异步写入，不阻塞主线程
- 日志轮转：单文件最大10MB，保留最近20个文件
- 支持导出为 JSON / CSV 格式
- 日志搜索与过滤

### 3.7 配置与Profile管理

#### 3.7.1 数据模型

```rust
struct GameProfile {
    id: String,
    name: String,
    game_process: String,           // 游戏进程名
    controller_config: ControllerConfig,
    macros: Vec<Macro>,
    scripts: Vec<Script>,
    schedules: Vec<ScheduledTask>,
}

struct AppConfig {
    devices: Vec<DeviceConfig>,      // 最多8个
    profiles: Vec<GameProfile>,
    active_profile: Option<String>,
    auto_start: bool,
    minimize_to_tray: bool,
    theme: Theme,
    log_level: LogLevel,
}
```

#### 3.7.2 存储方案

- 配置文件存储在 `%APPDATA%/AutoController/` 目录
- 使用 TOML 格式存储配置（Rust生态友好）
- Profile 文件独立存储，支持导入/导出
- 自动备份：每次修改前自动备份上一版本

---

## 四、UI设计规范

### 4.1 设计系统

基于 ui-ux-pro-max 生成的 AutoController 设计系统：

| 属性 | 值 |
|------|-----|
| 风格 | Vibrant & Block-based (工业极简) |
| 主色 | #0F172A (深蓝黑) |
| 辅色 | #1E293B (暗灰蓝) |
| 强调色 | #22C55E (活力绿) |
| 背景色 | #020617 (极深蓝黑) |
| 文字色 | #F8FAFC (近白) |
| 标题字体 | Fira Code |
| 正文字体 | Fira Sans |

### 4.2 布局设计

参考飞书应用的简洁高效设计理念，采用左侧导航 + 主内容区布局：

```
┌──────────────────────────────────────────────────────────┐
│  ◉ AutoController                            ─  □  ✕     │
├────┬─────────────────────────────────────────────────────┤
│    │                                                     │
│ 📊 │   [设备状态监控面板]                                  │
│ 设备│                                                     │
│    │   ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐             │
│ 🎮 │   │设备1 │ │设备2 │ │设备3 │ │ + 添加│             │
│ 宏  │   │Xbox  │ │Xbox  │ │空闲  │ │      │             │
│    │   │● 运行 │ │● 录制│ │○ 离线│ │      │             │
│ 📝 │   └──────┘ └──────┘ └──────┘ └──────┘             │
│ 脚本│                                                     │
│    │   [摇杆可视化]          [扳机可视化]                   │
│ ⚙️ │   ┌─────────┐          ┌──┐ ┌──┐                   │
│ 配置│   │  ╱  ·  ╲ │          │▓▓│ │▓ │                   │
│    │   │ ·      · │          │▓▓│ │  │                   │
│ 📅 │   │  ╲    ╱  │          │▓▓│ │  │                   │
│ 任务│   └─────────┘          └──┘ └──┘                   │
│    │   L-Stick               L-Trig R-Trig               │
│ 📋 │                                                     │
│ 日志│                                                     │
├────┴─────────────────────────────────────────────────────┤
│  ● 3设备运行中  │  CPU: 2.1%  │  MEM: 45MB  │  v1.0.0    │
└──────────────────────────────────────────────────────────┘
```

### 4.3 核心页面

| 页面 | 功能 | 关键组件 |
|------|------|----------|
| 设备监控 | 虚拟手柄状态、摇杆/扳机可视化 | DeviceCard, StickVisualizer, TriggerBar |
| 宏控制 | 录制/回放/编辑宏 | MacroTimeline, PlaybackControls, SpeedSlider |
| 脚本编辑 | 编写/调试脚本 | MonacoEditor, ApiReference, DebugPanel |
| 参数配置 | 设备参数、全局设置 | ConfigForm, ProfileSelector, ImportExport |
| 任务调度 | 定时任务管理 | TaskList, ScheduleEditor, CalendarView |
| 日志查看 | 操作日志浏览/导出 | LogTable, LogFilter, ExportButton |

### 4.4 组件规范

- 所有图标使用 Lucide Vue SVG 图标
- 所有可点击元素添加 `cursor-pointer`
- 悬停状态使用 150-300ms 过渡动画
- 深色模式为默认主题
- 遵循 `prefers-reduced-motion` 无障碍设置

---

## 五、数据流设计

### 5.1 输入模拟数据流

```
用户操作 (UI)
    │
    ▼
Vue组件 → Pinia Store → Tauri IPC → Rust Command Handler
    │
    ▼
Input Simulation Thread
    │
    ▼ (100Hz循环)
ViGEmBus Driver → 虚拟手柄设备 → 游戏
```

### 5.2 宏录制数据流

```
用户按键 (快捷键/界面)
    │
    ▼
Rust全局钩子 → Macro Engine Thread
    │
    ▼
事件流 → 内存缓冲区 → 磁盘持久化
    │
    ▼
Tauri IPC → Vue实时波形显示
```

### 5.3 脚本执行数据流

```
用户编写脚本 (Monaco Editor)
    │
    ▼
Tauri IPC → Script Engine Thread
    │
    ▼
QuickJS执行 → 调用手柄API → Input Simulation Thread
    │
    ▼
日志/状态 → Tauri Event → Vue状态更新
```

---

## 六、性能优化策略

### 6.1 输入延迟优化

- 输入模拟线程使用实时优先级
- 自旋等待替代睡眠，确保精确10ms周期
- ViGEmBus报告提交使用零拷贝
- IPC通信使用二进制序列化（bincode）

### 6.2 内存优化

- 宏事件使用内存映射文件，避免大数组占用堆内存
- 脚本引擎预编译，避免运行时编译开销
- 日志使用环形缓冲区，限制内存占用
- Vue组件懒加载，减少初始内存

### 6.3 CPU优化

- 游戏未运行时采样率降至10Hz
- UI更新频率限制为30fps
- 脚本执行使用工作窃取线程池
- 日志写入完全异步

---

## 七、安全与稳定性

### 7.1 防崩溃机制

- 每个虚拟设备运行在独立线程，单个设备崩溃不影响其他
- 脚本引擎沙箱隔离，脚本错误不会导致程序崩溃
- 全局 panic hook 捕获未处理异常
- Watchdog线程监控关键线程存活状态

### 7.2 状态恢复

- 定期（每30秒）将运行状态快照保存到磁盘
- 程序启动时检测未完成的快照，提示恢复
- 恢复内容包括：设备状态、运行中的任务、脚本执行位置

### 7.3 完整性校验

- 应用程序二进制签名验证
- 配置文件哈希校验
- 脚本执行前语法和API白名单检查

---

## 八、安装与更新

### 8.1 安装程序

- 使用 NSIS 或 WiX 制作 Windows 安装包
- 安装流程：许可协议 → 选择路径 → 检测/安装ViGEmBus → 完成
- ViGEmBus驱动自动检测与安装提示
- 支持静默安装参数

### 8.2 更新机制

- Tauri内置更新器（tauri-plugin-updater）
- 增量更新：使用 bsdiff 算法生成差分包
- 更新检查：启动时自动检查 + 手动检查
- 更新下载后台进行，完成后提示重启

---

## 九、项目目录结构

```
autocontroller/
├── src-tauri/                    # Rust后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── lib.rs                 # 库入口
│   │   ├── controller/            # 手柄模拟模块
│   │   │   ├── mod.rs
│   │   │   ├── vigem.rs           # ViGEmBus集成
│   │   │   ├── state.rs           # 设备状态管理
│   │   │   └── types.rs           # 类型定义
│   │   ├── macro_engine/          # 宏引擎模块
│   │   │   ├── mod.rs
│   │   │   ├── recorder.rs        # 录制器
│   │   │   ├── player.rs          # 回放器
│   │   │   └── types.rs
│   │   ├── script_engine/         # 脚本引擎模块
│   │   │   ├── mod.rs
│   │   │   ├── runtime.rs         # QuickJS运行时
│   │   │   ├── api.rs             # 脚本API绑定
│   │   │   └── types.rs
│   │   ├── scheduler/             # 任务调度模块
│   │   │   ├── mod.rs
│   │   │   ├── task.rs            # 任务定义
│   │   │   ├── cron.rs            # Cron解析
│   │   │   └── queue.rs           # 优先级队列
│   │   ├── config/                # 配置管理模块
│   │   │   ├── mod.rs
│   │   │   ├── app_config.rs      # 应用配置
│   │   │   └── profile.rs         # Profile管理
│   │   ├── logger/                # 日志模块
│   │   │   ├── mod.rs
│   │   │   └── writer.rs          # 异步日志写入
│   │   ├── system/                # 系统集成模块
│   │   │   ├── mod.rs
│   │   │   ├── tray.rs            # 系统托盘
│   │   │   ├── process.rs         # 进程监控
│   │   │   └── autostart.rs       # 开机自启
│   │   └── commands/              # Tauri命令
│   │       ├── mod.rs
│   │       ├── controller_cmd.rs
│   │       ├── macro_cmd.rs
│   │       ├── script_cmd.rs
│   │       ├── scheduler_cmd.rs
│   │       ├── config_cmd.rs
│   │       └── log_cmd.rs
│   └── icons/                     # 应用图标
├── src/                           # Vue前端
│   ├── App.vue
│   ├── main.ts
│   ├── router/
│   │   └── index.ts
│   ├── stores/                    # Pinia状态管理
│   │   ├── controller.ts
│   │   ├── macro.ts
│   │   ├── script.ts
│   │   ├── scheduler.ts
│   │   ├── config.ts
│   │   └── log.ts
│   ├── views/                     # 页面视图
│   │   ├── DeviceMonitor.vue
│   │   ├── MacroControl.vue
│   │   ├── ScriptEditor.vue
│   │   ├── ConfigPanel.vue
│   │   ├── TaskScheduler.vue
│   │   └── LogViewer.vue
│   ├── components/                # 通用组件
│   │   ├── layout/
│   │   │   ├── AppSidebar.vue
│   │   │   ├── AppHeader.vue
│   │   │   └── StatusBar.vue
│   │   ├── controller/
│   │   │   ├── DeviceCard.vue
│   │   │   ├── StickVisualizer.vue
│   │   │   └── TriggerBar.vue
│   │   ├── macro/
│   │   │   ├── MacroTimeline.vue
│   │   │   └── PlaybackControls.vue
│   │   ├── script/
│   │   │   └── CodeEditor.vue
│   │   ├── scheduler/
│   │   │   └── TaskCard.vue
│   │   └── common/
│   │       ├── FButton.vue
│   │       ├── FCard.vue
│   │       ├── FInput.vue
│   │       ├── FModal.vue
│   │       ├── FSelect.vue
│   │       ├── FSlider.vue
│   │       ├── FToggle.vue
│   │       └── FTooltip.vue
│   ├── composables/               # 组合式函数
│   │   ├── useTauriEvent.ts
│   │   ├── useController.ts
│   │   └── useTheme.ts
│   ├── styles/                    # 全局样式
│   │   ├── variables.css
│   │   ├── base.css
│   │   └── themes/
│   │       └── dark.css
│   └── types/                     # TypeScript类型
│       ├── controller.ts
│       ├── macro.ts
│       ├── script.ts
│       ├── scheduler.ts
│       └── config.ts
├── design-system/                 # 设计系统
│   └── forzascript/
│       └── MASTER.md
├── docs/                          # 文档
│   └── design.md                  # 本文档
└── package.json
```

---

## 十、API接口设计

### 10.1 Tauri IPC命令

| 模块 | 命令 | 参数 | 返回值 | 说明 |
|------|------|------|--------|------|
| 手柄 | `controller:create` | `{type, index}` | `DeviceId` | 创建虚拟手柄 |
| 手柄 | `controller:remove` | `{deviceId}` | `void` | 移除虚拟手柄 |
| 手柄 | `controller:set_button` | `{deviceId, button, pressed}` | `void` | 设置按键状态 |
| 手柄 | `controller:set_thumb` | `{deviceId, thumb, x, y}` | `void` | 设置摇杆位置 |
| 手柄 | `controller:set_trigger` | `{deviceId, trigger, value}` | `void` | 设置扳机值 |
| 手柄 | `controller:get_state` | `{deviceId}` | `ControllerState` | 获取设备状态 |
| 手柄 | `controller:list` | `{}` | `Vec<DeviceInfo>` | 列出所有设备 |
| 宏 | `macro:start_record` | `{deviceId}` | `MacroId` | 开始录制 |
| 宏 | `macro:stop_record` | `{macroId}` | `Macro` | 停止录制 |
| 宏 | `macro:play` | `{macroId, speed, loop}` | `PlaybackId` | 回放宏 |
| 宏 | `macro:pause` | `{playbackId}` | `void` | 暂停回放 |
| 宏 | `macro:resume` | `{playbackId}` | `void` | 继续回放 |
| 宏 | `macro:stop` | `{playbackId}` | `void` | 停止回放 |
| 宏 | `macro:list` | `{}` | `Vec<MacroMeta>` | 列出所有宏 |
| 宏 | `macro:delete` | `{macroId}` | `void` | 删除宏 |
| 脚本 | `script:create` | `{name, code}` | `ScriptId` | 创建脚本 |
| 脚本 | `script:execute` | `{scriptId}` | `ExecutionId` | 执行脚本 |
| 脚本 | `script:stop` | `{executionId}` | `void` | 停止脚本 |
| 脚本 | `script:list` | `{}` | `Vec<ScriptMeta>` | 列出所有脚本 |
| 调度 | `scheduler:create_task` | `{task}` | `TaskId` | 创建定时任务 |
| 调度 | `scheduler:remove_task` | `{taskId}` | `void` | 删除任务 |
| 调度 | `scheduler:toggle_task` | `{taskId, enabled}` | `void` | 启用/禁用任务 |
| 调度 | `scheduler:list` | `{}` | `Vec<ScheduledTask>` | 列出所有任务 |
| 配置 | `config:get` | `{}` | `AppConfig` | 获取配置 |
| 配置 | `config:set` | `{config}` | `void` | 保存配置 |
| 配置 | `config:export_profile` | `{profileId}` | `FilePath` | 导出Profile |
| 配置 | `config:import_profile` | `{filePath}` | `ProfileId` | 导入Profile |
| 日志 | `log:query` | `{level, start, end, keyword}` | `Vec<LogEntry>` | 查询日志 |
| 日志 | `log:export` | `{format, query}` | `FilePath` | 导出日志 |

### 10.2 Tauri事件（后端→前端）

| 事件名 | 数据 | 说明 |
|--------|------|------|
| `controller:state_changed` | `DeviceInfo` | 设备状态变更 |
| `controller:device_error` | `{deviceId, error}` | 设备错误 |
| `macro:record_event` | `MacroEvent` | 录制事件 |
| `macro:playback_progress` | `{playbackId, progress}` | 回放进度 |
| `macro:playback_complete` | `{playbackId}` | 回放完成 |
| `script:output` | `{executionId, level, message}` | 脚本输出 |
| `script:error` | `{executionId, error}` | 脚本错误 |
| `scheduler:task_triggered` | `{taskId}` | 任务触发 |
| `scheduler:task_completed` | `{taskId, result}` | 任务完成 |
| `system:game_detected` | `{processName, pid}` | 检测到游戏 |
| `system:game_exit` | `{processName}` | 游戏退出 |
| `log:entry` | `LogEntry` | 新日志条目 |
