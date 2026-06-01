# 🎮 AutoController

<p align="center">
  <strong>基于 Tauri 2.0 + Vue 3 + ViGEmBus 的专业级游戏手柄模拟与自动化挂机工具</strong><br/>
  <em>Professional-Grade Gamepad Simulation & Automation Tool Powered by Tauri 2.0 + Vue 3 + ViGEmBus</em>
</p>

<p align="center">
  <a href="#-简体中文">🇨🇳 简体中文</a> | <a href="#-english">🇬🇧 English</a>
</p>

---

<details open>
<summary><b>🇨🇳 简体中文 (点击可折叠/展开)</b></summary>

## 📝 项目概述

**AutoController** 是一款专为 Windows 平台打造的高性能、专业级虚拟手柄挂机与自动化工具。它基于 **Tauri 2.0** 框架，结合 **Vue 3 (TypeScript)** 现代化前端与 **Rust** 异步后端，底层通过 **ViGEmBus** 内核驱动实现高精度、低延迟的硬件级 Xbox 360 手柄信号模拟。无论是长时间挂机的核心玩家，还是需要进行批量自动化测试的开发人员，AutoController 都能提供直观、强悍的解决方案。

> ℹ️ **提示**：关于当前版本（v0.5.0）的详细功能更新，请查看 [最新更新日志](docs/update-log.md)。

---

## ✨ 核心特性

* **🎮 专业级手柄模拟系统**
    * 底层集成 **ViGEmBus** 驱动，支持 Xbox 360 虚拟手柄模拟。
    * **超低延迟**（<5ms）和**高精度**（100Hz+）输入模拟，保障极速响应。
    * 最多支持 **8 个虚拟手柄** 独立并发运行，并支持运行时热插拔管理。
    * 直观的可视化仪表盘，提供摇杆拖拽互动可视化及扳机键动态进度条。
* **⏱️ 高精度宏录制与回放**
    * 一键捕获所有手柄动作事件，记录毫秒级时间戳。
    * 回放支持 **50% ~ 200% 的速度缩放** 调节，满足各种特定节奏需求。
    * 支持无限次循环播放与**断点续播**，可在暂停后精准恢复。
* **📜 QuickJS 脚本引擎**
    * 内置轻量级沙箱化 **QuickJS** 脚本运行时，支持现代化 ES2020 语法。
    * 提供完备的脚本 API 绑定：手柄动作（按下/释放/移动摇杆/按压扳机）、高精度延时等待（`sleep`/`waitUntil`）、系统/进程事件监听及日志记录。
    * 集成 **Monaco Editor** 智能编辑器，支持 JavaScript/TypeScript 语法高亮、自定义 API 自动补全和实时错误检测。
* **📅 智能任务调度器**
    * 支持**单次执行、每日定时、周期循环及标准 Cron 表达式**等多种调度方式。
    * 配备高优先级任务抢占机制与冲突解决队列，保障任务有序执行。
* **🛡️ 系统集成与性能调优**
    * 支持**系统托盘最小化运行**、托盘气泡通知及运行状态托盘指示器。
    * **智能进程监控**：自动识别目标游戏启动与退出状态。
    * **智能资源调度**：游戏运行时自动开启高性能模式，空闲时降低采样率以实现超低功耗（CPU < 5%，内存 < 100MB）。
* **⚙️ 配置文件 & 级联日志**
    * 采用 TOML 格式存储应用配置，按游戏独立保存 Profile。
    * 基于 Rust `tracing` 库实现的**异步级联日志系统**，支持日志轮转，并可一键导出为 JSON / CSV 格式。
* **🔍 Rust 原生绑定 PaddleOCR 离线引擎 (免部署 🌟)**
    * **零外部依赖** (`app/src-tauri/src/ocr`)：全面重构 OCR 底层架构，将 PaddleOCR 核心识别引擎通过 **Rust 底层原生绑定** 直接无缝嵌入主程序，**不再需要**在外部单独部署任何 Python 环境、EXE 独立服务或 Docker 容器。
    * **内存级超低延迟**：截图数据直接在内存中完成像素流传递与 OCR 解析，免去磁盘 I/O 与复杂的网络序列化开销，为高频画面状态检测提供毫秒级响应。
    * **完备双本地引擎**：支持切换 **内置 PaddleOCR** 离线引擎与 **Windows 原生 (WinRT OCR)** 引擎，满足不同场景下的纯离线文字识别需求。
    * **多区域标定与脚本 API**：支持在 UI 界面多区域框选标定，脚本中通过 `ocr()`、`ocr(序号)` 或 `ocr(x,y,w,h)` 灵活读取。
    * **自适应高保真增强**：短边小于 600px 时自动采用 GDI HALFTONE 差值算法智能高清晰度放大，保障极高字亲和性与识别率。
* **🔔 多通道智能消息通知系统**
    * 深度集成 **飞书 Webhook (Feishu)**、**Server酱 (ServerChan)**、**Server酱三代 (ServerChan3)** 及 **Telegram Bot** 消息通道。
    * 支持脚本运行成功、出错、结束时以及自定义节点进行自动化远程消息推送，挂机动态了如指掌。
* **👁️ 防止游戏/窗口失去焦点 (No Focus Loss)**
    * 底层基于**跨进程 DLL 注入**与 Hook 技术拦截窗口失活消息，解决切屏/后台挂机时游戏自动暂停、静音或降帧（FPS）问题。
    * **物理隔离高安全注入**：将注入核心操作全部剥离至独立子进程 `injector.exe` 执行，彻底隔离敏感跨进程 Win32 API，规避主程序查杀报毒与崩溃。
    * **管理员权限原生检测**：前端实时检测系统特权状态并以显著色块进行提示，针对高权限游戏实现完美的一键式注入及无痕“安全卸载”剥离。
    * **极致单行折叠式交互**：免责声明与权限提示支持极简一键折叠，绝不占用多余的视觉操作空间。
    * *（本项目中防止失去焦点技术实现参考自优秀开源项目 [NoFocusLoss](https://github.com/araghon007/NoFocusLoss)）*

---

## 🛠️ 技术栈

| 层级 | 选用技术 | 用途说明 |
| :--- | :--- | :--- |
| **桌面框架** | Tauri 2.0 (Rust) | 提供底层的 IPC 通信、托盘集成、文件系统访问与系统服务 |
| **前端 UI** | Vue 3 + TypeScript | 响应式界面开发，提供模块化的高自由度用户体验 |
| **状态管理** | Pinia 2.x | 全局共享设备状态、宏与脚本运行状态 |
| **样式/组件** | Vanilla CSS + Lucide Icons | 自定义精美工业风 (Vibrant & Block-based) 主题，配合 Lucide 图标 |
| **脚本运行时**| QuickJS (Rust 绑定) | 轻量沙箱环境，执行用户编写的手柄自动化脚本 |
| **OCR 引擎** | PaddleOCR (Rust 原生绑定) | 主程序内置的离线高性能文本识别引擎，免外部部署 |
| **内核模拟** | ViGEmBus Driver (1.17+) | 核心虚拟手柄硬件级信号生成驱动 |

---

## 📂 目录结构

本项目的核心前端与后端均位于 `app/` 目录下：


```
AutoController/
├── app/                           # 应用程序核心
│   ├── src-tauri/                 # Rust 后端 (Tauri 2.0 框架)
│   │   ├── Cargo.toml             # 后端 Rust 依赖配置
│   │   └── src/
│   │       ├── controller/        # 手柄模拟与 ViGEmBus 通信
│   │       ├── macro_engine/      # 宏录制与回放引擎
│   │       ├── script_engine/     # QuickJS 引擎与 API 绑定
│   │       ├── scheduler/         # 任务调度与 Cron 解析
│   │       └── system/            # 托盘与进程监控集成
│   └── src/                       # Vue 3 前端 (TypeScript)
│       ├── views/                 # 页面视图 (设备监控、编辑器、调度器等)
│       ├── components/            # 通用及模块专用组件 (摇杆/扳机可视化)
│       └── stores/                # Pinia 状态管理仓库
├── design-system/                 # 系统 UI 视觉规范设计文档
├── docs/                          # 设计及任务计划文档
└── assets/                        # 静态资源与应用图标
```

---

## 🚀 使用说明

### 1. 下载安装
- 前往本项目的 [GitHub Releases 页面](https://github.com/chao-eng/AutoController/releases) 下载最新版本的 Windows 安装包（如 `.msi` 或打包好的独立 `.exe` 执行文件）。
- 按照向导完成软件安装。程序已**内置集成了所需的 ViGEmBus 内核驱动**与 **离线 PaddleOCR 模型**，安装过程中会自动处理，**无任何外部依赖，解压/安装即用**。

### 2. 运行与配置
- **普通模式**：双击启动桌面的 **AutoController** 即可开始使用设备模拟、录制宏、编写 QuickJS 脚本以及调用全内置的离线 OCR 功能。
- **管理员模式 (推荐 🛡️)**：如果您挂机的游戏具有高权限或需要使用 **防止窗口失去焦点 (No Focus Loss)** 注入技术，请在桌面上右键选择 **“以管理员身份运行”** 启动本程序。

---


</details>

---

<details>
<summary><b>🇬🇧 English (Click to expand/collapse)</b></summary>

## 📝 Project Overview

**AutoController** is a professional-grade, high-performance virtual gamepad simulation and automation utility designed specifically for Windows. Powered by the **Tauri 2.0** framework, featuring a modern **Vue 3 (TypeScript)** frontend paired with an asynchronous **Rust** backend, it leverages the kernel-level **ViGEmBus** driver to deliver hardware-level, high-precision, and low-latency Xbox 360 gamepad emulation. Whether you are a core gamer looking for long-term AFK gaming or a developer running batch automated testing, AutoController offers an intuitive, sleek, and robust solution.

> ℹ️ **Note**: For detailed features and release details on the current version (v0.5.0), check the [Latest Update Log](docs/update-log.md).

---

## ✨ Key Features

* **🎮 Professional Gamepad Simulation**
    * Kernel-level emulation for Xbox 360 controller powered by the **ViGEmBus** driver.
    * **Ultra-low latency** (<5ms) and **high-precision** (100Hz+) input simulation.
    * Supports up to **8 virtual gamepads** running concurrently with dynamic hot-swapping.
    * Intelligent visualization dashboard including interactive drag-and-drop joystick fields and dynamic trigger progress bars.
* **⏱️ High-Precision Macro Recorder & Player**
    * One-click gamepad input capture with millisecond-precision timestamps.
    * Speed scaling controls from **50% to 200%** playback speed.
    * Infinite looping options and **breakpoint-based resume** to continue playback seamlessly after pausing.
* **📜 QuickJS Scripting Engine**
    * Integrated sandboxed **QuickJS** runtime supporting modern ES2020 JavaScript/TypeScript syntax.
    * Rich API bindings: controller actions (press, release, move axes, pull triggers), high-precision waiting mechanisms (`sleep`, `waitUntil`), game/process detection events, and logging.
    * Embedded **Monaco Editor** with advanced syntax highlighting, custom API autocompletion, and live linting.
* **📅 Intelligent Task Scheduler**
    * Automates runs using **one-shot timers, daily timings, interval triggers, and standard Cron expressions**.
    * Priority-based task preemption and conflict resolution queues ensure smooth execution.
* **🛡️ System Integration & Performance Tuning**
    * System tray integration (minimize to tray, quick actions, status indicators, and notification balloons).
    * **Process Monitoring**: Automatically detects target game launch and termination.
    * **Smart Resource Allocation**: Boosts performance when games are active; drops sampling rates during idle times to keep resource footprints minimal (CPU < 5%, Memory < 100MB).
* **⚙️ Profiles & Structured Logging**
    * Saves game-specific profiles independently using user-friendly TOML files.
    * Asynchronous logging powered by Rust's `tracing` library with log-rotation, reloadable filters, and CSV/JSON exports.
* **🔍 Native Rust-Bound PaddleOCR Offline Engine (Zero-Dependency 🌟)**
    * **Zero External Dependencies** (`app/src-tauri/src/ocr`): The architecture has been completely overhauled. The core PaddleOCR engine is now directly embedded into the main application via **native Rust bindings**. Users **no longer need** to deploy any external Python environments, standalone EXEs, or Docker containers.
    * **Memory-Level Ultra-Low Latency**: Screenshot data is transferred as raw pixel streams directly in memory for OCR analysis. It eliminates disk I/O and complex network serialization overhead, unlocking millisecond-level responsiveness for high-frequency state detection.
    * **Dual Native Local Engines**: Seamlessly toggle between the fully **Built-in PaddleOCR** offline engine and **Windows Native (WinRT OCR)** to satisfy all offline text recognition needs.
    * **Multi-Region Calibration & Scripting APIs**: Frame and select multiple target regions on the UI, and query screen text instantly via `ocr()`, `ocr(index)`, or `ocr(x,y,w,h)` scripts.
    * **Adaptive Image Enhancing**: Automatically scales smaller bounding boxes (<600px) using GDI HALFTONE interpolation to guarantee outstanding text clarity and accuracy.
* **🔔 Multi-Channel Notification Dispatcher**
    * Out-of-the-box integration with **Feishu Webhook**, **ServerChan**, **ServerChan3**, and **Telegram Bot**.
    * Automates remote message pushes on script status changes (started, success, warning, error) or manually triggered checkpoints.
* **👁️ Prevent Game/Window Focus Loss (No Focus Loss)**
    * Leverages **cross-process DLL Injection** and Hook technology to intercept window deactivation messages, allowing games to continue rendering at full speed, playing sounds, and auto-farming even in the background.
    * **Sub-process Physical Isolation**: Isolates sensitive cross-process Win32 API calls within an independent background sub-process `injector.exe`, avoiding antivirus false-alarms or core shell crashes.
    * **Native Administrator Privilege Detection**: Evaluates privilege context and guides users (via orange warning banners) to restart under administrator mode for seamless injection and clean unloads.
    * **Space-Saving Collapsible Headers**: Toggles warning banners and guidelines in a single-row collapsible container to keep workspaces clean.
    * *(Credit for focus loss prevention goes to the open-source project [NoFocusLoss](https://github.com/araghon007/NoFocusLoss))*

---

## 🛠️ Tech Stack

| Layer | Technology | Description |
| :--- | :--- | :--- |
| **Desktop Shell** | Tauri 2.0 (Rust) | Manages IPC communications, tray integration, file I/O, and native system events |
| **Frontend UI** | Vue 3 + TypeScript | Modular, responsive interface with customized high-fidelity interactions |
| **State Management** | Pinia 2.x | Manages application state across devices, macro players, and active scripts |
| **Styling & Assets** | Vanilla CSS + Lucide Icons | Premium, dark-mode industrial visual style (Vibrant & Block-based) with Lucide SVGs |
| **Script Engine** | QuickJS (Rust bindings) | Secure, lightweight sandbox executing custom user automation scripts |
| **OCR Engine** | PaddleOCR (Rust Bindings) | Fully embedded, high-performance offline OCR engine (Zero setup required) |
| **Simulation Driver** | ViGEmBus Driver (1.17+) | Kernel-level driver generating virtual game controller signals |

---

## 📂 Directory Structure

Both frontend and backend resources are contained under the `app/` directory:

```
AutoController/
├── app/                           # Core Application Directory
│   ├── src-tauri/                 # Rust Backend (Tauri 2.0 Shell)
│   │   ├── Cargo.toml             # Rust dependencies
│   │   └── src/
│   │       ├── controller/        # Gamepad emulation & ViGEmBus client
│   │       ├── macro_engine/      # Macro record & playback system
│   │       ├── script_engine/     # QuickJS sandboxing & API bindings
│   │       ├── scheduler/         # Task queue & Cron parser
│   │       └── system/            # Tray & process monitor integration
│   └── src/                       # Vue 3 Frontend (TypeScript)
│       ├── views/                 # View pages (Device, Editor, Scheduler, etc.)
│       ├── components/            # Reusable UI components (Stick/Trigger visualizations)
│       └── stores/                # Pinia state stores
├── design-system/                 # UI Design guidelines & tokens
├── docs/                          # Architecture designs & checklist documents
└── assets/                        # Static assets and icons
```

---

## 🚀 Usage Guide

### 1. Download & Install
- Go to the [GitHub Releases page](https://github.com/chao-eng/AutoController/releases) and download the latest Windows installer (e.g., `.msi` or standalone `.exe`).
- Follow the wizard to complete the installation. The required **ViGEmBus** kernel driver is **fully bundled with the software** and will be configured automatically—**no manual driver installation is required** for end-users.

### 2. Running & Configuring
- **Standard Mode**: Double-click the **AutoController** desktop icon to start simulating controllers, recording macros, or writing QuickJS scripts.
- **Administrator Mode (Recommended 🛡️)**: If the target game runs under elevated privileges or if you wish to use the **Prevent Focus Loss (No Focus Loss)** injection feature, right-click the shortcut and select **"Run as Administrator"**.
- **OCR Integration**: If your automation scripts require high-precision screen text detection, follow the [🐳 PaddleOCR Local Service Deployment Guide](#🐳-paddleocr-local-service-deployment-guide-exe--docker) below to set up your local OCR endpoint.

---


### 🐳 Option B: Docker Container Deployment
Ideal for developers who prefer containerized services or already have Docker Desktop configured.

1. **Prerequisites: Enable WSL 2** (if not installed):
   - Open PowerShell as Administrator, and run:
     ```powershell
     wsl --install
     ```
   - Restart your computer after installation to apply the changes.
2. **Install Docker Desktop** (if not installed):
   - Visit [Docker Desktop for Windows Official Page](https://www.docker.com/products/docker-desktop/) to download and complete the setup.
3. **Pull and Run the PaddleOCR Image**:
   - Run the following command in PowerShell/CMD to pull and run the OCR container in the background (mapping to the default `8050` port):
     ```powershell
     docker run -d -p 8050:8000 --name win-paddleocr --restart always crpi-a1liy20beodq2bdl.cn-beijing.personal.cr.aliyuncs.com/bujic/win-paddleocr-x86:latest
     ```
4. **Verification & Configuration**:
   - **Verify**: Open your browser and navigate to `http://127.0.0.1:8050/docs#/` to view the API documentation page.
   - **Configure**: In **AutoController**'s **OCR Configuration** tab, switch engine to `PaddleOCR (HTTP API)` and input endpoint: `http://127.0.0.1:8050/ocr`. Click **Test Connection**.

</details>
