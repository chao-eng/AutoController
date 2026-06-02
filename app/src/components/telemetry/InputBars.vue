<script setup lang="ts">
import { computed } from 'vue'
import { useTelemetryStore } from '@/stores/telemetry'

const telemetry = useTelemetryStore()
const pkt = computed(() => telemetry.packet)

const steerNorm = computed(() => pkt.value ? pkt.value.steer / 128 : 0)
const steerFillLeft = computed(() => steerNorm.value < 0 ? Math.abs(steerNorm.value) * 50 : 0)
const steerFillRight = computed(() => steerNorm.value > 0 ? steerNorm.value * 50 : 0)

interface BarDef {
  label: string
  value: number
  color: string
}

const bars = computed<BarDef[]>(() => [
  { label: 'THR', value: pkt.value ? pkt.value.throttle / 255 : 0, color: '#22c55e' },
  { label: 'BRK', value: pkt.value ? pkt.value.brake / 255 : 0, color: '#ef4444' },
  { label: 'CLT', value: pkt.value ? pkt.value.clutch / 255 : 0, color: '#94a3b8' },
  { label: 'HBK', value: pkt.value ? pkt.value.handbrake / 255 : 0, color: '#f97316' },
])
</script>

<template>
  <div class="flex flex-col h-full p-2 gap-1 box-border overflow-hidden">
    <div class="flex flex-col items-center gap-0.5 shrink-0">
      <span class="text-[clamp(0.48rem,1.1vw,0.6rem)] font-bold tracking-wider text-[var(--tx-dim)]">STR</span>
      <div class="w-full h-2 bg-[var(--bg-track)] rounded relative overflow-hidden">
        <div class="absolute left-1/2 top-0 w-[1.5px] h-full bg-[var(--bd-muted)] -translate-x-1/2" />
        <div
          class="absolute top-0 h-full bg-[var(--ac)] transition-[width] duration-[33ms]"
          :style="{
            width: steerFillLeft + '%',
            right: '50%',
            borderRadius: '4px 0 0 4px',
          }"
        />
        <div
          class="absolute top-0 h-full bg-[var(--ac)] transition-[width] duration-[33ms]"
          :style="{
            width: steerFillRight + '%',
            left: '50%',
            borderRadius: '0 4px 4px 0',
          }"
        />
      </div>
    </div>

    <div class="flex-1 min-h-0 flex flex-row gap-[clamp(0.2rem,0.8vw,0.4rem)] items-end">
      <div v-for="bar in bars" :key="bar.label" class="flex flex-col items-center gap-1 flex-1 min-w-0 h-full">
        <div class="flex-1 w-full bg-[var(--bg-track)] rounded flex flex-col justify-end overflow-hidden min-h-0">
          <div
            class="w-full transition-[height] duration-[33ms] rounded"
            :style="{ height: bar.value * 100 + '%', background: bar.color }"
          />
        </div>
        <span class="text-[clamp(0.45rem,1vw,0.6rem)] font-bold tracking-wide text-[var(--tx-dim)] whitespace-nowrap">
          {{ bar.label }}
        </span>
      </div>
    </div>
  </div>
</template>