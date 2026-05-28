<script setup lang="ts">
import type { DeviceInfo } from '../../types/controller'
import { Circle, Trash2, Wifi, WifiOff, Power } from '@lucide/vue'

defineProps<{
  device: DeviceInfo
}>()

const emit = defineEmits<{
  remove: [deviceId: string]
  toggleConnection: [deviceId: string]
}>()
</script>

<template>
  <div class="device-card" :class="{ offline: !device.connected }">
    <div class="card-header">
      <div class="card-title">
        <span class="device-type">{{ device.controller_type === 'xbox360' ? 'Xbox 360' : 'DualShock 4' }}</span>
        <span class="device-index">#{{ device.index }}</span>
      </div>
      <div class="card-actions">
        <span class="vigem-badge" :class="device.vigem_connected ? 'real' : 'simulated'" :title="device.vigem_connected ? 'ViGEmBus 已连接（系统可识别）' : '模拟模式（系统不可识别）'">
          <Wifi v-if="device.vigem_connected" :size="10" />
          <WifiOff v-else :size="10" />
        </span>
        <span class="status-dot" :class="device.connected ? 'online' : 'offline'">
          <Circle :size="8" :fill="device.connected ? '#00B65B' : '#8F959E'" />
        </span>
        <button class="icon-btn" :class="{ 'power-off': !device.connected }" @click="emit('toggleConnection', device.id)" :title="device.connected ? '断开手柄（保留在列表）' : '连接手柄（挂载虚拟设备）'">
          <Power :size="13" />
        </button>
        <button class="icon-btn danger" @click="emit('remove', device.id)" title="移除设备">
          <Trash2 :size="13" />
        </button>
      </div>
    </div>
    <div class="card-body">
      <div class="state-row">
        <span class="state-label">L摇杆</span>
        <span class="state-value">X:{{ device.state.left_thumb_x }} Y:{{ device.state.left_thumb_y }}</span>
      </div>
      <div class="state-row">
        <span class="state-label">R摇杆</span>
        <span class="state-value">X:{{ device.state.right_thumb_x }} Y:{{ device.state.right_thumb_y }}</span>
      </div>
      <div class="state-row">
        <span class="state-label">L扳机</span>
        <div class="trigger-bar">
          <div class="trigger-fill" :style="{ width: (device.state.left_trigger / 255 * 100) + '%' }"></div>
        </div>
      </div>
      <div class="state-row">
        <span class="state-label">R扳机</span>
        <div class="trigger-bar">
          <div class="trigger-fill" :style="{ width: (device.state.right_trigger / 255 * 100) + '%' }"></div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.device-card {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-md);
  transition: all var(--transition-normal);
  cursor: default;
}

.device-card:hover {
  border-color: var(--color-cta);
  box-shadow: var(--shadow-md);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-md);
}

.card-title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.device-type {
  font-family: var(--font-heading);
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text);
}

.device-index {
  font-size: 11px;
  color: var(--color-text-dim);
  font-family: var(--font-heading);
}

.card-actions {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.vigem-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: var(--radius-sm);
}

.vigem-badge.real {
  color: #22C55E;
}

.vigem-badge.simulated {
  color: #eab308;
}

.status-dot {
  display: flex;
  align-items: center;
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

.icon-btn:not(.danger):not(.power-off) {
  color: var(--color-cta);
}

.icon-btn:not(.danger):not(.power-off):hover {
  background: rgba(245, 74, 69, 0.1);
  color: var(--color-error);
}

.icon-btn.power-off {
  color: var(--color-text-dim);
}

.icon-btn.power-off:hover {
  background: rgba(51, 112, 255, 0.1);
  color: var(--color-cta);
}

.icon-btn.danger:hover {
  background: rgba(239, 68, 68, 0.15);
  color: var(--color-error);
}

.card-body {
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  transition: opacity var(--transition-normal);
}

.device-card.offline {
  border-color: var(--color-border);
  background: var(--color-surface-elevated);
}

.device-card.offline:hover {
  border-color: var(--color-text-dim);
}

.device-card.offline .card-body {
  opacity: 0.35;
  pointer-events: none;
}

.state-row {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.state-label {
  font-size: 11px;
  color: var(--color-text-dim);
  min-width: 48px;
  font-family: var(--font-heading);
}

.state-value {
  font-size: 11px;
  color: var(--color-text-muted);
  font-family: var(--font-heading);
}

.trigger-bar {
  flex: 1;
  height: 6px;
  background: var(--color-surface-elevated);
  border-radius: 3px;
  overflow: hidden;
}

.trigger-fill {
  height: 100%;
  background: var(--color-cta);
  border-radius: 3px;
  transition: width var(--transition-fast);
}
</style>
