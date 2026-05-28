<script setup lang="ts">
import { Cpu, MemoryStick, Wifi } from '@lucide/vue'

defineProps<{
  deviceCount: number
  cpuUsage: string
  memUsage: string
  version: string
}>()
</script>

<template>
  <footer class="status-bar">
    <div class="status-left">
      <span class="status-item">
        <Wifi :size="12" />
        <span>{{ deviceCount }} 设备运行中</span>
      </span>
    </div>
    <div class="status-right">
      <span class="status-item">
        <Cpu :size="12" />
        <span>CPU: {{ cpuUsage }}</span>
      </span>
      <span class="status-item">
        <MemoryStick :size="12" />
        <span>MEM: {{ memUsage }}</span>
      </span>
      <span class="status-version">v{{ version }}</span>
    </div>
  </footer>
</template>

<style scoped>
.status-bar {
  height: var(--statusbar-height);
  background: var(--color-surface);
  border-top: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--space-md);
  font-size: 11px;
  color: var(--color-text-dim);
  flex-shrink: 0;
}

.status-left,
.status-right {
  display: flex;
  align-items: center;
  gap: var(--space-md);
}

.status-item {
  display: flex;
  align-items: center;
  gap: var(--space-xs);
}

.status-version {
  color: var(--color-text-dim);
  font-family: var(--font-heading);
  font-size: 10px;
}
</style>
