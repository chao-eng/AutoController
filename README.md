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

**AutoController** 是一款专为 Windows 平台打造的高性能、专业级虚拟手柄挂机与自动化工具。它基于 **Tauri 2.0** 框架，结合 **Vue 3 (TypeScript)** 现代化前端 and **Rust** 异步后端，底层通过 **ViGEmBus** 内核驱动实现高精度、低延迟 of 硬件级 Xbox 360 手柄信号模拟。无论是长时间挂机的核心玩家，还是需要进行批量自动化测试的开发人员，AutoController 都能提供直观、强悍 of 解决方案。

> ℹ️ **提示**：关于当前版本（v0.4.0）的详细功能更新，请查看 [最新更新日志](docs/update-log.md)。

---

## ✨ 核心特性

*   **🎮 专业级手柄模拟系统**
    *   底层集成 **ViGEmBus** 驱动，支持 Xbox 360 虚拟手柄模拟。
    *   **超低延迟**（<5ms）和**高精度**（100Hz+）输入模拟，保障极速响应。
    *   最多支持 **8 个虚拟手柄** 独立并发运行，并支持运行时热插拔管理。
    *   直观的可视化仪表盘，提供摇杆拖拽互动可视化及扳机键动态进度条。
*   **⏱️ 高精度宏录制与回放**
    *   一键捕获所有手柄动作事件，记录毫秒级时间戳。
    *   回放支持 **50% ~ 200% 的速度缩放** 调节，满足各种特定节奏需求。
    *   支持无限次循环播放与**断点续播**，可在暂停后精准恢复。
*   **📜 QuickJS 脚本引擎**
    *   内置轻量级沙箱化 **QuickJS** 脚本运行时，支持现代化 ES2020 语法。
    *   提供完备的脚本 API 绑定：手柄动作（按下/释放/移动摇杆/按压扳机）、高精度延时等待（`sleep`/`waitUntil`）、系统/进程事件监听及日志记录。
    *   集成 **Monaco Editor** 智能编辑器，支持 JavaScript/TypeScript 语法高亮、自定义 API 自动补全和实时错误检测。
*   **📅 智能任务调度器**
    *   支持**单次执行、每日定时、周期循环及标准 Cron 表达式**等多种调度方式。
    *   配备高优先级任务抢占机制与冲突解决队列，保障任务有序执行。
*   **🛡️ 系统集成与性能调优**
    *   支持**系统托盘最小化运行**、托盘气泡通知及运行状态托盘指示器。
    *   **智能进程监控**：自动识别目标游戏启动与退出状态。
    *   **智能资源调度**：游戏运行时自动开启高性能模式，空闲时降低采样率以实现超低功耗（CPU < 5%，内存 < 100MB）。
*   **⚙️ 配置文件 & 级联日志**
    *   采用 TOML 格式存储应用配置，按游戏独立保存 Profile。
    *   基于 Rust `tracing` 库实现的**异步级联日志系统**，支持日志轮转，并可一键导出为 JSON / CSV 格式。
*   **🔍 智能屏幕文字识别 (OCR) 系统**
    *   **原生/外部双引擎架构**：支持 **Windows 原生 (WinRT OCR)** 零依赖、高速度离线文字识别，以及 **外部 PaddleOCR (HTTP API)** 双模式运行。
    *   **PaddleOCR 双方案部署**：支持 **一键 EXE 独立部署**（零依赖、双击即用，从 GitHub Releases 下载）或 **Docker 容器化部署**（提供国内极速直连的阿里云优化镜像）。
    *   **多区域标定与脚本 API**：支持在 UI 界面多区域框选标定，脚本中通过 `ocr()`、`ocr(序号)` 或 `ocr(x,y,w,h)` 灵活读取。
    *   **自适应高保真增强**：短边小于 600px 时自动采用 GDI HALFTONE 差值算法智能高清晰度放大，保障极高字亲和性与识别率。
*   **🔔 多通道智能消息通知系统**
    *   深度集成 **飞书 Webhook (Feishu)**、**Server酱 (ServerChan)**、**Server酱三代 (ServerChan3)** 及 **Telegram Bot** 消息通道。
    *   支持脚本运行成功、出错、结束时以及自定义节点进行自动化远程消息推送，挂机动态了如指掌。
*   **👁️ 防止游戏/窗口失去焦点 (No Focus Loss)**
    *   底层基于**跨进程 DLL 注入**与 Hook 技术拦截窗口失活消息，解决切屏/后台挂机时游戏自动暂停、静音或降帧（FPS）问题。
    *   **物理隔离高安全注入**：将注入核心操作全部剥离至独立子进程 `injector.exe` 执行，彻底隔离敏感跨进程 Win32 API，规避主程序查杀报毒与崩溃。
    *   **管理员权限原生检测**：前端实时检测系统特权状态并以显著色块进行提示，针对高权限游戏实现完美的一键式注入及无痕“安全卸载”剥离。
    *   **极致单行折叠式交互**：免责声明与权限提示支持极简一键折叠，绝不占用多余的视觉操作空间。
    *   *（本项目中防止失去焦点技术实现参考自优秀开源项目 [NoFocusLoss](https://github.com/araghon007/NoFocusLoss)）*

---

## 🛠️ 技术栈

| 层级 | 选用技术 | 用途说明 |
| :--- | :--- | :--- |
| **桌面框架** | Tauri 2.0 (Rust) | 提供底层的 IPC 通信、托盘集成、文件系统访问与系统服务 |
| **前端 UI** | Vue 3 + TypeScript | 响应式界面开发，提供模块化的高自由度用户体验 |
| **状态管理** | Pinia 2.x | 全局共享设备状态、宏与脚本运行状态 |
| **样式/组件** | Vanilla CSS + Lucide Icons | 自定义精美工业风 (Vibrant & Block-based) 主题，配合 Lucide 图标 |
| **脚本运行时**| QuickJS (Rust 绑定) | 轻量沙箱环境，执行用户编写的手柄自动化脚本 |
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
- 按照向导完成软件安装。程序已**内置集成了所需的 ViGEmBus 内核驱动**，安装过程中会自动处理，**无需用户手动下载安装**。

### 2. 运行与配置
- **普通模式**：双击启动桌面的 **AutoController** 即可开始使用设备模拟、录制宏或编写 QuickJS 脚本。
- **管理员模式 (推荐 🛡️)**：如果您挂机的游戏具有高权限或需要使用 **防止窗口失去焦点 (No Focus Loss)** 注入技术，请在桌面上右键选择 **“以管理员身份运行”** 启动本程序。
- **OCR 增强**：如果编写的自动化脚本需要使用高精度文字识别，请参考下方 [🐳 PaddleOCR 本地服务部署教程](#🐳-paddleocr-本地服务部署教程-exe--docker) 开启外部服务。

---

## 🐳 PaddleOCR 本地服务部署教程 (EXE / Docker)

为了在脚本运行中使用更精准的 PaddleOCR 识别引擎，您需要在本地部署一个 OCR 服务。目前提供以下两种灵活的部署方案：

### 💡 方案 A：一键 EXE 点击部署 (推荐 🌟)
这是最简单、快捷且无需任何依赖（如 Docker / WSL2）的部署方式，适合绝大多数 Windows 用户。

1. **下载程序**：
   - 前往本项目的 [GitHub Releases 页面](https://github.com/chao-eng/AutoController/releases) 下载支持一键开启 OCR 服务的 EXE 启动文件（`ocr_service.exe`）。
2. **启动运行**：
   - 下载后双击运行程序（例如双击 `ocr_service.exe`）。
   - 看到控制台输出显示服务已在本地 `8050` 端口成功运行即可。整个服务离线运行，安全可靠。
3. **软件配置与联调**：
   - **验证**：打开浏览器访问 `http://127.0.0.1:8050`，若能正确显示 API 文档页面即部署成功。
   - 打开 **AutoController** 应用，切换至系统配置页面的 **OCR 自动化配置** 区域。
   - 将 OCR 引擎切换为 `PaddleOCR (HTTP API)`。
   - 在接口地址输入框中填写默认地址：`http://127.0.0.1:8050/ocr`。

---

### 🐳 方案 B：Docker 容器化部署
适合习惯使用 Docker 开发或已在本地安装 Docker 服务的用户。

1. **准备工作：启用 WSL 2**（若未安装）：
   - 以管理员身份打开 PowerShell，运行以下命令安装 WSL：
     ```powershell
     wsl --install
     ```
   - 安装完成后，请重启电脑以使配置生效。
2. **安装 Docker Desktop**（若未安装）：
   - [前往 Docker 官方网站](https://www.docker.com/products/docker-desktop/) 下载最新的 Docker Desktop for Windows 安装包，按照指引完成安装并启动。
3. **拉取并运行 PaddleOCR 镜像**：
   - 在 PowerShell 或 CMD 中运行以下指令拉取并后台启动 OCR 服务（该镜像已发布于阿里云容器仓库，国内直连拉取速度极快，我们将主机端口映射到了默认的 `8050`）：
     ```powershell
     docker run -d -p 8050:8000 --name win-paddleocr --restart always crpi-a1liy20beodq2bdl.cn-beijing.personal.cr.aliyuncs.com/bujic/win-paddleocr-x86:latest
     ```
4. **验证与联调**：
   - **验证**：打开浏览器访问 `http://127.0.0.1:8050/docs#/`，若能正确显示 API 文档页面即部署成功。
   - **配置**：打开 **AutoController** 应用，切换至系统配置页面的 **OCR 自动化配置**，将引擎切换为 `PaddleOCR (HTTP API)`，输入地址 `http://127.0.0.1:8050/ocr`。

</details>

---

<details>
<summary><b>🇬🇧 English (Click to expand/collapse)</b></summary>

## 📝 Project Overview

**AutoController** is a professional-grade, high-performance virtual gamepad simulation and automation utility designed specifically for Windows. Powered by the **Tauri 2.0** framework, featuring a modern **Vue 3 (TypeScript)** frontend paired with an asynchronous **Rust** backend, it leverages the kernel-level **ViGEmBus** driver to deliver hardware-level, high-precision, and low-latency Xbox 360 gamepad emulation. Whether you are a core gamer looking for long-term AFK gaming or a developer running batch automated testing, AutoController offers an intuitive, sleek, and robust solution.

> ℹ️ **Note**: For detailed features and release details on the current version (v0.4.0), check the [Latest Update Log](docs/update-log.md).

---

## ✨ Key Features

*   **🎮 Professional Gamepad Simulation**
    *   Kernel-level emulation for Xbox 360 controller powered by the **ViGEmBus** driver.
    *   **Ultra-low latency** (<5ms) and **high-precision** (100Hz+) input simulation.
    *   Supports up to **8 virtual gamepads** running concurrently with dynamic hot-swapping.
    *   Intelligent visualization dashboard including interactive drag-and-drop joystick fields and dynamic trigger progress bars.
*   **⏱️ High-Precision Macro Recorder & Player**
    *   One-click gamepad input capture with millisecond-precision timestamps.
    *   Speed scaling controls from **50% to 200%** playback speed.
    *   Infinite looping options and **breakpoint-based resume** to continue playback seamlessly after pausing.
*   **📜 QuickJS Scripting Engine**
    *   Integrated sandboxed **QuickJS** runtime supporting modern ES2020 JavaScript/TypeScript syntax.
    *   Rich API bindings: controller actions (press, release, move axes, pull triggers), high-precision waiting mechanisms (`sleep`, `waitUntil`), game/process detection events, and logging.
    *   Embedded **Monaco Editor** with advanced syntax highlighting, custom API autocompletion, and live linting.
*   **📅 Intelligent Task Scheduler**
    *   Automates runs using **one-shot timers, daily timings, interval triggers, and standard Cron expressions**.
    *   Priority-based task preemption and conflict resolution queues ensure smooth execution.
*   **🛡️ System Integration & Performance Tuning**
    *   System tray integration (minimize to tray, quick actions, status indicators, and notification balloons).
    *   **Process Monitoring**: Automatically detects target game launch and termination.
    *   **Smart Resource Allocation**: Boosts performance when games are active; drops sampling rates during idle times to keep resource footprints minimal (CPU < 5%, Memory < 100MB).
*   **⚙️ Profiles & Structured Logging**
    *   Saves game-specific profiles independently using user-friendly TOML files.
    *   Asynchronous logging powered by Rust's `tracing` library with log-rotation, reloadable filters, and CSV/JSON exports.
*   **🔍 Intelligent Screen OCR Text Recognition**
    *   **Dual-Engine Architecture**: Seamlessly switch between zero-dependency, ultra-fast **Windows Native (WinRT OCR)** and high-precision **External PaddleOCR (HTTP API)**.
    *   **Flexible PaddleOCR Deployment**: Supports both **One-Click EXE Deployment** (zero-dependency, double-click to run, available on GitHub Releases) and **Docker Containerization** (via highly-optimized registry image).
    *   **Multi-Region Calibration & Scripting APIs**: Frame and select multiple target regions on the UI, and query screen text instantly via `ocr()`, `ocr(index)`, or `ocr(x,y,w,h)` scripts.
    *   **Adaptive Image Enhancing**: Automatically scales smaller bounding boxes (<600px) using GDI HALFTONE interpolation to guarantee outstanding text clarity and accuracy.
*   **🔔 Multi-Channel Notification Dispatcher**
    *   Out-of-the-box integration with **Feishu Webhook**, **ServerChan**, **ServerChan3**, and **Telegram Bot**.
    *   Automates remote message pushes on script status changes (started, success, warning, error) or manually triggered checkpoints.
*   **👁️ Prevent Game/Window Focus Loss (No Focus Loss)**
    *   Leverages **cross-process DLL Injection** and Hook technology to intercept window deactivation messages, allowing games to continue rendering at full speed, playing sounds, and auto-farming even in the background.
    *   **Sub-process Physical Isolation**: Isolates sensitive cross-process Win32 API calls within an independent background sub-process `injector.exe`, avoiding antivirus false-alarms or core shell crashes.
    *   **Native Administrator Privilege Detection**: Evaluates privilege context and guides users (via orange warning banners) to restart under administrator mode for seamless injection and clean unloads.
    *   **Space-Saving Collapsible Headers**: Toggles warning banners and guidelines in a single-row collapsible container to keep workspaces clean.
    *   *(Credit for focus loss prevention goes to the open-source project [NoFocusLoss](https://github.com/araghon007/NoFocusLoss))*

---

## 🛠️ Tech Stack

| Layer | Technology | Description |
| :--- | :--- | :--- |
| **Desktop Shell** | Tauri 2.0 (Rust) | Manages IPC communications, tray integration, file I/O, and native system events |
| **Frontend UI** | Vue 3 + TypeScript | Modular, responsive interface with customized high-fidelity interactions |
| **State Management** | Pinia 2.x | Manages application state across devices, macro players, and active scripts |
| **Styling & Assets** | Vanilla CSS + Lucide Icons | Premium, dark-mode industrial visual style (Vibrant & Block-based) with Lucide SVGs |
| **Script Engine** | QuickJS (Rust bindings) | Secure, lightweight sandbox executing custom user automation scripts |
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

## 🐳 PaddleOCR Local Service Deployment Guide (EXE / Docker)

To utilize the higher-precision PaddleOCR engine in your scripts, you need to deploy a local OCR HTTP service. We provide two flexible options:

### 💡 Option A: One-Click EXE Deployment (Recommended 🌟)
This is the simplest, zero-dependency, and fastest deployment method, suitable for most Windows users.

1. **Download the Service**:
   - Go to the [GitHub Releases page](https://github.com/chao-eng/AutoController/releases) of this repository and download the Windows OCR service package (e.g., `win-paddleocr.zip` or `ocr_service.exe`).
2. **Launch & Run**:
   - Extract and double-click the executable (e.g., `ocr_service.exe`) to start the service.
   - The terminal console will spin up the local server on the default port `8050`. Keep this window open while using OCR.
3. **Connect in AutoController**:
   - Open **AutoController**, navigate to the **OCR Configuration** tab.
   - Select the OCR engine to `PaddleOCR (HTTP API)`.
   - Input the local OCR API endpoint: `http://127.0.0.1:8050/ocr` and click **Test Connection**.

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
