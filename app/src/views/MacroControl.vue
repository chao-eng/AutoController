<script setup lang="ts">
import { onMounted, ref, onUnmounted } from 'vue'
import { useMacroStore } from '../stores/macro'
import { Circle, Square, Play, Trash2, CheckCircle, AlertTriangle } from '@lucide/vue'
import { invoke } from '@tauri-apps/api/core'
import { Button } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

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
  <div class="p-6 h-full overflow-y-auto">
    <div class="mb-4">
      <h2 class="text-lg font-semibold text-foreground">宏控制</h2>
    </div>

    <!-- 物理手柄自诊断栏 -->
    <div
      v-if="xinputStatus"
      class="flex items-center gap-2 px-3 py-2 rounded-lg text-xs font-medium mb-4 border"
      :class="xinputStatus.available && xinputStatus.connected_devices.length > 0
        ? 'bg-emerald-500/10 text-emerald-500 border-emerald-500/20'
        : xinputStatus.available && xinputStatus.connected_devices.length === 0
          ? 'bg-amber-500/10 text-amber-500 border-amber-500/20'
          : 'bg-destructive/10 text-destructive border-destructive/20'"
    >
      <CheckCircle v-if="xinputStatus.available && xinputStatus.connected_devices.length > 0" :size="14" />
      <AlertTriangle v-else :size="14" />
      <span v-if="!xinputStatus.available">
        XInput 驱动加载失败：{{ xinputStatus.error }}
      </span>
      <span v-else-if="xinputStatus.connected_devices.length === 0">
        未检测到物理 Xbox (XInput) 手柄已连接！
      </span>
      <span v-else>
        物理手柄监听中（通道：{{ xinputStatus.connected_devices.map(i => i + 1).join(', ') }}）
      </span>
    </div>

    <Card class="mb-4">
      <CardContent class="p-4">
        <div class="flex gap-2 items-center">
          <Input
            v-model="recordingName"
            placeholder="宏名称"
            :disabled="store.isRecording"
            class="flex-1"
          />
          <Button
            v-if="!store.isRecording"
            variant="destructive"
            size="sm"
            @click="startRecording"
          >
            <Circle :size="14" fill="currentColor" class="mr-1" />
            <span>开始录制</span>
          </Button>
          <Button
            v-else
            size="sm"
            class="bg-amber-500 hover:bg-amber-500/80 text-white"
            @click="stopRecording"
          >
            <Square :size="14" fill="currentColor" class="mr-1" />
            <span>停止录制</span>
          </Button>
        </div>
      </CardContent>
    </Card>

    <Card class="mb-4">
      <CardContent class="p-4">
        <div class="flex gap-6">
          <div class="flex items-center gap-2 text-xs text-muted-foreground">
            <Label class="min-w-[56px]">回放速度</Label>
            <input type="range" min="0.5" max="2" step="0.1" v-model.number="playbackSpeed" class="w-[120px] accent-primary" />
            <span class="font-heading min-w-[40px] text-xs font-medium text-foreground">{{ (playbackSpeed * 100).toFixed(0) }}%</span>
          </div>
          <div class="flex items-center gap-2 text-xs text-muted-foreground">
            <Label class="min-w-[56px]">循环次数</Label>
            <Input type="number" min="1" max="9999" v-model.number="playbackLoop" class="w-20" />
          </div>
        </div>
      </CardContent>
    </Card>

    <div class="flex flex-col gap-2">
      <div v-if="store.macros.length === 0" class="text-center text-muted-foreground py-12 text-xs">
        暂无宏，点击"开始录制"创建
      </div>
      <div
        v-for="macro in store.macros"
        :key="macro.id"
        class="flex justify-between items-center bg-card border border-border rounded-lg px-4 py-2.5 transition-colors hover:border-primary"
      >
        <div class="flex flex-col gap-0.5">
          <span class="text-sm font-medium text-foreground">{{ macro.name }}</span>
          <span class="text-[11px] text-muted-foreground font-heading">{{ formatDuration(macro.total_duration_ms) }} / {{ macro.event_count }} 事件</span>
        </div>
        <div class="flex gap-1">
          <Button variant="ghost" size="icon-xs" @click="playMacro(macro.id)" title="回放">
            <Play :size="14" />
          </Button>
          <Button variant="ghost" size="icon-xs" class="text-destructive hover:text-destructive hover:bg-destructive/10" @click="deleteMacro(macro.id)" title="删除">
            <Trash2 :size="14" />
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>