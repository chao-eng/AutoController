<script setup lang="ts">
import { computed } from 'vue'
import { useTelemetryStore } from '@/stores/telemetry'

const telemetry = useTelemetryStore()
const pkt = computed(() => telemetry.displayPacket)
const steerNorm = computed(() => (pkt.value?.steer ?? 0) / 127)
const steerDeg = computed(() => steerNorm.value * 120)
</script>

<template>
  <div class="flex flex-col items-center gap-0.5 shrink-0">
    <svg
      viewBox="-50 -50 100 100"
      class="w-[clamp(48px,7.5vw,76px)] h-[clamp(48px,7.5vw,76px)] shrink-0"
    >
      <rect x="-2" y="-48" width="4" height="7" rx="1.5" class="fill-[var(--bd-muted)]" />
      <g
        :style="{ transform: `rotate(${steerDeg}deg)`, transition: 'transform 40ms linear' }"
      >
        <circle cx="0" cy="0" r="40" fill="none" class="stroke-[var(--bd-strong)]" stroke-width="8" stroke-linecap="round" />
        <line x1="0" y1="-32" x2="0" y2="0" class="stroke-[var(--bd-strong)]" stroke-width="4" stroke-linecap="round" />
        <line x1="27.7" y1="16" x2="0" y2="0" class="stroke-[var(--bd-strong)]" stroke-width="4" stroke-linecap="round" />
        <line x1="-27.7" y1="16" x2="0" y2="0" class="stroke-[var(--bd-strong)]" stroke-width="4" stroke-linecap="round" />
        <circle cx="0" cy="0" r="8" class="fill-[var(--bg-elevated)] stroke-[var(--bd-strong)]" stroke-width="1.5" />
        <circle cx="0" cy="-38" r="4" class="fill-[var(--ac)]" />
      </g>
    </svg>
    <div class="flex gap-1 items-baseline">
      <span class="text-[clamp(0.4rem,0.8vw,0.52rem)] font-bold text-[var(--ac)] tracking-wide min-w-[1.5rem] text-center">
        {{ Math.abs(steerNorm) < 0.05 ? '中' : steerNorm < 0 ? '左' : '右' }}
      </span>
      <span class="text-[clamp(0.38rem,0.75vw,0.48rem)] font-bold text-[var(--tx-xdim)] tabular-nums">
        {{ Math.round(Math.abs(steerNorm) * 100) }}%
      </span>
    </div>
  </div>
</template>