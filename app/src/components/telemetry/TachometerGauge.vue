<script setup lang="ts">
import { computed } from 'vue'
import { useTelemetryStore } from '@/stores/telemetry'

const props = withDefaults(defineProps<{ useMph?: boolean }>(), { useMph: true })

const telemetry = useTelemetryStore()
const pkt = computed(() => telemetry.displayPacket)
const speed = computed(() => props.useMph ? Math.round(telemetry.speedMph) : Math.round(telemetry.speedKph))
const unit = computed(() => props.useMph ? 'MPH' : 'KPH')
const rpm = computed(() => telemetry.rpmPercent)
const isRedline = computed(() => rpm.value > 90)

const gearLabel = computed(() => {
  if (!pkt.value) return '—'
  if (pkt.value.gear === 0) return 'R'
  if (pkt.value.gear === 11) return 'N'
  return String(pkt.value.gear)
})

const CX = 160, CY = 148, R = 112
const C = 2 * Math.PI * R
const bgArc = (270 / 360) * C
const ROT = 135

const rpmArc = computed(() => (rpm.value / 100) * bgArc)
const boost = computed(() => pkt.value?.boost ?? 0)
const boostActive = computed(() => boost.value > 0.5)
const throttleFrac = computed(() => (pkt.value?.throttle ?? 0) / 255)
const brakeFrac = computed(() => (pkt.value?.brake ?? 0) / 255)
const clutchFrac = computed(() => (pkt.value?.clutch ?? 0) / 255)
const handbrakeOn = computed(() => (pkt.value?.handbrake ?? 0) > 127)
</script>

<template>
  <div class="flex flex-col h-full min-h-0 overflow-hidden">
    <div class="flex-1 min-h-0 w-full h-full">
      <svg viewBox="0 0 320 265" class="w-full h-full">
        <circle
          :cx="CX" :cy="CY" :r="R"
          fill="none" stroke="rgba(239,68,68,0.12)" stroke-width="20" stroke-linecap="round"
          :stroke-dasharray="`${0.1 * bgArc} ${C - 0.1 * bgArc}`"
          :transform="`rotate(${ROT + 0.9 * 270}, ${CX}, ${CY})`"
        />
        <circle
          :cx="CX" :cy="CY" :r="R"
          fill="none" stroke-width="20" stroke-linecap="round"
          class="stroke-[var(--bg-track)]"
          :stroke-dasharray="`${bgArc} ${C - bgArc}`"
          :transform="`rotate(${ROT}, ${CX}, ${CY})`"
        />
        <circle
          :cx="CX" :cy="CY" :r="R"
          fill="none" stroke-width="20" stroke-linecap="round"
          :style="{
            stroke: isRedline ? '#ef4444' : 'var(--ac)',
            transition: 'stroke-dasharray 40ms linear, stroke 80ms ease',
          }"
          :stroke-dasharray="`${rpmArc} ${C - rpmArc}`"
          :transform="`rotate(${ROT}, ${CX}, ${CY})`"
        />

        <text
          :x="CX" :y="CY - 14"
          text-anchor="middle" font-size="66" font-weight="900"
          class="fill-[var(--tx-hi)]"
          font-family="'Segoe UI', system-ui, sans-serif"
          style="font-variant-numeric: tabular-nums;"
        >
          {{ speed }}
        </text>
        <text
          :x="CX" :y="CY + 10"
          text-anchor="middle" font-size="14" font-weight="700"
          class="fill-[var(--tx-xdim)]"
          font-family="'Segoe UI', system-ui, sans-serif"
          letter-spacing="4"
        >
          {{ unit }}
        </text>

        <rect :x="CX - 27" :y="CY + 22" width="54" height="46" rx="8"
          class="fill-[var(--bg-elevated)]"
          :style="{ stroke: isRedline ? '#ef4444' : 'var(--bd-muted)' }"
          stroke-width="2"
        />
        <text
          :x="CX" :y="CY + 58"
          text-anchor="middle" font-size="32" font-weight="900"
          :style="{ fill: isRedline ? '#ef4444' : 'var(--tx-mid)' }"
          font-family="'Segoe UI', system-ui, sans-serif"
        >
          {{ gearLabel }}
        </text>

        <template v-for="i in 11" :key="i">
          <line
            v-bind="(() => {
              const idx = i - 1
              const angle = (ROT + idx * 27) * Math.PI / 180
              const inner = R - 14
              const outer = R + 14
              return {
                x1: CX + inner * Math.cos(angle),
                y1: CY + inner * Math.sin(angle),
                x2: CX + outer * Math.cos(angle),
                y2: CY + outer * Math.sin(angle),
              }
            })()"
            :style="{ stroke: i - 1 >= 9 ? '#ef4444' : 'var(--bd-muted)' }"
            :stroke-width="i % 5 === 0 ? 2.5 : 1.5"
            stroke-linecap="round"
          />
        </template>
      </svg>
    </div>

    <div class="shrink-0 flex items-center gap-4 px-4 pb-3 min-h-0 w-full">
      <div class="flex items-center gap-1 shrink-0">
        <div
          class="relative overflow-hidden rounded-full shrink-0"
          :style="{
            width: 'clamp(56px, 8vw, 80px)',
            height: 'clamp(56px, 8vw, 80px)',
            border: '1px solid var(--bd-muted)',
            background: 'radial-gradient(circle, var(--bg-elevated) 0%, var(--bg-panel) 100%)',
          }"
        >
          <div class="absolute rounded-full border border-[var(--bd-subtle)] top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-1/2 h-1/2" />
          <div class="absolute rounded-full border border-[var(--bd-subtle)] top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[78%] h-[78%]" />
          <div class="absolute top-1/2 left-[5%] w-[90%] h-px bg-[var(--bd-subtle)] -translate-y-1/2" />
          <div class="absolute left-1/2 top-[5%] h-[90%] w-px bg-[var(--bd-subtle)] -translate-x-1/2" />
        </div>
      </div>

      <div class="flex-1 w-full flex flex-col gap-2">
        <div class="flex items-center gap-3 w-full text-base sm:text-lg">
          <span class="font-bold text-[var(--tx-dim)] shrink-0 min-w-[36px]">油门</span>
          <div class="h-3 bg-[var(--bg-track)] rounded overflow-hidden flex-1">
            <div class="h-full bg-green-500 rounded transition-[width] duration-[33ms]" :style="{ width: throttleFrac * 100 + '%' }" />
          </div>
          <span class="font-bold text-[var(--tx-xdim)] tabular-nums w-10 text-right">{{ pkt ? Math.round(throttleFrac * 100) : '—' }}</span>
        </div>

        <div class="flex items-center gap-3 w-full text-base sm:text-lg">
          <span class="font-bold text-[var(--tx-dim)] shrink-0 min-w-[36px]">刹车</span>
          <div class="h-3 bg-[var(--bg-track)] rounded overflow-hidden flex-1">
            <div class="h-full bg-red-500 rounded transition-[width] duration-[33ms]" :style="{ width: brakeFrac * 100 + '%' }" />
          </div>
          <span class="font-bold text-[var(--tx-xdim)] tabular-nums w-10 text-right">{{ pkt ? Math.round(brakeFrac * 100) : '—' }}</span>
        </div>

        <div class="flex items-center gap-3 w-full text-base sm:text-lg">
          <span class="font-bold text-[var(--tx-dim)] shrink-0 min-w-[36px]">离合</span>
          <div class="h-3 bg-[var(--bg-track)] rounded overflow-hidden flex-1">
            <div class="h-full bg-slate-400 rounded transition-[width] duration-[33ms]" :style="{ width: clutchFrac * 100 + '%' }" />
          </div>
          <span class="font-bold text-[var(--tx-xdim)] tabular-nums w-10 text-right">{{ pkt ? Math.round(clutchFrac * 100) : '—' }}</span>
        </div>

        <div class="flex flex-wrap items-center gap-x-6 gap-y-2 mt-0.5 text-base sm:text-lg">
          <div class="flex items-center gap-2">
            <span class="font-bold text-[var(--tx-dim)] shrink-0">手刹</span>
            <div
              class="w-5 h-5 rounded-full border-2 transition-colors duration-100"
              :class="handbrakeOn ? 'bg-orange-500 border-orange-500' : 'border-[var(--bd-muted)]'"
            />
            <span class="font-bold tabular-nums w-7 text-left" :class="handbrakeOn ? 'text-orange-500' : 'text-[var(--tx-xdim)]'">
              {{ handbrakeOn ? '开' : '关' }}
            </span>
          </div>

          <div class="flex items-center gap-2">
            <span class="font-bold text-[var(--tx-dim)] shrink-0">增压</span>
            <div
              class="w-5 h-5 rounded-full border-2 transition-colors duration-100"
              :class="boostActive ? 'bg-yellow-500 border-yellow-500' : 'border-[var(--bd-muted)]'"
            />
            <span class="font-bold tabular-nums text-left" :class="boostActive ? 'text-yellow-500' : 'text-[var(--tx-xdim)]'">
              {{ pkt ? boost.toFixed(1) : '—' }}
            </span>
            <span class="text-xs font-bold text-[var(--tx-xdim)] self-end mb-0.5">PSI</span>
          </div>

          <div v-if="pkt" class="flex items-center gap-1.5 ml-auto">
            <span class="text-lg font-bold text-[var(--tx-hi)] tabular-nums">{{ Math.round(pkt.currentEngineRpm).toLocaleString() }}</span>
            <span class="text-xs font-bold text-[var(--tx-xdim)] self-end mb-0.5">RPM</span>
          </div>
        </div>

      </div>
    </div>
  </div>
</template>