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
  <footer class="flex h-6 flex-shrink-0 items-center justify-between border-t border-border bg-background px-3 text-[11px] text-muted-foreground">
    <div class="flex items-center gap-3">
      <span class="flex items-center gap-1">
        <Wifi :size="12" />
        <span>{{ deviceCount }} 设备运行中</span>
      </span>
    </div>
    <div class="flex items-center gap-3">
      <span class="flex items-center gap-1">
        <Cpu :size="12" />
        <span>CPU: {{ cpuUsage }}</span>
      </span>
      <span class="flex items-center gap-1">
        <MemoryStick :size="12" />
        <span>MEM: {{ memUsage }}</span>
      </span>
      <span class="text-[10px] text-muted-foreground/60">v{{ version }}</span>
    </div>
  </footer>
</template>