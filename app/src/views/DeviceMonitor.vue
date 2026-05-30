<script setup lang="ts">
import { onMounted, onUnmounted, computed } from 'vue'
import { useControllerStore } from '../stores/controller'
import DeviceCard from '../components/controller/DeviceCard.vue'
import StickVisualizer from '../components/controller/StickVisualizer.vue'
import TriggerBar from '../components/controller/TriggerBar.vue'
import { Plus, AlertTriangle, CheckCircle, AlertCircle } from '@lucide/vue'
import { ref } from 'vue'

const store = useControllerStore()
const selectedDeviceId = ref<string | null>(null)

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

    <div v-if="selectedDevice()" class="control-panel" :class="{ 'is-offline': !selectedDevice()!.connected }">
      <div class="control-header">
        <h3>输入控制 — 设备 #{{ selectedDevice()!.index }}</h3>
        <span v-if="!selectedDevice()!.connected" class="offline-badge">已断开连接（保留在列表）</span>
      </div>
      <div class="control-grid">
        <div class="stick-section">
          <StickVisualizer
            :modelValue="{ x: selectedDevice()!.state.left_thumb_x / 32767, y: selectedDevice()!.state.left_thumb_y / 32767 }"
            label="L 摇杆"
            @update:modelValue="(v) => { store.setThumb(selectedDeviceId!, 'LeftX', v.x); store.setThumb(selectedDeviceId!, 'LeftY', v.y) }"
          />
          <StickVisualizer
            :modelValue="{ x: selectedDevice()!.state.right_thumb_x / 32767, y: selectedDevice()!.state.right_thumb_y / 32767 }"
            label="R 摇杆"
            @update:modelValue="(v) => { store.setThumb(selectedDeviceId!, 'RightX', v.x); store.setThumb(selectedDeviceId!, 'RightY', v.y) }"
          />
        </div>
        <div class="trigger-section">
          <TriggerBar
            :modelValue="selectedDevice()!.state.left_trigger"
            label="L扳机"
            @update:modelValue="(v) => store.setTrigger(selectedDeviceId!, 'Left', v / 255)"
          />
          <TriggerBar
            :modelValue="selectedDevice()!.state.right_trigger"
            label="R扳机"
            @update:modelValue="(v) => store.setTrigger(selectedDeviceId!, 'Right', v / 255)"
          />
        </div>
        <div class="buttons-section">
          <div class="controller-layout">
            <div class="shoulder-row">
              <button
                class="game-btn shoulder-btn"
                :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.LB) }"
                @mousedown="store.setButton(selectedDeviceId!, 'LB', true)"
                @mouseup="store.setButton(selectedDeviceId!, 'LB', false)"
                @mouseleave="store.setButton(selectedDeviceId!, 'LB', false)"
                @touchstart.prevent="store.setButton(selectedDeviceId!, 'LB', true)"
                @touchend.prevent="store.setButton(selectedDeviceId!, 'LB', false)"
              >LB</button>
              <button
                class="game-btn guide-btn"
                :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.GUIDE) }"
                @mousedown="store.setButton(selectedDeviceId!, 'Guide', true)"
                @mouseup="store.setButton(selectedDeviceId!, 'Guide', false)"
                @mouseleave="store.setButton(selectedDeviceId!, 'Guide', false)"
                @touchstart.prevent="store.setButton(selectedDeviceId!, 'Guide', true)"
                @touchend.prevent="store.setButton(selectedDeviceId!, 'Guide', false)"
              >⌂</button>
              <button
                class="game-btn shoulder-btn"
                :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.RB) }"
                @mousedown="store.setButton(selectedDeviceId!, 'RB', true)"
                @mouseup="store.setButton(selectedDeviceId!, 'RB', false)"
                @mouseleave="store.setButton(selectedDeviceId!, 'RB', false)"
                @touchstart.prevent="store.setButton(selectedDeviceId!, 'RB', true)"
                @touchend.prevent="store.setButton(selectedDeviceId!, 'RB', false)"
              >RB</button>
            </div>
            <div class="main-row">
              <div class="left-side">
                <button
                  class="game-btn stick-btn"
                  :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.LS) }"
                  @mousedown="store.setButton(selectedDeviceId!, 'LeftThumb', true)"
                  @mouseup="store.setButton(selectedDeviceId!, 'LeftThumb', false)"
                  @mouseleave="store.setButton(selectedDeviceId!, 'LeftThumb', false)"
                  @touchstart.prevent="store.setButton(selectedDeviceId!, 'LeftThumb', true)"
                  @touchend.prevent="store.setButton(selectedDeviceId!, 'LeftThumb', false)"
                >LS</button>
                <div class="dpad-cluster">
                  <div class="dpad-row">
                    <button
                      class="game-btn dpad-btn"
                      :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.UP) }"
                      @mousedown="store.setButton(selectedDeviceId!, 'DPadUp', true)"
                      @mouseup="store.setButton(selectedDeviceId!, 'DPadUp', false)"
                      @mouseleave="store.setButton(selectedDeviceId!, 'DPadUp', false)"
                      @touchstart.prevent="store.setButton(selectedDeviceId!, 'DPadUp', true)"
                      @touchend.prevent="store.setButton(selectedDeviceId!, 'DPadUp', false)"
                    >↑</button>
                  </div>
                  <div class="dpad-row">
                    <button
                      class="game-btn dpad-btn"
                      :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.LEFT) }"
                      @mousedown="store.setButton(selectedDeviceId!, 'DPadLeft', true)"
                      @mouseup="store.setButton(selectedDeviceId!, 'DPadLeft', false)"
                      @mouseleave="store.setButton(selectedDeviceId!, 'DPadLeft', false)"
                      @touchstart.prevent="store.setButton(selectedDeviceId!, 'DPadLeft', true)"
                      @touchend.prevent="store.setButton(selectedDeviceId!, 'DPadLeft', false)"
                    >←</button>
                    <div class="dpad-center"></div>
                    <button
                      class="game-btn dpad-btn"
                      :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.RIGHT) }"
                      @mousedown="store.setButton(selectedDeviceId!, 'DPadRight', true)"
                      @mouseup="store.setButton(selectedDeviceId!, 'DPadRight', false)"
                      @mouseleave="store.setButton(selectedDeviceId!, 'DPadRight', false)"
                      @touchstart.prevent="store.setButton(selectedDeviceId!, 'DPadRight', true)"
                      @touchend.prevent="store.setButton(selectedDeviceId!, 'DPadRight', false)"
                    >→</button>
                  </div>
                  <div class="dpad-row">
                    <button
                      class="game-btn dpad-btn"
                      :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.DOWN) }"
                      @mousedown="store.setButton(selectedDeviceId!, 'DPadDown', true)"
                      @mouseup="store.setButton(selectedDeviceId!, 'DPadDown', false)"
                      @mouseleave="store.setButton(selectedDeviceId!, 'DPadDown', false)"
                      @touchstart.prevent="store.setButton(selectedDeviceId!, 'DPadDown', true)"
                      @touchend.prevent="store.setButton(selectedDeviceId!, 'DPadDown', false)"
                    >↓</button>
                  </div>
                </div>
              </div>
              <div class="center-side">
                <button
                  class="game-btn misc-btn"
                  :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.BACK) }"
                  @mousedown="store.setButton(selectedDeviceId!, 'Back', true)"
                  @mouseup="store.setButton(selectedDeviceId!, 'Back', false)"
                  @mouseleave="store.setButton(selectedDeviceId!, 'Back', false)"
                  @touchstart.prevent="store.setButton(selectedDeviceId!, 'Back', true)"
                  @touchend.prevent="store.setButton(selectedDeviceId!, 'Back', false)"
                >BACK</button>
                <button
                  class="game-btn misc-btn"
                  :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.START) }"
                  @mousedown="store.setButton(selectedDeviceId!, 'Start', true)"
                  @mouseup="store.setButton(selectedDeviceId!, 'Start', false)"
                  @mouseleave="store.setButton(selectedDeviceId!, 'Start', false)"
                  @touchstart.prevent="store.setButton(selectedDeviceId!, 'Start', true)"
                  @touchend.prevent="store.setButton(selectedDeviceId!, 'Start', false)"
                >START</button>
              </div>
              <div class="right-side">
                <div class="abxy-cluster">
                  <div class="abxy-row">
                    <button
                      class="game-btn y-btn"
                      :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.Y) }"
                      @mousedown="store.setButton(selectedDeviceId!, 'Y', true)"
                      @mouseup="store.setButton(selectedDeviceId!, 'Y', false)"
                      @mouseleave="store.setButton(selectedDeviceId!, 'Y', false)"
                      @touchstart.prevent="store.setButton(selectedDeviceId!, 'Y', true)"
                      @touchend.prevent="store.setButton(selectedDeviceId!, 'Y', false)"
                    >Y</button>
                  </div>
                  <div class="abxy-row">
                    <button
                      class="game-btn x-btn"
                      :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.X) }"
                      @mousedown="store.setButton(selectedDeviceId!, 'X', true)"
                      @mouseup="store.setButton(selectedDeviceId!, 'X', false)"
                      @mouseleave="store.setButton(selectedDeviceId!, 'X', false)"
                      @touchstart.prevent="store.setButton(selectedDeviceId!, 'X', true)"
                      @touchend.prevent="store.setButton(selectedDeviceId!, 'X', false)"
                    >X</button>
                    <div class="abxy-center"></div>
                    <button
                      class="game-btn b-btn"
                      :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.B) }"
                      @mousedown="store.setButton(selectedDeviceId!, 'B', true)"
                      @mouseup="store.setButton(selectedDeviceId!, 'B', false)"
                      @mouseleave="store.setButton(selectedDeviceId!, 'B', false)"
                      @touchstart.prevent="store.setButton(selectedDeviceId!, 'B', true)"
                      @touchend.prevent="store.setButton(selectedDeviceId!, 'B', false)"
                    >B</button>
                  </div>
                  <div class="abxy-row">
                    <button
                      class="game-btn a-btn"
                      :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.A) }"
                      @mousedown="store.setButton(selectedDeviceId!, 'A', true)"
                      @mouseup="store.setButton(selectedDeviceId!, 'A', false)"
                      @mouseleave="store.setButton(selectedDeviceId!, 'A', false)"
                      @touchstart.prevent="store.setButton(selectedDeviceId!, 'A', true)"
                      @touchend.prevent="store.setButton(selectedDeviceId!, 'A', false)"
                    >A</button>
                  </div>
                </div>
                <button
                  class="game-btn stick-btn"
                  :class="{ pressed: isPressed(selectedDevice()!.state.buttons, BTN.RS) }"
                  @mousedown="store.setButton(selectedDeviceId!, 'RightThumb', true)"
                  @mouseup="store.setButton(selectedDeviceId!, 'RightThumb', false)"
                  @mouseleave="store.setButton(selectedDeviceId!, 'RightThumb', false)"
                  @touchstart.prevent="store.setButton(selectedDeviceId!, 'RightThumb', true)"
                  @touchend.prevent="store.setButton(selectedDeviceId!, 'RightThumb', false)"
                >RS</button>
              </div>
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
  background: rgba(34, 197, 94, 0.1);
  color: var(--color-cta);
  border: 1px solid rgba(34, 197, 94, 0.2);
}

.vigem-status-bar.disconnected {
  background: rgba(234, 179, 8, 0.1);
  color: #eab308;
  border: 1px solid rgba(234, 179, 8, 0.2);
}

.vigem-suggestion-bar {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
  font-size: 12px;
  margin-bottom: var(--space-md);
  background: rgba(59, 130, 246, 0.1);
  color: #3b82f6;
  border: 1px solid rgba(59, 130, 246, 0.2);
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
  border-radius: var(--radius-lg);
  color: var(--color-text-dim);
  cursor: pointer;
  transition: all var(--transition-normal);
  font-size: 12px;
}

.add-device-card:hover {
  border-color: var(--color-cta);
  color: var(--color-cta);
}

.device-card.selected {
  border-color: var(--color-cta);
  box-shadow: 0 0 0 1px var(--color-cta);
}

.control-panel {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-lg);
}

.control-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-md);
}

.control-header h3 {
  font-size: 14px;
  color: var(--color-text);
  margin-bottom: 0;
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

.control-panel.is-offline .control-grid {
  opacity: 0.45;
  pointer-events: none;
}

.control-grid {
  display: flex;
  gap: var(--space-xl);
  flex-wrap: wrap;
}

.stick-section {
  display: flex;
  gap: var(--space-xl);
}

.trigger-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
  min-width: 200px;
  justify-content: center;
}

.buttons-section {
  display: flex;
  align-items: center;
  justify-content: center;
}

.controller-layout {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: 20px;
  padding: var(--space-sm) var(--space-lg) var(--space-md);
  min-width: 340px;
}

.shoulder-row {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  padding: 0 var(--space-sm);
}

.main-row {
  display: flex;
  gap: var(--space-sm);
  align-items: flex-start;
  justify-content: space-between;
}

.left-side {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-md);
  min-width: 90px;
}

.center-side {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--space-sm);
  padding-top: 28px;
}

.right-side {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-md);
  min-width: 90px;
}

.dpad-cluster,
.abxy-cluster {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.dpad-row,
.abxy-row {
  display: flex;
  gap: 2px;
  justify-content: center;
  align-items: center;
}

.dpad-center,
.abxy-center {
  width: 28px;
  height: 28px;
}

.game-btn {
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  border: 1px solid var(--color-border);
  background: var(--color-background);
  color: var(--color-text-dim);
  font-family: var(--font-heading);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}

.game-btn.pressed {
  border-color: var(--color-cta);
  background: rgba(51, 112, 255, 0.12);
  color: var(--color-cta);
  box-shadow: none;
}

.a-btn.pressed { background: rgba(0, 182, 91, 0.12); color: #00B65B; border-color: #00B65B; box-shadow: none; }
.b-btn.pressed { background: rgba(245, 74, 69, 0.12); color: #F54A45; border-color: #F54A45; box-shadow: none; }
.x-btn.pressed { background: rgba(51, 112, 255, 0.12); color: #3370FF; border-color: #3370FF; box-shadow: none; }
.y-btn.pressed { background: rgba(255, 136, 0, 0.12); color: #FF8800; border-color: #FF8800; box-shadow: none; }

.shoulder-btn {
  width: 44px;
  height: 22px;
  font-size: 9px;
  border-radius: var(--radius-sm) var(--radius-sm) var(--radius-md) var(--radius-md);
}

.dpad-btn {
  width: 28px;
  height: 28px;
  font-size: 13px;
  border-radius: 4px;
}

.stick-btn {
  width: 44px;
  height: 44px;
  border-radius: 50%;
  font-size: 10px;
  border: 2px solid var(--color-border);
}

.guide-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  font-size: 14px;
  border: 2px solid var(--color-border);
}

.misc-btn {
  width: 36px;
  height: 20px;
  font-size: 7px;
  border-radius: 10px;
  padding: 0 3px;
  letter-spacing: 0.3px;
}
</style>
