<script setup lang="ts">
import { onMounted, onUnmounted, computed, ref, watch } from 'vue'
import { useControllerStore } from '../stores/controller'
import DeviceCard from '../components/controller/DeviceCard.vue'
import StickVisualizer from '../components/controller/StickVisualizer.vue'
import TriggerBar from '../components/controller/TriggerBar.vue'
import { Plus, AlertTriangle, CheckCircle, AlertCircle, Activity } from '@lucide/vue'

const store = useControllerStore()
const selectedDeviceId = ref<string | null>(null)

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
  <div class="device-monitor">
    <div class="page-header">
      <h2>设备监控</h2>
      <button class="btn-primary" @click="addDevice">
        <Plus :size="16" />
        <span>添加设备</span>
      </button>
    </div>

    <div v-if="store.vigemStatus" class="vigem-status-bar" :class="{ connected: store.vigemStatus.connected, disconnected: !store.vigemStatus.connected }">
      <CheckCircle v-if="store.vigemStatus.connected" :size="14" />
      <AlertTriangle v-else :size="14" />
      <span>{{ store.vigemStatus.message }}</span>
    </div>

    <div v-if="vigemSuggestion" class="vigem-suggestion-bar">
      <AlertCircle :size="14" />
      <span>{{ vigemSuggestion }}</span>
    </div>

    <div class="devices-grid">
      <DeviceCard
        v-for="device in store.devices"
        :key="device.id"
        :device="device"
        :class="{ selected: selectedDeviceId === device.id }"
        @click="selectDevice(device.id)"
        @remove="store.removeDevice"
        @toggle-connection="store.toggleConnection"
      />
      <button v-if="store.devices.length < 8" class="add-device-card" @click="addDevice">
        <Plus :size="24" />
        <span>添加设备</span>
      </button>
    </div>

    <!-- Active Virtual Controller Input Dashboard -->
    <div v-if="selectedDevice()" class="control-panel" :class="{ 'is-offline': !selectedDevice()!.connected }">
      <div class="control-header">
        <div class="header-left">
          <Activity :size="16" class="header-icon text-indigo-500" />
          <h3>输入控制 — 虚拟手柄 #{{ selectedDevice()!.index }}</h3>
        </div>
        <span v-if="!selectedDevice()!.connected" class="offline-badge">已断开连接（保留在列表）</span>
      </div>

      <div class="monitor-dashboard">
        <!-- Left Side: Interactive SVG Controller -->
        <div class="dashboard-left">
          <div class="panel-card controller-card">
            <div class="card-inner">
              <svg viewBox="0 0 441 383" class="controller-svg">
                <g id="XboxController">
                  <!-- Left main shell -->
                  <path d="M220.5 294.5C220.5 294.5 195 294.5 150 294.5C105 294.5 81.5 378.5 49.5 378.5C17.5 378.5 4 363.9 4 317.5C4 271.1 43.5 165.5 55 137.5C66.5 109.5 95.5 92.0001 128 92.0001C154 92.0001 200.5 92.0001 220.5 92.0001" class="fill-body stroke-body" />
                  <!-- Right main shell -->
                  <path d="M220 294.5C220 294.5 245.5 294.5 290.5 294.5C335.5 294.5 359 378.5 391 378.5C423 378.5 436.5 363.9 436.5 317.5C436.5 271.1 397 165.5 385.5 137.5C374 109.5 345 92.0001 312.5 92.0001C286.5 92.0001 240 92.0001 220 92.0001" class="fill-body stroke-body" />
                  
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
                    class="transition-colors duration-100 cursor-ns-resize" 
                  />
                  
                  <!-- Right Trigger (RT) - Interactive drag & depth fill -->
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
                    class="transition-colors duration-100 cursor-ns-resize" 
                  />

                  <!-- Left Bumper (LB) - Interactive click -->
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
                      class="text-[10px] font-bold pointer-events-none transition-colors duration-200"
                    >LB</text>
                  </g>

                  <!-- Right Bumper (RB) - Interactive click -->
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
                      class="text-[10px] font-bold pointer-events-none transition-colors duration-200"
                    >RB</text>
                  </g>

                  <!-- Left Stick well and interactive pointer drag knob -->
                  <g 
                    @pointerdown="handleLeftStickPointerDown" 
                    @pointermove="handleLeftStickPointerMove" 
                    @pointerup="handleLeftStickPointerUp" 
                    @pointercancel="handleLeftStickPointerUp"
                    class="interactive-stick select-none"
                  >
                    <circle cx="113" cy="160" r="37.5" class="fill-well stroke-well" />
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
                      <circle cx="113" cy="160" r="22" class="fill-none stroke-black/5 dark:stroke-white/5 stroke-2" />
                      <circle cx="113" cy="160" r="10" class="fill-stone-400/50 stroke-none" />
                    </g>
                    <!-- Transparent helper target circle to extend click space -->
                    <circle cx="113" cy="160" r="42" fill="transparent" class="cursor-grab" />
                  </g>

                  <!-- Right Stick well and interactive pointer drag knob -->
                  <g 
                    @pointerdown="handleRightStickPointerDown" 
                    @pointermove="handleRightStickPointerMove" 
                    @pointerup="handleRightStickPointerUp" 
                    @pointercancel="handleRightStickPointerUp"
                    class="interactive-stick select-none"
                  >
                    <circle cx="278" cy="238" r="37.5" class="fill-well stroke-well" />
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
                      <circle cx="278" cy="238" r="22" class="fill-none stroke-black/5 dark:stroke-white/5 stroke-2" />
                      <circle cx="278" cy="238" r="10" class="fill-stone-400/50 stroke-none" />
                    </g>
                    <!-- Transparent helper target circle to extend click space -->
                    <circle cx="278" cy="238" r="42" fill="transparent" class="cursor-grab" />
                  </g>

                  <!-- D-pad well and four directional rects -->
                  <circle cx="166" cy="238" r="37.5" class="fill-well stroke-well" />
                  
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
                    class="transition-all duration-100 cursor-pointer" 
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
                    class="transition-all duration-100 cursor-pointer" 
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
                    class="transition-all duration-100 cursor-pointer" 
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
                    class="transition-all duration-100 cursor-pointer" 
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
                  <circle cx="329" cy="160" r="42" class="fill-well stroke-well" />
                  
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
                      class="text-[12px] font-black pointer-events-none transition-colors duration-100"
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
                      class="text-[12px] font-black pointer-events-none transition-colors duration-100"
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
                      class="text-[12px] font-black pointer-events-none transition-colors duration-100"
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
                      class="text-[12px] font-black pointer-events-none transition-colors duration-100"
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
                    <rect x="208" y="178" width="24" height="14" rx="7" class="fill-well" />
                    <path d="M220 180 V186 M218 182 L220 180 L222 182 M217 188 H223" stroke="currentColor" stroke-width="1.5" class="text-stone-400 dark:text-stone-300 pointer-events-none" fill="none" stroke-linecap="round" stroke-linejoin="round" />
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
            <div class="card-footer">
              <span class="tip-tag">提示</span>
              <span>可以直接在手柄上拖拽摇杆、按压扳机或点击按钮进行交互测试</span>
            </div>
          </div>
        </div>

        <!-- Right Side: Diagnostics, Precision Panels -->
        <div class="dashboard-right">
          <!-- Stick Coordinates circles -->
          <div class="panel-card precision-card">
            <h4 class="card-title">摇杆高精视窗 (±1.00)</h4>
            <div class="stick-visualizers-grid">
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
          </div>

          <!-- Trigger pressure visualizers -->
          <div class="panel-card triggers-card">
            <h4 class="card-title">扳机键压力指示 (0 - 1)</h4>
            <div class="trigger-bars-layout">
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
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.device-monitor {
  padding: var(--space-lg);
  height: 100%;
  overflow-y: auto;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-lg);
}

.page-header h2 {
  font-size: 18px;
  font-weight: 600;
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  background: var(--color-cta);
  color: white;
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-weight: 500;
  font-size: 13px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-primary:hover {
  background: var(--color-cta-hover);
  transform: translateY(-1px);
}

.vigem-status-bar {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: 12px;
  margin-bottom: var(--space-md);
}

.vigem-status-bar.connected {
  background: rgba(0, 182, 91, 0.1);
  color: var(--color-success);
  border: 1px solid rgba(0, 182, 91, 0.2);
}

.vigem-status-bar.disconnected {
  background: rgba(255, 136, 0, 0.1);
  color: var(--color-warning);
  border: 1px solid rgba(255, 136, 0, 0.2);
}

.vigem-suggestion-bar {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: 12px;
  margin-bottom: var(--space-md);
  background: rgba(51, 112, 255, 0.1);
  color: var(--color-info);
  border: 1px solid rgba(51, 112, 255, 0.2);
}

.devices-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: var(--space-md);
  margin-bottom: var(--space-xl);
}

.add-device-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-sm);
  min-height: 140px;
  background: transparent;
  border: 2px dashed var(--color-border);
  border-radius: var(--radius-xl);
  color: var(--color-text-dim);
  cursor: pointer;
  transition: all var(--transition-normal);
  font-size: 12px;
}

.add-device-card:hover {
  border-color: var(--color-cta);
  color: var(--color-cta);
  background: rgba(51, 112, 255, 0.02);
}

.device-card.selected {
  border-color: var(--color-cta);
  box-shadow: var(--shadow-md), 0 0 0 1px var(--color-cta);
}

.control-panel {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  padding: var(--space-lg);
  box-shadow: var(--shadow-md);
  transition: opacity var(--transition-normal);
}

.control-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-lg);
  padding-bottom: var(--space-md);
  border-bottom: 1px solid var(--color-border);
}

.header-left {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.control-header h3 {
  font-size: 15px;
  color: var(--color-text);
  margin-bottom: 0;
  font-weight: 600;
}

.offline-badge {
  font-size: 11px;
  font-weight: 500;
  color: var(--color-text-dim);
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  padding: 2px 8px;
  border-radius: var(--radius-sm);
}

.control-panel.is-offline .monitor-dashboard {
  opacity: 0.45;
  pointer-events: none;
}

/* Upgraded modern two-column dashboard structure */
.monitor-dashboard {
  display: grid;
  grid-template-columns: 1.2fr 1fr;
  gap: var(--space-lg);
}

@media (max-width: 1024px) {
  .monitor-dashboard {
    grid-template-columns: 1fr;
  }
}

.dashboard-left {
  display: flex;
  flex-direction: column;
}

.dashboard-right {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.panel-card {
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  padding: var(--space-md);
  box-shadow: var(--shadow-sm);
  transition: all var(--transition-normal);
}

.panel-card:hover {
  border-color: rgba(99, 102, 241, 0.3);
  box-shadow: var(--shadow-md);
}

.card-title {
  font-size: 12px;
  font-weight: 700;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: var(--space-md);
}

/* Left controller SVG container styling */
.controller-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: center;
  min-height: 420px;
  padding: var(--space-lg) var(--space-md) var(--space-md);
  position: relative;
}

.card-inner {
  width: 100%;
  max-width: 440px;
  margin: 0 auto;
}

.controller-svg {
  width: 100%;
  height: auto;
  filter: drop-shadow(0 10px 15px rgba(0, 0, 0, 0.05));
  transition: transform 0.1s ease;
}

/* Interactive SVG color schemes & classes mapping to dynamic theme colors */
.fill-body {
  fill: var(--color-surface);
  transition: fill var(--transition-normal);
}
.stroke-body {
  stroke: var(--color-border);
  stroke-width: 3px;
  transition: stroke var(--transition-normal);
}

.fill-well {
  fill: var(--color-background);
  transition: fill var(--transition-normal);
}
.stroke-well {
  stroke: var(--color-border);
  stroke-width: 2px;
  transition: stroke var(--transition-normal);
}

.card-footer {
  width: 100%;
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  font-size: 11px;
  color: var(--color-text-dim);
  border-top: 1px dashed var(--color-border);
  padding-top: var(--space-sm);
  margin-top: var(--space-md);
}

.tip-tag {
  background: rgba(99, 102, 241, 0.1);
  color: #6366f1;
  padding: 1px 6px;
  border-radius: var(--radius-sm);
  font-weight: 700;
  font-size: 10px;
}

/* Precision circle visualizers grid layout */
.stick-visualizers-grid {
  display: flex;
  justify-content: space-around;
  gap: var(--space-md);
  padding: var(--space-sm) 0;
}

.trigger-bars-layout {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

/* Fix text cursor showing up over SVG texts */
.controller-svg text {
  user-select: none;
  pointer-events: none; /* Clicks and hovers ignore text, going straight to the parent interactive group */
}

/* Ensure all clickable areas display the hand cursor natively */
.cursor-pointer,
.cursor-pointer * {
  cursor: pointer !important;
}

.cursor-grab,
.cursor-grab * {
  cursor: grab !important;
}

.cursor-ns-resize,
.cursor-ns-resize * {
  cursor: ns-resize !important;
}
</style>
