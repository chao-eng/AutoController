<script setup lang="ts">
import { onMounted, ref, onUnmounted } from 'vue'
import { useMacroStore } from '../stores/macro'
import { Circle, Square, Play, Trash2, CheckCircle, AlertTriangle } from '@lucide/vue'
import { invoke } from '@tauri-apps/api/core'

interface XInputStatus {
  available: boolean
  error: string | null
  connected_devices: number[]
}

const store = useMacroStore()
const recordingName = ref('新宏')
const playbackSpeed = ref(1.0)
const playbackLoop = ref(1)

const xinputStatus = ref<XInputStatus | null>(null)
let pollTimer: ReturnType<typeof setInterval> | null = null

async function checkXInputStatus() {
  try {
    xinputStatus.value = await invoke<XInputStatus>('macro_xinput_status')
  } catch (e) {
    console.error('获取 XInput 状态失败:', e)
  }
}

onMounted(() => {
  store.fetchMacros()
  checkXInputStatus()
  pollTimer = setInterval(checkXInputStatus, 1000)
})

onUnmounted(() => {
  if (pollTimer) {
    clearInterval(pollTimer)
  }
})

async function startRecording() {
  await store.startRecord('default', recordingName.value)
}

async function stopRecording() {
  await store.stopRecord()
}

async function playMacro(macroId: string) {
  await store.play(macroId, playbackSpeed.value, playbackLoop.value)
}

async function deleteMacro(macroId: string) {
  await store.deleteMacro(macroId)
}

function formatDuration(ms: number): string {
  const s = Math.floor(ms / 1000)
  const m = Math.floor(s / 60)
  const sec = s % 60
  return `${m}:${sec.toString().padStart(2, '0')}`
}
</script>

<template>
  <div class="macro-control">
    <div class="page-header">
      <h2>宏控制</h2>
    </div>

    <div class="record-section">
      <!-- 物理手柄自诊断栏 -->
      <div v-if="xinputStatus" class="xinput-diagnostic-bar" :class="{ ok: xinputStatus.available && xinputStatus.connected_devices.length > 0, warn: xinputStatus.available && xinputStatus.connected_devices.length === 0, err: !xinputStatus.available }">
        <CheckCircle v-if="xinputStatus.available && xinputStatus.connected_devices.length > 0" :size="14" />
        <AlertTriangle v-else :size="14" />
        <span v-if="!xinputStatus.available" class="diagnostic-text">
          XInput 驱动加载失败：{{ xinputStatus.error }}
        </span>
        <span v-else-if="xinputStatus.connected_devices.length === 0" class="diagnostic-text">
          未检测到物理 Xbox (XInput) 手柄已连接！
        </span>
        <span v-else class="diagnostic-text">
          物理手柄监听中（通道：{{ xinputStatus.connected_devices.map(i => i + 1).join(', ') }}）
        </span>
      </div>

      <div class="record-controls">
        <input
          v-model="recordingName"
          class="input"
          placeholder="宏名称"
          :disabled="store.isRecording"
        />
        <button
          v-if="!store.isRecording"
          class="btn-record"
          @click="startRecording"
        >
          <Circle :size="14" fill="currentColor" />
          <span>开始录制</span>
        </button>
        <button
          v-else
          class="btn-stop"
          @click="stopRecording"
        >
          <Square :size="14" fill="currentColor" />
          <span>停止录制</span>
        </button>
      </div>
    </div>

    <div class="playback-settings">
      <div class="setting-item">
        <label>回放速度</label>
        <input type="range" min="0.5" max="2" step="0.1" v-model.number="playbackSpeed" />
        <span class="setting-value">{{ (playbackSpeed * 100).toFixed(0) }}%</span>
      </div>
      <div class="setting-item">
        <label>循环次数</label>
        <input type="number" min="1" max="9999" v-model.number="playbackLoop" class="input small" />
      </div>
    </div>

    <div class="macro-list">
      <div v-if="store.macros.length === 0" class="empty-state">
        暂无宏，点击"开始录制"创建
      </div>
      <div
        v-for="macro in store.macros"
        :key="macro.id"
        class="macro-item"
      >
        <div class="macro-info">
          <span class="macro-name">{{ macro.name }}</span>
          <span class="macro-meta">{{ formatDuration(macro.total_duration_ms) }} / {{ macro.event_count }} 事件</span>
        </div>
        <div class="macro-actions">
          <button class="icon-btn" @click="playMacro(macro.id)" title="回放">
            <Play :size="14" />
          </button>
          <button class="icon-btn danger" @click="deleteMacro(macro.id)" title="删除">
            <Trash2 :size="14" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.macro-control {
  padding: var(--space-lg);
  height: 100%;
  overflow-y: auto;
}

.page-header {
  margin-bottom: var(--space-lg);
}

.page-header h2 {
  font-size: 18px;
  font-weight: 600;
}

.record-section {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-md);
  margin-bottom: var(--space-lg);
}

.record-controls {
  display: flex;
  gap: var(--space-sm);
  align-items: center;
}

.input {
  flex: 1;
  padding: var(--space-sm) var(--space-md);
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-size: 13px;
  transition: border-color var(--transition-fast);
}

.input:focus {
  border-color: var(--color-cta);
}

.input.small {
  width: 80px;
  flex: none;
}

.btn-record {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  background: var(--color-error);
  color: white;
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-record:hover {
  opacity: 0.9;
}

.btn-stop {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
  background: var(--color-warning);
  color: var(--color-primary);
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-stop:hover {
  opacity: 0.9;
}

.playback-settings {
  display: flex;
  gap: var(--space-lg);
  margin-bottom: var(--space-lg);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-md);
}

.setting-item {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  font-size: 12px;
  color: var(--color-text-muted);
}

.setting-item label {
  min-width: 56px;
}

.setting-item input[type="range"] {
  width: 120px;
  accent-color: var(--color-cta);
}

.setting-value {
  font-family: var(--font-heading);
  min-width: 40px;
}

.macro-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
}

.empty-state {
  text-align: center;
  color: var(--color-text-dim);
  padding: var(--space-2xl);
  font-size: 13px;
}

.macro-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  padding: var(--space-sm) var(--space-md);
  transition: border-color var(--transition-fast);
}

.macro-item:hover {
  border-color: var(--color-cta);
}

.macro-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.macro-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text);
}

.macro-meta {
  font-size: 11px;
  color: var(--color-text-dim);
  font-family: var(--font-heading);
}

.macro-actions {
  display: flex;
  gap: var(--space-xs);
}

.icon-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  color: var(--color-text-dim);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.icon-btn:hover {
  background: var(--color-surface-elevated);
  color: var(--color-text);
}

.icon-btn.danger:hover {
  background: rgba(239, 68, 68, 0.15);
  color: var(--color-error);
}

.xinput-diagnostic-bar {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: 8px 12px;
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  margin-bottom: var(--space-md);
  border: 1px solid transparent;
}

.xinput-diagnostic-bar.ok {
  background: rgba(34, 197, 94, 0.1);
  color: var(--color-cta);
  border-color: rgba(34, 197, 94, 0.2);
}

.xinput-diagnostic-bar.warn {
  background: rgba(234, 179, 8, 0.1);
  color: #eab308;
  border-color: rgba(234, 179, 8, 0.2);
}

.xinput-diagnostic-bar.err {
  background: rgba(239, 68, 68, 0.1);
  color: var(--color-error);
  border-color: rgba(239, 68, 68, 0.2);
}

.diagnostic-text {
  font-family: var(--font-heading);
}
</style>
