<script setup lang="ts">
import { computed } from 'vue'
import { useTelemetryStore } from '@/stores/telemetry'

const telemetry = useTelemetryStore()
const pkt = computed(() => telemetry.displayPacket)
const rawPkt = computed(() => telemetry.packet)

const inEvent = computed(() =>
  rawPkt.value?.isRaceOn === true && (rawPkt.value?.racePosition ?? 0) > 0
)

function formatTime(seconds: number): string {
  if (seconds <= 0) return '—:——.———'
  const mins = Math.floor(seconds / 60)
  const secs = seconds % 60
  return `${mins}:${secs.toFixed(3).padStart(6, '0')}`
}
</script>

<template>
  <div class="flex items-center justify-center h-full bg-[var(--bg-panel)] border-t border-[var(--bd-dim)] px-[clamp(0.5rem,2vw,1.5rem)] overflow-hidden">
    <div class="flex flex-col items-center px-[clamp(0.5rem,2vw,1.5rem)] min-w-0 shrink">
      <span class="text-[clamp(0.55rem,1.2vw,0.7rem)] font-bold tracking-wider text-[var(--tx-xdim)] whitespace-nowrap flex items-center gap-0.5">圈数</span>
      <span class="text-[clamp(0.7rem,1.8vw,1.1rem)] font-extrabold tabular-nums text-[var(--tx-mid)] whitespace-nowrap overflow-hidden text-ellipsis">
        {{ pkt ? pkt.lapNumber : '—' }}
      </span>
    </div>
    <div class="w-px h-[clamp(1rem,3vh,2rem)] bg-[var(--bd-subtle)] shrink-0" />
    <div class="flex flex-col items-center px-[clamp(0.5rem,2vw,1.5rem)] min-w-0 shrink">
      <span class="text-[clamp(0.55rem,1.2vw,0.7rem)] font-bold tracking-wider text-[var(--tx-xdim)] whitespace-nowrap">当前圈</span>
      <span class="text-[clamp(0.7rem,1.8vw,1.1rem)] font-extrabold tabular-nums text-[var(--ac)] whitespace-nowrap overflow-hidden text-ellipsis">
        {{ formatTime(pkt?.currentLap ?? 0) }}
      </span>
    </div>
    <div class="w-px h-[clamp(1rem,3vh,2rem)] bg-[var(--bd-subtle)] shrink-0" />
    <div class="flex flex-col items-center px-[clamp(0.5rem,2vw,1.5rem)] min-w-0 shrink">
      <span class="text-[clamp(0.55rem,1.2vw,0.7rem)] font-bold tracking-wider text-[var(--tx-xdim)] whitespace-nowrap">上一圈</span>
      <span class="text-[clamp(0.7rem,1.8vw,1.1rem)] font-extrabold tabular-nums text-[var(--tx-mid)] whitespace-nowrap overflow-hidden text-ellipsis">
        {{ formatTime(pkt?.lastLap ?? 0) }}
      </span>
    </div>
    <div class="w-px h-[clamp(1rem,3vh,2rem)] bg-[var(--bd-subtle)] shrink-0" />
    <div class="flex flex-col items-center px-[clamp(0.5rem,2vw,1.5rem)] min-w-0 shrink">
      <span class="text-[clamp(0.55rem,1.2vw,0.7rem)] font-bold tracking-wider text-[var(--tx-xdim)] whitespace-nowrap">最快圈</span>
      <span class="text-[clamp(0.7rem,1.8vw,1.1rem)] font-extrabold tabular-nums text-purple-500 whitespace-nowrap overflow-hidden text-ellipsis">
        {{ formatTime(pkt?.bestLap ?? 0) }}
      </span>
    </div>
    <div class="w-px h-[clamp(1rem,3vh,2rem)] bg-[var(--bd-subtle)] shrink-0" />
    <div class="flex flex-col items-center px-[clamp(0.5rem,2vw,1.5rem)] min-w-0 shrink">
      <span class="text-[clamp(0.55rem,1.2vw,0.7rem)] font-bold tracking-wider text-[var(--tx-xdim)] whitespace-nowrap flex items-center gap-0.5">
        本次会话
        <span v-if="inEvent" class="inline-block w-[5px] h-[5px] rounded-full bg-green-500 shadow-[0_0_4px_#22c55e] shrink-0" />
      </span>
      <span
        class="text-[clamp(0.7rem,1.8vw,1.1rem)] font-extrabold tabular-nums w-[clamp(5rem,10vw,7rem)] text-center inline-block transition-colors duration-300 whitespace-nowrap overflow-hidden text-ellipsis"
        :class="inEvent ? 'text-[var(--tx-mid)]' : 'text-[var(--tx-xdim)]'"
      >
        {{ formatTime(pkt?.currentRaceTime ?? 0) }}
      </span>
    </div>
  </div>
</template>