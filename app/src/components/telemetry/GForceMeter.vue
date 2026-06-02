<script setup lang="ts">
import { computed } from 'vue'
import { useTelemetryStore } from '@/stores/telemetry'

const telemetry = useTelemetryStore()
const pkt = computed(() => telemetry.displayPacket)

const G_MAX = 2.0
const latG = computed(() => pkt.value ? -(pkt.value.accelX / 9.81) : 0)
const longG = computed(() => pkt.value ? -(pkt.value.accelZ / 9.81) : 0)
const gDotX = computed(() => Math.min(Math.max(latG.value / G_MAX, -1), 1) * 50 + 50)
const gDotY = computed(() => Math.min(Math.max(-longG.value / G_MAX, -1), 1) * 50 + 50)
const gMag = computed(() => Math.hypot(latG.value, longG.value))
const gDotColor = computed(() => gMag.value > 1.5 ? '#ef4444' : gMag.value > 0.8 ? '#f59e0b' : '#22c55e')
</script>

<template>
  <div class="flex items-center gap-1 shrink-0">
    <div
      class="relative overflow-hidden rounded-full shrink-0"
      :style="{
        width: 'clamp(50px, 7.5vw, 76px)',
        height: 'clamp(50px, 7.5vw, 76px)',
        border: '1px solid var(--bd-muted)',
        background: 'radial-gradient(circle, var(--bg-elevated) 0%, var(--bg-panel) 100%)',
      }"
    >
      <div class="absolute rounded-full border border-[var(--bd-subtle)] top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-1/2 h-1/2" />
      <div class="absolute rounded-full border border-[var(--bd-subtle)] top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[78%] h-[78%]" />
      <div class="absolute top-1/2 left-[5%] w-[90%] h-px bg-[var(--bd-subtle)] -translate-y-1/2" />
      <div class="absolute left-1/2 top-[5%] h-[90%] w-px bg-[var(--bd-subtle)] -translate-x-1/2" />
      <div
        class="absolute w-[10px] h-[10px] rounded-full -translate-x-1/2 -translate-y-1/2 transition-[left,top] duration-[40ms]"
        :style="{
          left: gDotX + '%',
          top: gDotY + '%',
          background: gDotColor,
          boxShadow: `0 0 7px ${gDotColor}80`,
        }"
      />
    </div>
    <div class="flex flex-col gap-0.5">
      <span class="text-[clamp(0.38rem,0.7vw,0.48rem)] font-bold text-[var(--tx-xdim)] tabular-nums whitespace-nowrap">
        横向 {{ Math.abs(latG).toFixed(1) }}G
      </span>
      <span class="text-[clamp(0.38rem,0.7vw,0.48rem)] font-bold text-[var(--tx-xdim)] tabular-nums whitespace-nowrap">
        纵向 {{ Math.abs(longG).toFixed(1) }}G
      </span>
    </div>
  </div>
</template>