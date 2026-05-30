<!-- app/src/views/NoFocusLoss.vue -->
<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { 
  AlertTriangle, 
  RefreshCw, 
  Search, 
  Zap, 
  ZapOff, 
  ShieldAlert, 
  HelpCircle, 
  Info,
  CheckCircle,
  XCircle
} from '@lucide/vue'

interface ProcessInfo {
  pid: number
  name: string
  window_title: string
  is_64bit: boolean
}

// 进程数据状态
const injectableProcesses = ref<ProcessInfo[]>([])
const injectedProcesses = ref<ProcessInfo[]>([])
const searchQuery = ref('')
const loading = ref(false)
const errorMessage = ref<string | null>(null)
const successMessage = ref<string | null>(null)

// 显示 Defender 信任区引导指南
const showGuide = ref(false)

// 过滤后的待注入进程列表
const filteredProcesses = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) return injectableProcesses.value

  return injectableProcesses.value.filter(
    (p) =>
      p.pid.toString().includes(query) ||
      p.name.toLowerCase().includes(query) ||
      p.window_title.toLowerCase().includes(query)
  )
})

// 获取可注入的窗口进程列表
async function fetchProcesses() {
  loading.value = true
  errorMessage.value = null
  try {
    const list = await invoke<ProcessInfo[]>('get_injectable_processes')
    // 排除已经注入过的进程
    const injectedPids = injectedProcesses.value.map((ip) => ip.pid)
    injectableProcesses.value = list.filter((p) => !injectedPids.includes(p.pid))
  } catch (err: any) {
    console.error('获取窗口进程列表失败:', err)
    errorMessage.value = `获取窗口进程列表失败: ${err.toString()}`
  } finally {
    loading.value = false
  }
}

// 执行注入 Hook
async function injectHook(proc: ProcessInfo) {
  errorMessage.value = null
  successMessage.value = null
  
  try {
    await invoke('inject_focus_hook', { pid: proc.pid, is64bit: proc.is_64bit })
    
    // 从待注入移动到已注入
    injectableProcesses.value = injectableProcesses.value.filter((p) => p.pid !== proc.pid)
    injectedProcesses.value.push(proc)
    
    successMessage.value = `成功将防失去焦点 Hook 注入至进程「${proc.name}」(PID: ${proc.pid})！`
    
    // 自动清除成功提示
    setTimeout(() => {
      if (successMessage.value?.includes(proc.pid.toString())) {
        successMessage.value = null
      }
    }, 5000)
  } catch (err: any) {
    console.error('注入 Hook 失败:', err)
    errorMessage.value = err.toString()
    
    // 如果提示核心组件丢失/拦截，则自动展开安全引导提示
    if (err.toString().includes('injector.exe') || err.toString().includes('拦截') || err.toString().includes('隔离')) {
      showGuide.value = true
    }
  }
}

// 执行安全卸载 Hook
async function unloadHook(proc: ProcessInfo) {
  errorMessage.value = null
  successMessage.value = null
  
  try {
    await invoke('unload_focus_hook', { pid: proc.pid, is64bit: proc.is_64bit })
    
    // 从已注入移动回待注入
    injectedProcesses.value = injectedProcesses.value.filter((p) => p.pid !== proc.pid)
    injectableProcesses.value.push(proc)
    
    // 重新排序待注入列表
    injectableProcesses.value.sort((a, b) => a.name.toLowerCase().localeCompare(b.name.toLowerCase()))
    
    successMessage.value = `成功从进程「${proc.name}」(PID: ${proc.pid}) 中安全卸载 Hook！`
    
    setTimeout(() => {
      successMessage.value = null
    }, 5000)
  } catch (err: any) {
    console.error('卸载 Hook 失败:', err)
    errorMessage.value = err.toString()
  }
}

onMounted(() => {
  fetchProcesses()
})
</script>

<template>
  <div class="nofocus-container">
    <!-- 头部区域 -->
    <div class="page-header">
      <h2>防止窗口失去焦点 (No Focus Loss)</h2>
      <div class="header-actions">
        <button class="btn-secondary" @click="showGuide = !showGuide" :class="{ active: showGuide }">
          <HelpCircle :size="14" />
          <span>杀软信任指引</span>
        </button>
        <button class="btn-primary" @click="fetchProcesses" :disabled="loading">
          <RefreshCw :size="14" :class="{ 'spinning': loading }" />
          <span>刷新进程</span>
        </button>
      </div>
    </div>

    <!-- 醒目防封警告卡片 (轻量 HSL 对齐 DeviceMonitor 风格) -->
    <div class="warning-banner">
      <div class="warning-title">
        <AlertTriangle :size="16" />
        <span>高危安全警告与免责声明 (Ban Risk Warning)</span>
      </div>
      <div class="warning-content">
        <p>1. <strong>封号风险警告</strong>：本功能基于跨进程注入技术（DLL Injection）拦截窗口失活消息。这会被反作弊系统（如 EAC、BattlEye、Vanguard 等）视为外挂注入，<strong>在多人网络游戏或带有反作弊保护的游戏中开启此功能有极高封号风险！</strong></p>
        <p>2. <strong>网络游戏禁用</strong>：<strong>严禁在网络联机游戏中使用此功能</strong>。仅推荐在单机游戏（例如单机挂机、防止切屏暂停/静音、双显屏辅助等）中使用。</p>
        <p>3. <strong>免责说明</strong>：本工具为开源辅助软件，因违反规则或在网游中误用导致的任何损失（包括但不限于账号被封禁、处罚）均由使用者本人承担。</p>
      </div>
    </div>

    <!-- 消息提示栏 -->
    <div v-if="errorMessage" class="message-banner error-banner">
      <XCircle :size="14" />
      <span class="message-text">{{ errorMessage }}</span>
      <button class="close-msg" @click="errorMessage = null">×</button>
    </div>
    
    <div v-if="successMessage" class="message-banner success-banner">
      <CheckCircle :size="14" />
      <span class="message-text">{{ successMessage }}</span>
      <button class="close-msg" @click="successMessage = null">×</button>
    </div>

    <!-- 杀软拦截排除配置引导 (平滑手风琴抽屉) -->
    <Transition name="slide">
      <div v-if="showGuide" class="guide-panel">
        <div class="guide-header">
          <ShieldAlert :size="16" />
          <h4>Windows Defender 杀软拦截修复指引</h4>
        </div>
        <div class="guide-steps">
          <p>由于本功能采用<strong>“物理隔离”</strong>技术（由独立子进程 <code>injector.exe</code> 动态解密并执行注入，彻底避免主程序崩溃或被报毒），Windows Defender 或杀毒软件可能会对 <code>injector.exe</code> 进行警报或拦截。请按照以下步骤添加排除项：</p>
          <ol>
            <li>打开 Windows 的 <strong>「安全中心」</strong> ➔ <strong>「病毒和威胁防护」</strong>。</li>
            <li>点击 <strong>「“病毒和威胁防护”设置」</strong> 下方的 <strong>「管理设置」</strong>。</li>
            <li>向下滑动到 <strong>「排除项」</strong>，点击 <strong>「添加或删除排除项」</strong>。</li>
            <li>点击 <strong>「添加排除项」</strong> ➔ 选择 <strong>「文件」</strong>。</li>
            <li>定位到 AutoController 的安装目录（或开发环境的 target 目录），选中 <code>injector.exe</code> 并点击添加即可。</li>
          </ol>
          <div class="guide-tip">
            <Info :size="12" />
            <span>提示：我们的注入器代码完全开源，绝无后门及恶意行为，您可以放心添加信任运行。</span>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 双面板并排布局 -->
    <div class="panels-grid">
      <!-- 左侧待注入卡片 -->
      <div class="panel-card">
        <div class="panel-header">
          <h3>
            <span>活动窗口进程 (可注入)</span>
            <span class="badge count-badge">{{ filteredProcesses.length }}</span>
          </h3>
          <div class="panel-actions">
            <!-- 搜索过滤 -->
            <div class="search-box">
              <Search :size="14" />
              <input 
                v-model="searchQuery" 
                type="text" 
                placeholder="搜索 PID、进程或窗口标题..." 
              />
            </div>
            <!-- 小刷新按钮 -->
            <button class="btn-icon" @click="fetchProcesses" :disabled="loading" title="刷新列表">
              <RefreshCw :size="14" :class="{ 'spinning': loading }" />
            </button>
          </div>
        </div>
        
        <div class="process-list-container">
          <div v-if="filteredProcesses.length === 0" class="empty-state">
            <Info :size="24" />
            <p>{{ searchQuery ? '未找到符合条件的活动窗口' : '暂无符合附加条件的活动窗口，请点击刷新' }}</p>
          </div>
          
          <div v-else class="process-list">
            <div 
              v-for="proc in filteredProcesses" 
              :key="proc.pid" 
              class="process-item"
            >
              <div class="proc-details">
                <div class="proc-meta">
                  <span class="proc-name">{{ proc.name }}</span>
                  <span class="badge pid-badge">PID: {{ proc.pid }}</span>
                  <span class="badge arch-badge" :class="{ 'x64': proc.is_64bit, 'x86': !proc.is_64bit }">
                    {{ proc.is_64bit ? '64-bit' : '32-bit' }}
                  </span>
                </div>
                <div class="proc-title" :title="proc.window_title">
                  窗口: {{ proc.window_title }}
                </div>
              </div>
              <button class="btn-inject" @click="injectHook(proc)">
                <Zap :size="12" />
                <span>注入 Hook</span>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- 右侧已注入卡片 -->
      <div class="panel-card injected-card">
        <div class="panel-header">
          <h3>
            <span>已附加 Hook 进程 (运行中)</span>
            <span class="badge count-badge active-badge">{{ injectedProcesses.length }}</span>
          </h3>
        </div>

        <div class="process-list-container">
          <div v-if="injectedProcesses.length === 0" class="empty-state">
            <ZapOff :size="24" />
            <p>当前无附加 Hook 状态的进程</p>
            <span class="empty-hint">在左侧列表中选择进程并点击“注入 Hook”</span>
          </div>

          <div v-else class="process-list">
            <div 
              v-for="proc in injectedProcesses" 
              :key="proc.pid" 
              class="process-item injected-item"
            >
              <div class="proc-details">
                <div class="proc-meta">
                  <span class="proc-name">{{ proc.name }}</span>
                  <span class="badge pid-badge">PID: {{ proc.pid }}</span>
                  <span class="badge arch-badge" :class="{ 'x64': proc.is_64bit, 'x86': !proc.is_64bit }">
                    {{ proc.is_64bit ? '64-bit' : '32-bit' }}
                  </span>
                </div>
                <div class="proc-title" :title="proc.window_title">
                  窗口: {{ proc.window_title }}
                </div>
              </div>
              <button class="btn-unload" @click="unloadHook(proc)">
                <ZapOff :size="12" />
                <span>安全卸载</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.nofocus-container {
  padding: var(--space-lg);
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  background-color: var(--color-background);
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-md);
  flex-shrink: 0;
}

.page-header h2 {
  font-size: 18px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: var(--space-sm);
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  background: var(--color-cta);
  color: white;
  padding: 6px 12px;
  border-radius: var(--radius-md);
  font-weight: 500;
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-primary:hover:not(:disabled) {
  background: var(--color-cta-hover);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  background: var(--color-surface);
  color: var(--color-text-muted);
  border: 1px solid var(--color-border);
  padding: 6px 12px;
  border-radius: var(--radius-md);
  font-weight: 500;
  font-size: 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-secondary:hover {
  background: var(--color-surface-elevated);
  color: var(--color-text);
  border-color: var(--color-text-dim);
}

.btn-secondary.active {
  background: rgba(51, 112, 255, 0.08);
  color: var(--color-cta);
  border-color: var(--color-cta);
}

/* 红色醒目警示条 (完美的 Light-Mode 驱动对齐设计) */
.warning-banner {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  padding: var(--space-md);
  border-radius: var(--radius-lg);
  margin-bottom: var(--space-md);
  background: rgba(245, 74, 69, 0.06);
  color: var(--color-error);
  border: 1px solid rgba(245, 74, 69, 0.18);
  flex-shrink: 0;
}

.warning-title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  font-weight: 700;
  font-size: 14px;
}

.warning-content {
  font-size: 12px;
  line-height: 1.6;
  color: #c93b37; /* 稍微偏暗红，确保可读性 */
}

.warning-content p {
  margin-bottom: 4px;
}
.warning-content p:last-child {
  margin-bottom: 0;
}

/* 提示通知消息条 */
.message-banner {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: 10px var(--space-md);
  border-radius: var(--radius-md);
  font-size: 12px;
  margin-bottom: var(--space-md);
  position: relative;
  animation: fadeIn var(--transition-normal);
  flex-shrink: 0;
}

.message-text {
  flex: 1;
  font-weight: 500;
}

.close-msg {
  background: none;
  border: none;
  font-size: 16px;
  font-weight: bold;
  color: inherit;
  cursor: pointer;
  padding: 0 4px;
}

.error-banner {
  background: rgba(245, 74, 69, 0.08);
  color: var(--color-error);
  border: 1px solid rgba(245, 74, 69, 0.15);
}

.success-banner {
  background: rgba(0, 182, 91, 0.08);
  color: var(--color-success);
  border: 1px solid rgba(0, 182, 91, 0.15);
}

/* 信任区指引模块 */
.guide-panel {
  background: rgba(51, 112, 255, 0.04);
  color: var(--color-text-muted);
  border: 1px solid rgba(51, 112, 255, 0.15);
  border-radius: var(--radius-lg);
  padding: var(--space-md);
  margin-bottom: var(--space-md);
  flex-shrink: 0;
}

.guide-header {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  color: var(--color-cta);
  margin-bottom: var(--space-sm);
}

.guide-header h4 {
  margin: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--color-cta);
}

.guide-steps {
  font-size: 12px;
  line-height: 1.6;
}

.guide-steps ol {
  margin-left: 18px;
  margin-top: 6px;
  margin-bottom: 8px;
}

.guide-tip {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  margin-top: 8px;
  color: var(--color-text-dim);
  font-size: 11px;
}

/* 双面板卡片网格 */
.panels-grid {
  display: flex;
  gap: var(--space-md);
  flex: 1;
  min-height: 0;
}

.panel-card {
  flex: 1;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.panel-header {
  padding: var(--space-md);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-md);
  flex-shrink: 0;
}

.panel-header h3 {
  font-size: 13px;
  font-weight: 600;
  margin: 0;
  display: flex;
  align-items: center;
  gap: var(--space-xs);
}

.panel-actions {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-icon:hover:not(:disabled) {
  background: var(--color-surface-elevated);
  color: var(--color-text);
  border-color: var(--color-text-dim);
}

.btn-icon:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.search-box {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: 4px var(--space-sm);
  width: 200px;
  transition: all var(--transition-fast);
}

.search-box:focus-within {
  border-color: var(--color-cta);
  background: var(--color-surface);
}

.search-box input {
  border: none;
  background: transparent;
  font-size: 11px;
  width: 100%;
  color: var(--color-text);
}

.badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: 600;
  border-radius: var(--radius-sm);
  padding: 2px 6px;
}

.count-badge {
  background: var(--color-surface-elevated);
  color: var(--color-text-muted);
}

.active-badge {
  background: rgba(51, 112, 255, 0.1);
  color: var(--color-cta);
}

/* 进程列表容器 */
.process-list-container {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-sm);
}

.process-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.process-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px var(--space-md);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.process-item:hover {
  background: var(--color-surface-elevated);
  border-color: var(--color-text-dim);
}

.injected-item {
  border-left: 3px solid var(--color-cta);
}

.proc-details {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
  margin-right: var(--space-md);
}

.proc-meta {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  flex-wrap: wrap;
}

.proc-name {
  font-weight: 600;
  font-size: 12px;
  color: var(--color-text);
}

.pid-badge {
  background: var(--color-surface-elevated);
  color: var(--color-text-muted);
  font-family: monospace;
}

.arch-badge {
  font-size: 9px;
  padding: 1px 4px;
}

.arch-badge.x64 {
  background: rgba(51, 112, 255, 0.08);
  color: var(--color-cta);
}

.arch-badge.x86 {
  background: var(--color-surface-elevated);
  color: var(--color-text-dim);
}

.proc-title {
  font-size: 11px;
  color: var(--color-text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 320px;
}

/* 操作按钮 */
.btn-inject {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  background: rgba(51, 112, 255, 0.08);
  color: var(--color-cta);
  border: 1px solid rgba(51, 112, 255, 0.2);
  padding: 5px 10px;
  border-radius: var(--radius-md);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-inject:hover {
  background: var(--color-cta);
  color: white;
  border-color: var(--color-cta);
}

.btn-unload {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  background: rgba(245, 74, 69, 0.08);
  color: var(--color-error);
  border: 1px solid rgba(245, 74, 69, 0.2);
  padding: 5px 10px;
  border-radius: var(--radius-md);
  font-size: 11px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-unload:hover {
  background: var(--color-error);
  color: white;
  border-color: var(--color-error);
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--space-2xl) var(--space-md);
  color: var(--color-text-dim);
  text-align: center;
}

.empty-state p {
  font-size: 12px;
  font-weight: 500;
  margin-top: var(--space-sm);
  color: var(--color-text-muted);
}

.empty-hint {
  font-size: 11px;
  color: var(--color-text-dim);
  margin-top: 4px;
}

/* 动效 */
.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  100% {
    transform: rotate(360deg);
  }
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(-4px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Slide 过渡动效 */
.slide-enter-active,
.slide-leave-active {
  transition: all var(--transition-normal);
}
.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
