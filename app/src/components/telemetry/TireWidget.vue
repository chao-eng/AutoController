<script setup lang="ts">
import { computed } from 'vue'
import { useTelemetryStore } from '@/stores/telemetry'

withDefaults(defineProps<{
  tireTempCold?: number
  tireTempOptimal?: number
  tireTempHot?: number
}>(), {
  tireTempCold: 60,
  tireTempOptimal: 85,
  tireTempHot: 110,
})

const telemetry = useTelemetryStore()
const pkt = computed(() => telemetry.displayPacket)

function tempColor(t: number, cold: number, optimal: number, hot: number): string {
  if (t < cold) return '#3b82f6'
  if (t < optimal) return '#22c55e'
  if (t < hot) return '#f59e0b'
  return '#ef4444'
}

function slipColor(s: number): string {
  const a = Math.abs(s)
  if (a < 0.05) return '#22c55e'
  if (a < 0.15) return '#f59e0b'
  return '#ef4444'
}

function suspColor(s: number): string {
  if (s > 0.88) return '#ef4444'
  if (s > 0.72) return '#f59e0b'
  if (s < 0.12) return '#3b82f6'
  return '#6366f1'
}

function wearColor(w: number): string {
  if (w > 0.7) return '#22c55e'
  if (w > 0.4) return '#f59e0b'
  return '#ef4444'
}

interface TireData {
  label: string
  temp: number
  slip: number
  susp: number
  wear: number | null
}

const tires = computed<TireData[]>(() => [
  { label: '前左', temp: pkt.value?.tireTempFl ?? 0, slip: pkt.value?.tireSlipRatioFl ?? 0, susp: pkt.value?.suspensionFl ?? 0.5, wear: pkt.value?.tireWearFl ?? null },
  { label: '前右', temp: pkt.value?.tireTempFr ?? 0, slip: pkt.value?.tireSlipRatioFr ?? 0, susp: pkt.value?.suspensionFr ?? 0.5, wear: pkt.value?.tireWearFr ?? null },
  { label: '后左', temp: pkt.value?.tireTempRl ?? 0, slip: pkt.value?.tireSlipRatioRl ?? 0, susp: pkt.value?.suspensionRl ?? 0.5, wear: pkt.value?.tireWearRl ?? null },
  { label: '后右', temp: pkt.value?.tireTempRr ?? 0, slip: pkt.value?.tireSlipRatioRr ?? 0, susp: pkt.value?.suspensionRr ?? 0.5, wear: pkt.value?.tireWearRr ?? null },
])
</script>

<template>
  <div class="h-full flex flex-col p-1 box-border overflow-hidden">
    <div class="grid grid-cols-2 grid-rows-2 gap-1 flex-1 min-h-0">
      <div
        v-for="tire in tires" :key="tire.label"
        class="flex flex-row gap-0.5 overflow-hidden min-h-0 relative rounded-lg p-1 pl-1.5"
        :style="{
          border: '1.5px solid color-mix(in srgb, ' + tempColor(tire.temp, tireTempCold, tireTempOptimal, tireTempHot) + ' 28%, var(--bg-elevated))',
          background: 'var(--bg-card)',
        }"
      >
        <div
          class="absolute inset-0 pointer-events-none"
          :style="{
            background: 'radial-gradient(ellipse at 30% 30%, color-mix(in srgb, ' + tempColor(tire.temp, tireTempCold, tireTempOptimal, tireTempHot) + ' 7%, transparent) 0%, transparent 65%)',
          }"
        />

        <div class="flex-1 min-w-0 flex flex-col items-center justify-between py-[0.1rem]">
          <span class="text-[clamp(0.58rem,1.4vw,0.75rem)] font-extrabold text-[var(--tx-xdim)] tracking-wider self-start">
            {{ tire.label }}
          </span>
          <span
            class="text-[clamp(0.85rem,2.8vw,1.2rem)] font-black tabular-nums leading-none"
            :style="{ color: tempColor(tire.temp, tireTempCold, tireTempOptimal, tireTempHot) }"
          >
            {{ pkt ? Math.round(tire.temp) + '°' : '—' }}
          </span>
          <div class="flex items-center justify-center gap-0.5">
            <span
              class="w-[clamp(6px,1.2vw,9px)] h-[clamp(6px,1.2vw,9px)] rounded-full shrink-0 transition-colors duration-[120ms]"
              :style="{ background: slipColor(tire.slip) }"
            />
            <span
              v-if="tire.wear !== null"
              class="text-[clamp(0.55rem,1.3vw,0.72rem)] font-bold tabular-nums"
              :style="{ color: wearColor(tire.wear) }"
            >
              {{ Math.round(tire.wear * 100) }}%
            </span>
          </div>
        </div>

        <div class="w-[clamp(10px,2.2vw,18px)] shrink-0 flex items-stretch py-[0.15rem]">
          <svg viewBox="0 0 10 58" preserveAspectRatio="xMidYMid meet" class="w-full h-full overflow-visible">
            <circle cx="5" cy="3" r="2" class="fill-[var(--bd-strong)]" />
            <line x1="5" y1="5" x2="5" y2="34" class="stroke-[var(--bd-strong)]" stroke-width="1.5" stroke-linecap="round" />
            <template v-if="(1 - Math.min(Math.max(tire.susp, 0), 1)) * 22 > 0.5">
              <rect
                :x="3.8"
                :y="34 - (1 - Math.min(Math.max(tire.susp, 0), 1)) * 22"
                :width="2.4"
                :height="(1 - Math.min(Math.max(tire.susp, 0), 1)) * 22"
                rx="1.2"
                :fill="suspColor(tire.susp)"
                :style="{ transition: 'height 60ms linear, y 60ms linear' }"
              />
            </template>
            <rect x="1.5" y="34" width="7" height="16" rx="2.5" :style="{ fill: 'var(--bg-panel)', stroke: 'var(--bd-strong)' }" stroke-width="1.2" />
            <rect x="3.2" y="33.2" width="3.6" height="2.5" rx="1" :fill="suspColor(tire.susp)" opacity="0.5" />
            <line x1="5" y1="50" x2="5" y2="55" class="stroke-[var(--bd-strong)]" stroke-width="1.5" stroke-linecap="round" />
            <circle cx="5" cy="55.5" r="2" class="fill-[var(--bd-strong)]" />
          </svg>
        </div>
      </div>
    </div>
  </div>
</template>