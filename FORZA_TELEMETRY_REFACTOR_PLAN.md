# 全量重构计划：统一 Tailwind + shadcn-vue (Reka UI)

## 目标

1. **Forza Telemetry**: Svelte → Vue + Tailwind + Reka UI（含 Pinia stores）
2. **所有 Vue 页面**: 从 `variables.css` + scoped CSS 统一为 Tailwind + shadcn-vue
3. **清理**: 移除 `variables.css`，只保留 `tailwind.css`

---

## 状态

### 🟢 P0: 基础设施（1 文件）
- [ ] 统一 App.vue / layout 组件的引用，确保 tailwind.css 优先

### 🟢 P1: Forza Telemetry — Stores（2 文件）
- [x] `stores/telemetry.ts` (Svelte writable/derived → Pinia)
- [x] `stores/sessions.ts` (Svelte writable/derived → Pinia)

### 🟢 P2: Forza Telemetry — 纯展示组件（5 文件）
- [x] `components/telemetry/SteeringWheel.vue`
- [x] `components/telemetry/AttitudeIndicator.vue`
- [x] `components/telemetry/GForceMeter.vue`
- [x] `components/telemetry/InputBars.vue`
- [x] `components/telemetry/TachometerGauge.vue`

### 🟢 P3: Forza Telemetry — 复合组件（5 文件）
- [x] `components/telemetry/TopBar.vue`
- [x] `components/telemetry/CompassBar.vue`
- [x] `components/telemetry/CenterPanel.vue`
- [x] `components/telemetry/TireWidget.vue`
- [x] `components/telemetry/LapBar.vue`

### 🟢 P4: Forza Telemetry — 交互组件（4 文件）
- [x] `components/telemetry/FloatingPanel.vue`
- [x] `components/telemetry/SessionDrawer.vue`
- [x] `components/telemetry/ReplayBar.vue`
- [x] `components/telemetry/SettingsDialog.vue`

### 🟢 P5: Forza Telemetry — 复杂视图（2 文件）
- [x] `components/telemetry/SessionViewer.vue`
- [x] `components/telemetry/AnalysisTab.vue`

### 🟢 P6: Forza Telemetry — 主视图 + 清理（2 文件）
- [ ] `components/telemetry/ForzaDashboard.vue`
- [ ] `views/ForzaTelemetry.vue`（不再 embed Svelte）
- [ ] 删除所有 `*.svelte` 文件

### 🔵 P7: DeviceMonitor 页（4 文件）
- [ ] `views/DeviceMonitor.vue` (scoped CSS → Tailwind + shadcn-vue)
- [ ] `components/controller/DeviceCard.vue`
- [ ] `components/controller/StickVisualizer.vue`
- [ ] `components/controller/TriggerBar.vue`

### 🔵 P8: ScriptEditor 页（2 文件）
- [ ] `views/ScriptEditor.vue` (scoped CSS → Tailwind + shadcn-vue)
- [ ] `components/script/CodeEditor.vue`

### 🔵 P9: ConfigPanel 页（1 文件）
- [ ] `views/ConfigPanel.vue` (scoped CSS → Tailwind + shadcn-vue)

### 🔵 P10: TaskScheduler 页（1 文件）
- [ ] `views/TaskScheduler.vue` (scoped CSS → Tailwind + shadcn-vue)

### 🔵 P11: LogViewer 页（1 文件）
- [ ] `views/LogViewer.vue` (scoped CSS → Tailwind + shadcn-vue)

### 🔵 P12: NotificationConfig 页（1 文件）
- [ ] `views/NotificationConfig.vue` (scoped CSS → Tailwind + shadcn-vue)

### 🔵 P13: NoFocusLoss 页（1 文件）
- [ ] `views/NoFocusLoss.vue` (scoped CSS → Tailwind + shadcn-vue)

### 🔵 P14: MacroControl 页（1 文件）
- [ ] `views/MacroControl.vue` (scoped CSS → Tailwind + shadcn-vue)

### 🔵 P15: 清理收尾（1+ 文件）
- [ ] `src/styles/variables.css` — 删除（已迁移到 tailwind.css）
- [ ] 清理 `package.json` 中 Svelte 相关依赖
- [ ] 验证所有页面路由正常

---

## 文件清单

### 现有 Vue 页面（需改造样式）
| 页面 | 路由 | 说明 |
|---|---|---|
| DeviceMonitor | `/devices` | 自定义按钮/卡片/可视化 → shadcn Button/Card/Badge |
| ScriptEditor | `/scripts` | 自定义排版/按钮/输入 → shadcn Button/Input/Select |
| ConfigPanel | `/config` | 自定义配置列表/弹窗 → shadcn Card/Dialog/Select |
| TaskScheduler | `/scheduler` | 自定义任务列表/表单 → shadcn Card/Dialog/Select/Switch |
| LogViewer | `/logs` | 自定义过滤按钮/表格 → shadcn Button/Badge |
| NotificationConfig | `/notifications` | 自定义通道管理 → shadcn Card/Dialog/Input |
| NoFocusLoss | `/nofocus` | 自定义进程列表 → shadcn Card/Badge/Button |
| MacroControl | `/macro` | 自定义录制/回放 → shadcn Card/Button/Input |

### Svelte 文件（需转 Vue + 改造）
15 个 `.svelte` 文件全部在 `fh6-tel/` 目录下，转换为 `components/telemetry/` 中对应的 `.vue` 文件。

### 保留的纯 TS/JSON 文件
| 文件 | 说明 |
|---|---|
| `fh6-tel/lib/types.ts` | 纯 TypeScript 类型定义 |
| `fh6-tel/lib/ipc.ts` | 纯 TS IPC 通信 |
| `fh6-tel/lib/analysis.ts` | 纯 TS 数据分析 |
| `fh6-tel/lib/car-name.ts` | 纯 TS 车型映射 |
| `fh6-tel/lib/mapCrs.ts` | 纯 TS 地图坐标 |
| `fh6-tel/lib/*.json` | 纯 JSON 数据 |

---

## 技术决策

| 决策点 | 选择 |
|---|---|
| UI 组件库 | Reka UI (Radix Vue) + shadcn-vue（项目已有） |
| 图标库 | @lucide/vue（项目已有） |
| 状态管理 | Pinia composition API（项目标准） |
| 图表 | uPlot（保持现有，项目已依赖） |
| 样式 | Tailwind CSS + cn()（项目标准） |
| 浮动面板 | Vue composable 封装 drag/resize（shadcn 无现成组件） |
| 颜色体系 | tailwind.css 中 oklch CSS 变量 + Tailwind dark: |
| 组件映射 | 自定义 `<button class="icon-btn">` → `<Button variant="ghost" size="icon">` |
| 组件映射 | 自定义 `<div class="page-header">` → `<Card><CardHeader>` |
| 组件映射 | 自定义 `<div class="filter-group">` → `<div class="flex gap-1 bg-muted p-0.5 rounded-md">` |