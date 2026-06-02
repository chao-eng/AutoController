<script setup lang="ts">
import { onMounted, onUnmounted, computed, ref, watch } from 'vue'
import { useControllerStore } from '../stores/controller'
import { useUIStore } from '../stores/ui'
import DeviceCard from '../components/controller/DeviceCard.vue'
import StickVisualizer from '../components/controller/StickVisualizer.vue'
import TriggerBar from '../components/controller/TriggerBar.vue'
import { Plus, AlertTriangle, CheckCircle, AlertCircle, Activity } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle, CardFooter, CardDescription } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'

const store = useControllerStore()
const uiStore = useUIStore()
const selectedDeviceId = ref<string | null>(null)
const reconnecting = ref(false)

async function handleReconnect() {
  try {
    reconnecting.value = true
    await store.reconnectViGEm()
    uiStore.showToast('🎉 ViGEmBus 内核驱动热重连成功，已激活虚拟设备！', 'success')
  } catch (err) {
    uiStore.showAlert('重连失败', `重连 ViGEmBus 驱动失败，请确认已正确安装驱动。错误: ${err}`)
  } finally {
    reconnecting.value = false
  }
}

// Dragging interaction states
const isDraggingLeft = ref(false)
const isDraggingRight = ref(false)
const isDraggingLT = ref(false)
const isDraggingRT = ref(false)

onMounted(() => {
  store.init()
})

onUnmounted(() => {
  store.cleanup()
})

async function addDevice() {
  await store.createDevice()
}

function selectDevice(id: string) {
  selectedDeviceId.value = id
}

const selectedDevice = () => {
  return store.devices.find((d) => d.id === selectedDeviceId.value)
}

const BTN = {
  A: 0x1000,
  B: 0x2000,
  X: 0x4000,
  Y: 0x8000,
  LB: 0x0100,
  RB: 0x0200,
  BACK: 0x0020,
  START: 0x0010,
  GUIDE: 0x0400,
  LS: 0x0040,
  RS: 0x0080,
  UP: 0x0001,
  DOWN: 0x0002,
  LEFT: 0x0004,
  RIGHT: 0x0008,
} as const

function isPressed(buttons: number, flag: number): boolean {
  return (buttons & flag) !== 0
}

const vigemSuggestion = computed(() => {
  const status = store.vigemStatus
  if (!status || status.connected) return null
  if (!status.dll_found) {
    return '请将 ViGEmClient.dll 放到程序目录下，或安装 ViGEmBus 驱动'
  }
  switch (status.error_code) {
    case 1:
      return '请前往 https://github.com/nefarius/ViGEmBus/releases 下载安装 ViGEmBus 驱动，安装后重启电脑'
    case 2:
      return '请右键程序，选择「以管理员身份运行」'
    default:
      return '请确认 ViGEmBus 驱动已正确安装，且版本与 ViGEmClient.dll 匹配'
  }
})

// Mapped state values for selected device
const activeDevice = computed(() => selectedDevice())

const isBtnA = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.A) : false)
const isBtnB = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.B) : false)
const isBtnX = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.X) : false)
const isBtnY = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.Y) : false)
const isBtnLB = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.LB) : false)
const isBtnRB = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.RB) : false)
const isBtnBack = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.BACK) : false)
const isBtnStart = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.START) : false)
const isBtnGuide = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.GUIDE) : false)
const isBtnLS = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.LS) : false)
const isBtnRS = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.RS) : false)

const isDpadUp = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.UP) : false)
const isDpadDown = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.DOWN) : false)
const isDpadLeft = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.LEFT) : false)
const isDpadRight = computed(() => activeDevice.value ? isPressed(activeDevice.value.state.buttons, BTN.RIGHT) : false)

const leftStickOffset = computed(() => {
  if (!activeDevice.value) return { x: 0, y: 0 }
  return {
    x: (activeDevice.value.state.left_thumb_x / 32767) * 12,
    y: -(activeDevice.value.state.left_thumb_y / 32767) * 12
  }
})

const rightStickOffset = computed(() => {
  if (!activeDevice.value) return { x: 0, y: 0 }
  return {
    x: (activeDevice.value.state.right_thumb_x / 32767) * 12,
    y: -(activeDevice.value.state.right_thumb_y / 32767) * 12
  }
})

const leftTriggerVal = computed(() => {
  if (!activeDevice.value) return 0
  return activeDevice.value.state.left_trigger
})

const rightTriggerVal = computed(() => {
  if (!activeDevice.value) return 0
  return activeDevice.value.state.right_trigger
})

// Left stick drag handler
function handleLeftStickPointerDown(e: PointerEvent) {
  if (!activeDevice.value || !activeDevice.value.connected) return
  isDraggingLeft.value = true
  const target = e.currentTarget as SVGElement
  target.setPointerCapture(e.pointerId)
  updateLeftStick(e)
}

function handleLeftStickPointerMove(e: PointerEvent) {
  if (!isDraggingLeft.value) return
  updateLeftStick(e)
}

function handleLeftStickPointerUp(e: PointerEvent) {
  if (!isDraggingLeft.value) return
  isDraggingLeft.value = false
  const target = e.currentTarget as SVGElement
  try {
    target.releasePointerCapture(e.pointerId)
  } catch (err) {}
  store.setThumb(selectedDeviceId.value!, 'LeftX', 0)
  store.setThumb(selectedDeviceId.value!, 'LeftY', 0)
}

function updateLeftStick(e: PointerEvent) {
  const rect = (e.currentTarget as SVGElement).getBoundingClientRect()
  const centerClientX = rect.left + (113 / 441) * rect.width
  const centerClientY = rect.top + (160 / 383) * rect.height
  
  const dx = e.clientX - centerClientX
  const dy = -(e.clientY - centerClientY)
  
  const maxDistClient = (37.5 / 441) * rect.width
  const dist = Math.sqrt(dx * dx + dy * dy)
  const ratio = dist > maxDistClient ? maxDistClient / dist : 1
  
  const x = (dx * ratio) / maxDistClient
  const y = (dy * ratio) / maxDistClient
  
  const deadzone = 0.05
  const finalX = Math.abs(x) < deadzone ? 0 : Math.round(x * 1000) / 1000
  const finalY = Math.abs(y) < deadzone ? 0 : Math.round(y * 1000) / 1000
  
  store.setThumb(selectedDeviceId.value!, 'LeftX', finalX)
  store.setThumb(selectedDeviceId.value!, 'LeftY', finalY)
}

// Right stick drag handler
function handleRightStickPointerDown(e: PointerEvent) {
  if (!activeDevice.value || !activeDevice.value.connected) return
  isDraggingRight.value = true
  const target = e.currentTarget as SVGElement
  target.setPointerCapture(e.pointerId)
  updateRightStick(e)
}

function handleRightStickPointerMove(e: PointerEvent) {
  if (!isDraggingRight.value) return
  updateRightStick(e)
}

function handleRightStickPointerUp(e: PointerEvent) {
  if (!isDraggingRight.value) return
  isDraggingRight.value = false
  const target = e.currentTarget as SVGElement
  try {
    target.releasePointerCapture(e.pointerId)
  } catch (err) {}
  store.setThumb(selectedDeviceId.value!, 'RightX', 0)
  store.setThumb(selectedDeviceId.value!, 'RightY', 0)
}

function updateRightStick(e: PointerEvent) {
  const rect = (e.currentTarget as SVGElement).getBoundingClientRect()
  const centerClientX = rect.left + (278 / 441) * rect.width
  const centerClientY = rect.top + (238 / 383) * rect.height
  
  const dx = e.clientX - centerClientX
  const dy = -(e.clientY - centerClientY)
  
  const maxDistClient = (37.5 / 441) * rect.width
  const dist = Math.sqrt(dx * dx + dy * dy)
  const ratio = dist > maxDistClient ? maxDistClient / dist : 1
  
  const x = (dx * ratio) / maxDistClient
  const y = (dy * ratio) / maxDistClient
  
  const deadzone = 0.05
  const finalX = Math.abs(x) < deadzone ? 0 : Math.round(x * 1000) / 1000
  const finalY = Math.abs(y) < deadzone ? 0 : Math.round(y * 1000) / 1000
  
  store.setThumb(selectedDeviceId.value!, 'RightX', finalX)
  store.setThumb(selectedDeviceId.value!, 'RightY', finalY)
}

// LT / RT drag handlers
function handleLTPointerDown(e: PointerEvent) {
  if (!activeDevice.value || !activeDevice.value.connected) return
  isDraggingLT.value = true
  const target = e.currentTarget as SVGElement
  target.setPointerCapture(e.pointerId)
  updateLT(e)
}

function handleLTPointerMove(e: PointerEvent) {
  if (!isDraggingLT.value) return
  updateLT(e)
}

function handleLTPointerUp(e: PointerEvent) {
  if (!isDraggingLT.value) return
  isDraggingLT.value = false
  const target = e.currentTarget as SVGElement
  try {
    target.releasePointerCapture(e.pointerId)
  } catch (err) {}
  store.setTrigger(selectedDeviceId.value!, 'Left', 0)
}

function updateLT(e: PointerEvent) {
  const rect = (e.currentTarget as SVGElement).getBoundingClientRect()
  const y = (e.clientY - rect.top) / rect.height
  const val = Math.max(0, Math.min(1, 1 - y))
  store.setTrigger(selectedDeviceId.value!, 'Left', val)
}

function handleRTPointerDown(e: PointerEvent) {
  if (!activeDevice.value || !activeDevice.value.connected) return
  isDraggingRT.value = true
  const target = e.currentTarget as SVGElement
  target.setPointerCapture(e.pointerId)
  updateRT(e)
}

function handleRTPointerMove(e: PointerEvent) {
  if (!isDraggingRT.value) return
  updateRT(e)
}

function handleRTPointerUp(e: PointerEvent) {
  if (!isDraggingRT.value) return
  isDraggingRT.value = false
  const target = e.currentTarget as SVGElement
  try {
    target.releasePointerCapture(e.pointerId)
  } catch (err) {}
  store.setTrigger(selectedDeviceId.value!, 'Right', 0)
}

function updateRT(e: PointerEvent) {
  const rect = (e.currentTarget as SVGElement).getBoundingClientRect()
  const y = (e.clientY - rect.top) / rect.height
  const val = Math.max(0, Math.min(1, 1 - y))
  store.setTrigger(selectedDeviceId.value!, 'Right', val)
}

// Watch device list to auto-select first device
watch(
  () => store.devices,
  (list) => {
    if (list.length > 0 && !selectedDeviceId.value) {
      selectedDeviceId.value = list[0].id
    }
  },
  { immediate: true }
)
</script>

<template>
  <div class="flex h-full flex-col gap-4 overflow-y-auto p-4 lg:p-6">
    <div class="flex items-center justify-between">
      <h2 class="text-lg font-semibold">设备监控</h2>
      <Button variant="default" size="sm" @click="addDevice">
        <Plus :size="16" />
        <span>添加设备</span>
      </Button>
    </div>

    <div v-if="store.vigemStatus" :class="[
      'flex items-center justify-between gap-2 rounded-md px-4 py-2 text-xs',
      store.vigemStatus.connected
        ? 'border border-green-500/20 bg-green-500/10 text-green-600 dark:text-green-400'
        : 'border border-orange-500/20 bg-orange-500/10 text-orange-600 dark:text-orange-400'
    ]">
      <div class="flex items-center gap-2">
        <CheckCircle v-if="store.vigemStatus.connected" :size="14" />
        <AlertTriangle v-else :size="14" />
        <span>{{ store.vigemStatus.message }}</span>
      </div>
      <Button v-if="!store.vigemStatus.connected" variant="outline" size="xs" :disabled="reconnecting" @click="handleReconnect">
        {{ reconnecting ? '正在连接...' : '🔄 尝试热重连并激活驱动' }}
      </Button>
    </div>

    <div v-if="vigemSuggestion" class="flex items-center gap-2 rounded-md border border-blue-500/20 bg-blue-500/10 px-4 py-2 text-xs text-blue-600 dark:text-blue-400">
      <AlertCircle :size="14" />
      <span>{{ vigemSuggestion }}</span>
    </div>

    <div class="grid gap-4" style="grid-template-columns: repeat(auto-fill, minmax(220px, 1fr))">
      <DeviceCard
        v-for="device in store.devices"
        :key="device.id"
        :device="device"
        :class="[selectedDeviceId === device.id ? 'ring-2 ring-primary' : '']"
        @click="selectDevice(device.id)"
        @remove="store.removeDevice"
        @toggle-connection="store.toggleConnection"
      />
      <button
        v-if="store.devices.length < 8"
        class="flex min-h-[140px] cursor-pointer flex-col items-center justify-center gap-2 rounded-xl border-2 border-dashed border-border text-xs text-muted-foreground transition-all duration-300 hover:border-primary hover:bg-primary/[0.02] hover:text-primary"
        @click="addDevice"
      >
        <Plus :size="24" />
        <span>添加设备</span>
      </button>
    </div>

    <!-- Active Virtual Controller Input Dashboard -->
    <div v-if="selectedDevice()">
      <Card :class="[!selectedDevice()!.connected ? 'opacity-45 pointer-events-none' : '']">
        <CardHeader class="flex flex-row items-center justify-between border-b border-border pb-4">
          <div class="flex items-center gap-2">
            <Activity :size="16" class="text-indigo-500" />
            <CardTitle class="text-sm">输入控制 — 虚拟手柄 #{{ selectedDevice()!.index }}</CardTitle>
          </div>
          <Badge v-if="!selectedDevice()!.connected" variant="outline" class="text-muted-foreground">已断开连接（保留在列表）</Badge>
        </CardHeader>

        <CardContent class="p-0 lg:p-0">
          <div class="grid grid-cols-1 gap-4 p-4 lg:grid-cols-[1.2fr_1fr] lg:gap-6">
            <!-- Left Side: Interactive SVG Controller -->
            <div class="flex flex-col">
              <Card class="flex min-h-[420px] flex-col items-center justify-between border border-border p-4 lg:p-6">
                <div class="mx-auto w-full max-w-[440px]">
                  <svg viewBox="0 0 441 383" class="h-auto w-full drop-shadow-lg transition-transform duration-100">
                    <g id="XboxController">
                      <!-- Left main shell -->
                      <path d="M220.5 294.5C220.5 294.5 195 294.5 150 294.5C105 294.5 81.5 378.5 49.5 378.5C17.5 378.5 4 363.9 4 317.5C4 271.1 43.5 165.5 55 137.5C66.5 109.5 95.5 92.0001 128 92.0001C154 92.0001 200.5 92.0001 220.5 92.0001" class="fill-[var(--color-surface)] stroke-[var(--color-border)]" stroke-width="3" />
                      <!-- Right main shell -->
                      <path d="M220 294.5C220 294.5 245.5 294.5 290.5 294.5C335.5 294.5 359 378.5 391 378.5C423 378.5 436.5 363.9 436.5 317.5C436.5 271.1 397 165.5 385.5 137.5C374 109.5 345 92.0001 312.5 92.0001C286.5 92.0001 240 92.0001 220 92.0001" class="fill-[var(--color-surface)] stroke-[var(--color-border)]" stroke-width="3" />
                      
                      <!-- Left Trigger (LT) - Interactive drag & depth fill -->
                      <path 
                        @pointerdown="handleLTPointerDown"
                        @pointermove="handleLTPointerMove"
                        @pointerup="handleLTPointerUp"
                        @pointercancel="handleLTPointerUp"
                        d="m152.5,52.97c0,4.61 -3.35,8.36 -7.5,8.36l-13,0c-4.14,0 -7.5,-3.74 -7.5,-8.36l0,-22.86c0,-8.62 6.27,-15.61 14,-15.61c7.73,0 14,6.99 14,15.61l0,22.86z" 
                        :style="{ 
                          fill: `rgba(99, 102, 241, ${0.15 + (leftTriggerVal / 255) * 0.85})`,
                          stroke: leftTriggerVal > 0 ? '#6366f1' : 'var(--color-border)',
                          strokeWidth: '2px'
                        }"
                        class="cursor-ns-resize transition-colors duration-100" 
                      />
                      
                      <!-- Right Trigger (RT) - Interactive drag -->
                      <path 
                        @pointerdown="handleRTPointerDown"
                        @pointermove="handleRTPointerMove"
                        @pointerup="handleRTPointerUp"
                        @pointercancel="handleRTPointerUp"
                        d="m316.83,53.44c0,4.64 -3.44,8.39 -7.68,8.39l-13.31,0c-4.24,0 -7.68,-3.76 -7.68,-8.39l0,-22.94c0,-8.65 6.42,-15.67 14.33,-15.67c7.92,0 14.33,7.01 14.33,15.67l0,22.94z" 
                        :style="{ 
                          fill: `rgba(99, 102, 241, ${0.15 + (rightTriggerVal / 255) * 0.85})`,
                          stroke: rightTriggerVal > 0 ? '#6366f1' : 'var(--color-border)',
                          strokeWidth: '2px'
                        }"
                        class="cursor-ns-resize transition-colors duration-100" 
                      />

                      <!-- Left Bumper (LB) -->
                      <g 
                        @mousedown="store.setButton(selectedDeviceId!, 'LB', true)"
                        @mouseup="store.setButton(selectedDeviceId!, 'LB', false)"
                        @mouseleave="store.setButton(selectedDeviceId!, 'LB', false)"
                        @touchstart.prevent="store.setButton(selectedDeviceId!, 'LB', true)"
                        @touchend.prevent="store.setButton(selectedDeviceId!, 'LB', false)"
                        class="cursor-pointer"
                      >
                        <rect 
                          x="116.8" y="66.8" width="43.3" height="17" rx="4" 
                          :style="{ 
                            fill: isBtnLB ? '#6366f1' : 'var(--color-surface-elevated)', 
                            stroke: isBtnLB ? '#4f46e5' : 'var(--color-border)',
                            strokeWidth: '2px'
                          }" 
                          class="transition-all duration-200" 
                        />
                        <text 
                          x="138.5" y="79" text-anchor="middle" 
                          :style="{ fill: isBtnLB ? '#ffffff' : 'var(--color-text-muted)' }" 
                          class="pointer-events-none text-[10px] font-bold transition-colors duration-200"
                        >LB</text>
                      </g>

                      <!-- Right Bumper (RB) -->
                      <g 
                        @mousedown="store.setButton(selectedDeviceId!, 'RB', true)"
                        @mouseup="store.setButton(selectedDeviceId!, 'RB', false)"
                        @mouseleave="store.setButton(selectedDeviceId!, 'RB', false)"
                        @touchstart.prevent="store.setButton(selectedDeviceId!, 'RB', true)"
                        @touchend.prevent="store.setButton(selectedDeviceId!, 'RB', false)"
                        class="cursor-pointer"
                      >
                        <rect 
                          x="281.3" y="67" width="42.6" height="17" rx="4" 
                          :style="{ 
                            fill: isBtnRB ? '#6366f1' : 'var(--color-surface-elevated)', 
                            stroke: isBtnRB ? '#4f46e5' : 'var(--color-border)',
                            strokeWidth: '2px'
                          }" 
                          class="transition-all duration-200" 
                        />
                        <text 
                          x="302.5" y="79" text-anchor="middle" 
                          :style="{ fill: isBtnRB ? '#ffffff' : 'var(--color-text-muted)' }" 
                          class="pointer-events-none text-[10px] font-bold transition-colors duration-200"
                        >RB</text>
                      </g>

                      <!-- Left Stick well and interactive pointer drag knob -->
                      <g 
                        @pointerdown="handleLeftStickPointerDown" 
                        @pointermove="handleLeftStickPointerMove" 
                        @pointerup="handleLeftStickPointerUp" 
                        @pointercancel="handleLeftStickPointerUp"
                        class="select-none"
                      >
                        <circle cx="113" cy="160" r="37.5" class="fill-[var(--color-background)] stroke-[var(--color-border)]" stroke-width="2" />
                        <g :transform="'translate(' + leftStickOffset.x + ', ' + leftStickOffset.y + ')'" class="transition-transform duration-75">
                          <circle 
                            cx="113" cy="160" r="28" 
                            :style="{ 
                              fill: isBtnLS ? '#6366f1' : 'var(--color-text-dim)', 
                              stroke: isBtnLS ? '#4f46e5' : 'var(--color-surface)',
                              strokeWidth: '2px'
                            }" 
                            class="transition-colors duration-100" 
                          />
                          <circle cx="113" cy="160" r="22"  stroke-width="2" />
                          <circle cx="113" cy="160" r="10"  />
                        </g>
                        <circle cx="113" cy="160" r="42" fill="transparent" class="cursor-grab" />
                      </g>

                      <!-- Right Stick well and interactive pointer drag knob -->
                      <g 
                        @pointerdown="handleRightStickPointerDown" 
                        @pointermove="handleRightStickPointerMove" 
                        @pointerup="handleRightStickPointerUp" 
                        @pointercancel="handleRightStickPointerUp"
                        class="select-none"
                      >
                        <circle cx="278" cy="238" r="37.5" class="fill-[var(--color-background)] stroke-[var(--color-border)]" stroke-width="2" />
                        <g :transform="'translate(' + rightStickOffset.x + ', ' + rightStickOffset.y + ')'" class="transition-transform duration-75">
                          <circle 
                            cx="278" cy="238" r="28" 
                            :style="{ 
                              fill: isBtnRS ? '#6366f1' : 'var(--color-text-dim)', 
                              stroke: isBtnRS ? '#4f46e5' : 'var(--color-surface)',
                              strokeWidth: '2px'
                            }" 
                            class="transition-colors duration-100" 
                          />
                          <circle cx="278" cy="238" r="22" class="stroke-black/5 fill-none dark:stroke-white/5" stroke-width="2" />
                          <circle cx="278" cy="238" r="10"  />
                        </g>
                        <circle cx="278" cy="238" r="42" fill="transparent" class="cursor-grab" />
                      </g>

                      <!-- D-pad well and four directional rects -->
                      <circle cx="166" cy="238" r="37.5" class="fill-[var(--color-background)] stroke-[var(--color-border)]" stroke-width="2" />
                      
                      <rect 
                        @mousedown="store.setButton(selectedDeviceId!, 'DPadUp', true)"
                        @mouseup="store.setButton(selectedDeviceId!, 'DPadUp', false)"
                        @mouseleave="store.setButton(selectedDeviceId!, 'DPadUp', false)"
                        @touchstart.prevent="store.setButton(selectedDeviceId!, 'DPadUp', true)"
                        @touchend.prevent="store.setButton(selectedDeviceId!, 'DPadUp', false)"
                        x="159" y="211" width="14" height="20" 
                        :style="{ 
                          fill: isDpadUp ? '#6366f1' : 'var(--color-surface)',
                          stroke: 'var(--color-border)',
                          strokeWidth: '1.5px'
                        }" 
                        class="cursor-pointer transition-all duration-100" 
                      />
                      <rect 
                        @mousedown="store.setButton(selectedDeviceId!, 'DPadDown', true)"
                        @mouseup="store.setButton(selectedDeviceId!, 'DPadDown', false)"
                        @mouseleave="store.setButton(selectedDeviceId!, 'DPadDown', false)"
                        @touchstart.prevent="store.setButton(selectedDeviceId!, 'DPadDown', true)"
                        @touchend.prevent="store.setButton(selectedDeviceId!, 'DPadDown', false)"
                        x="159" y="239" width="14" height="20" 
                        :style="{ 
                          fill: isDpadDown ? '#6366f1' : 'var(--color-surface)',
                          stroke: 'var(--color-border)',
                          strokeWidth: '1.5px'
                        }" 
                        class="cursor-pointer transition-all duration-100" 
                      />
                      <rect 
                        @mousedown="store.setButton(selectedDeviceId!, 'DPadLeft', true)"
                        @mouseup="store.setButton(selectedDeviceId!, 'DPadLeft', false)"
                        @mouseleave="store.setButton(selectedDeviceId!, 'DPadLeft', false)"
                        @touchstart.prevent="store.setButton(selectedDeviceId!, 'DPadLeft', true)"
                        @touchend.prevent="store.setButton(selectedDeviceId!, 'DPadLeft', false)"
                        x="142" y="228" width="20" height="14" 
                        :style="{ 
                          fill: isDpadLeft ? '#6366f1' : 'var(--color-surface)',
                          stroke: 'var(--color-border)',
                          strokeWidth: '1.5px'
                        }" 
                        class="cursor-pointer transition-all duration-100" 
                      />
                      <rect 
                        @mousedown="store.setButton(selectedDeviceId!, 'DPadRight', true)"
                        @mouseup="store.setButton(selectedDeviceId!, 'DPadRight', false)"
                        @mouseleave="store.setButton(selectedDeviceId!, 'DPadRight', false)"
                        @touchstart.prevent="store.setButton(selectedDeviceId!, 'DPadRight', true)"
                        @touchend.prevent="store.setButton(selectedDeviceId!, 'DPadRight', false)"
                        x="170" y="228" width="20" height="14" 
                        :style="{ 
                          fill: isDpadRight ? '#6366f1' : 'var(--color-surface)',
                          stroke: 'var(--color-border)',
                          strokeWidth: '1.5px'
                        }" 
                        class="cursor-pointer transition-all duration-100" 
                      />
                      <!-- Dpad Center -->
                      <rect 
                        x="159" y="228" width="14" height="14" 
                        :style="{ 
                          fill: 'var(--color-surface)',
                          stroke: 'var(--color-border)',
                          strokeWidth: '1.5px'
                        }" 
                      />

                       <!-- ABXY well -->
                      <circle cx="329" cy="160" r="42" class="fill-[var(--color-background)] stroke-[var(--color-border)]" stroke-width="2" />
                      
                      <!-- Y Button (Yellow) -->
                      <g 
                        @mousedown="store.setButton(selectedDeviceId!, 'Y', true)"
                        @mouseup="store.setButton(selectedDeviceId!, 'Y', false)"
                        @mouseleave="store.setButton(selectedDeviceId!, 'Y', false)"
                        @touchstart.prevent="store.setButton(selectedDeviceId!, 'Y', true)"
                        @touchend.prevent="store.setButton(selectedDeviceId!, 'Y', false)"
                        class="cursor-pointer"
                      >
                        <circle 
                          cx="329" cy="136" r="13" 
                          :style="{ 
                            fill: isBtnY ? '#f59e0b' : 'rgba(245, 158, 11, 0.18)', 
                            stroke: '#f59e0b', 
                            strokeWidth: '2px' 
                          }" 
                          class="transition-all duration-100" 
                        />
                        <text 
                          x="329" y="140" text-anchor="middle" 
                          :style="{ fill: isBtnY ? '#ffffff' : '#f59e0b' }" 
                          class="pointer-events-none text-[12px] font-black transition-colors duration-100"
                        >Y</text>
                      </g>

                      <!-- X Button (Blue) -->
                      <g 
                        @mousedown="store.setButton(selectedDeviceId!, 'X', true)"
                        @mouseup="store.setButton(selectedDeviceId!, 'X', false)"
                        @mouseleave="store.setButton(selectedDeviceId!, 'X', false)"
                        @touchstart.prevent="store.setButton(selectedDeviceId!, 'X', true)"
                        @touchend.prevent="store.setButton(selectedDeviceId!, 'X', false)"
                        class="cursor-pointer"
                      >
                        <circle 
                          cx="305" cy="160" r="13" 
                          :style="{ 
                            fill: isBtnX ? '#3b82f6' : 'rgba(59, 130, 246, 0.18)', 
                            stroke: '#3b82f6', 
                            strokeWidth: '2px' 
                          }" 
                          class="transition-all duration-100" 
                        />
                        <text 
                          x="305" y="164" text-anchor="middle" 
                          :style="{ fill: isBtnX ? '#ffffff' : '#3b82f6' }" 
                          class="pointer-events-none text-[12px] font-black transition-colors duration-100"
                        >X</text>
                      </g>

                      <!-- B Button (Red) -->
                      <g 
                        @mousedown="store.setButton(selectedDeviceId!, 'B', true)"
                        @mouseup="store.setButton(selectedDeviceId!, 'B', false)"
                        @mouseleave="store.setButton(selectedDeviceId!, 'B', false)"
                        @touchstart.prevent="store.setButton(selectedDeviceId!, 'B', true)"
                        @touchend.prevent="store.setButton(selectedDeviceId!, 'B', false)"
                        class="cursor-pointer"
                      >
                        <circle 
                          cx="353" cy="160" r="13" 
                          :style="{ 
                            fill: isBtnB ? '#ef4444' : 'rgba(239, 68, 68, 0.18)', 
                            stroke: '#ef4444', 
                            strokeWidth: '2px' 
                          }" 
                          class="transition-all duration-100" 
                        />
                        <text 
                          x="353" y="164" text-anchor="middle" 
                          :style="{ fill: isBtnB ? '#ffffff' : '#ef4444' }" 
                          class="pointer-events-none text-[12px] font-black transition-colors duration-100"
                        >B</text>
                      </g>

                      <!-- A Button (Green) -->
                      <g 
                        @mousedown="store.setButton(selectedDeviceId!, 'A', true)"
                        @mouseup="store.setButton(selectedDeviceId!, 'A', false)"
                        @mouseleave="store.setButton(selectedDeviceId!, 'A', false)"
                        @touchstart.prevent="store.setButton(selectedDeviceId!, 'A', true)"
                        @touchend.prevent="store.setButton(selectedDeviceId!, 'A', false)"
                        class="cursor-pointer"
                      >
                        <circle 
                          cx="329" cy="184" r="13" 
                          :style="{ 
                            fill: isBtnA ? '#10b981' : 'rgba(16, 185, 129, 0.18)', 
                            stroke: '#10b981', 
                            strokeWidth: '2px' 
                          }" 
                          class="transition-all duration-100" 
                        />
                        <text 
                          x="329" y="188" text-anchor="middle" 
                          :style="{ fill: isBtnA ? '#ffffff' : '#10b981' }" 
                          class="pointer-events-none text-[12px] font-black transition-colors duration-100"
                        >A</text>
                      </g>

                      <!-- View/Back Button -->
                      <g 
                        @mousedown="store.setButton(selectedDeviceId!, 'Back', true)"
                        @mouseup="store.setButton(selectedDeviceId!, 'Back', false)"
                        @mouseleave="store.setButton(selectedDeviceId!, 'Back', false)"
                        @touchstart.prevent="store.setButton(selectedDeviceId!, 'Back', true)"
                        @touchend.prevent="store.setButton(selectedDeviceId!, 'Back', false)"
                        class="cursor-pointer"
                      >
                        <circle cx="188" cy="162" r="10" :style="{ fill: isBtnBack ? '#6366f1' : 'var(--color-surface-elevated)' }" class="transition-all duration-200" />
                        <rect x="184" y="158" width="8" height="8" rx="1" fill="none" stroke="currentColor" stroke-width="1.5" :style="{ stroke: isBtnBack ? '#ffffff' : 'var(--color-text-muted)' }" class="pointer-events-none transition-colors duration-200" />
                      </g>

                      <!-- Menu/Start Button -->
                      <g 
                        @mousedown="store.setButton(selectedDeviceId!, 'Start', true)"
                        @mouseup="store.setButton(selectedDeviceId!, 'Start', false)"
                        @mouseleave="store.setButton(selectedDeviceId!, 'Start', false)"
                        @touchstart.prevent="store.setButton(selectedDeviceId!, 'Start', true)"
                        @touchend.prevent="store.setButton(selectedDeviceId!, 'Start', false)"
                        class="cursor-pointer"
                      >
                        <circle cx="253" cy="162" r="10" :style="{ fill: isBtnStart ? '#6366f1' : 'var(--color-surface-elevated)' }" class="transition-all duration-200" />
                        <path d="M249 158 H257 M249 162 H257 M249 166 H257" stroke="currentColor" stroke-width="1.5" :style="{ stroke: isBtnStart ? '#ffffff' : 'var(--color-text-muted)' }" class="pointer-events-none transition-colors duration-200" />
                      </g>

                      <!-- Share Button (Visual only) -->
                      <g class="opacity-70">
                        <rect x="208" y="178" width="24" height="14" rx="7" class="fill-[var(--color-background)]" />
                        <path d="M220 180 V186 M218 182 L220 180 L222 182 M217 188 H223" stroke="currentColor" stroke-width="1.5" class="fill-none stroke-stone-400 dark:stroke-stone-300" stroke-linecap="round" stroke-linejoin="round" />
                      </g>

                      <!-- Guide Button (Home logo) -->
                      <g 
                        @mousedown="store.setButton(selectedDeviceId!, 'Guide', true)"
                        @mouseup="store.setButton(selectedDeviceId!, 'Guide', false)"
                        @mouseleave="store.setButton(selectedDeviceId!, 'Guide', false)"
                        @touchstart.prevent="store.setButton(selectedDeviceId!, 'Guide', true)"
                        @touchend.prevent="store.setButton(selectedDeviceId!, 'Guide', false)"
                        class="cursor-pointer"
                      >
                        <circle cx="220.5" cy="125" r="16" :style="{ fill: isBtnGuide ? '#6366f1' : 'var(--color-surface-elevated)' }" class="transition-all duration-200" />
                        <path d="M216 120 L225 130 M225 120 L216 130" stroke-width="3" stroke-linecap="round" :style="{ stroke: isBtnGuide ? '#ffffff' : 'var(--color-text-muted)' }" class="transition-colors duration-200" />
                      </g>
                    </g>
                  </svg>
                </div>
                <CardFooter class="flex w-full items-center gap-2 border-t border-dashed border-border pb-0 pl-0 pr-0 pt-3 text-[11px] text-muted-foreground">
                  <span class="rounded bg-indigo-500/10 px-[6px] py-[1px] text-[10px] font-bold text-indigo-500">提示</span>
                  <span>可以直接在手柄上拖拽摇杆、按压扳机或点击按钮进行交互测试</span>
                </CardFooter>
              </Card>
            </div>

            <!-- Right Side: Diagnostics, Precision Panels -->
            <div class="flex flex-col gap-4">
              <!-- Stick Coordinates circles -->
              <Card>
                <CardHeader>
                  <CardTitle class="text-[12px] font-bold uppercase tracking-wide text-muted-foreground">摇杆高精视窗 (±1.00)</CardTitle>
                </CardHeader>
                <CardContent>
                  <div class="flex justify-around gap-4 py-1">
                    <StickVisualizer
                      :modelValue="{ x: selectedDevice()!.state.left_thumb_x / 32767, y: selectedDevice()!.state.left_thumb_y / 32767 }"
                      label="左摇杆"
                      @update:modelValue="(v) => { store.setThumb(selectedDeviceId!, 'LeftX', v.x); store.setThumb(selectedDeviceId!, 'LeftY', v.y) }"
                    />
                    <StickVisualizer
                      :modelValue="{ x: selectedDevice()!.state.right_thumb_x / 32767, y: selectedDevice()!.state.right_thumb_y / 32767 }"
                      label="右摇杆"
                      @update:modelValue="(v) => { store.setThumb(selectedDeviceId!, 'RightX', v.x); store.setThumb(selectedDeviceId!, 'RightY', v.y) }"
                    />
                  </div>
                </CardContent>
              </Card>

              <!-- Trigger pressure visualizers -->
              <Card>
                <CardHeader>
                  <CardTitle class="text-[12px] font-bold uppercase tracking-wide text-muted-foreground">扳机键压力指示 (0 - 1)</CardTitle>
                </CardHeader>
                <CardContent class="flex flex-col gap-4">
                  <TriggerBar
                    :modelValue="selectedDevice()!.state.left_trigger"
                    label="L 扳机 (LT)"
                    @update:modelValue="(v) => store.setTrigger(selectedDeviceId!, 'Left', v / 255)"
                  />
                  <TriggerBar
                    :modelValue="selectedDevice()!.state.right_trigger"
                    label="R 扳机 (RT)"
                    @update:modelValue="(v) => store.setTrigger(selectedDeviceId!, 'Right', v / 255)"
                  />
                </CardContent>
              </Card>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  </div>
</template>