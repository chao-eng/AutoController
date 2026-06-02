<script setup lang="ts">
import { computed } from 'vue'
import { useTelemetryStore } from '@/stores/telemetry'

const telemetry = useTelemetryStore()
const pkt = computed(() => telemetry.displayPacket)

const headingDeg = computed(() => {
  if (!pkt.value) return 0
  return ((pkt.value.yaw * 180 / Math.PI) % 360 + 360) % 360
})

const compassDir = computed(() => {
  const d = headingDeg.value
  if (d < 22.5 || d >= 337.5) return '北'
  if (d < 67.5) return '东北'
  if (d < 112.5) return '东'
  if (d < 157.5) return '东南'
  if (d < 202.5) return '南'
  if (d < 247.5) return '西南'
  if (d < 292.5) return '西'
  return '西北'
})

const LABELS: Record<number, string> = {
  0: '北', 45: '东北', 90: '东', 135: '东南',
  180: '南', 225: '西南', 270: '西', 315: '西北',
}
const CX = 400
const SCALE = 3

interface TickDef {
  x: number
  deg: number
  isCardinal: boolean
  isMajor: boolean
  label: string | null
}

const ticks = computed<TickDef[]>(() => {
  const result: TickDef[] = []
  const hdg = headingDeg.value
  for (let t = 0; t < 360; t += 5) {
    let diff = ((t - hdg) % 360 + 360) % 360
    if (diff > 180) diff -= 360
    if (Math.abs(diff) > 133) continue
    result.push({
      x: CX + diff * SCALE,
      deg: t,
      isCardinal: t % 45 === 0,
      isMajor: t % 10 === 0,
      label: LABELS[t] ?? null,
    })
  }
  return result
})
</script>

<template>
  <div class="shrink-0 h-[clamp(24px,2.8vh,32px)] bg-[var(--bg-panel)] border-b border-[var(--bd-dim)] relative overflow-hidden">
    <svg viewBox="0 0 800 30" class="w-full h-full block" preserveAspectRatio="xMidYMid meet">
      <line
        v-for="tick in ticks" :key="tick.deg"
        :x1="tick.x" y1="0"
        :x2="tick.x" :y2="tick.isCardinal ? 11 : tick.isMajor ? 7 : 4"
        :class="tick.isCardinal ? 'stroke-[var(--ac)]' : 'stroke-[var(--bd-subtle)]'"
        :stroke-width="tick.isCardinal ? 1.5 : 1"
        stroke-linecap="round"
      />
      <template v-for="tick in ticks" :key="'l'+tick.deg">
        <text
          v-if="tick.label"
          :x="tick.x" :y="(tick.isCardinal ? 11 : 7) + 8"
          text-anchor="middle"
          :font-size="tick.deg % 90 === 0 ? '11' : '9'"
          :font-weight="tick.deg % 90 === 0 ? '800' : '600'"
          :class="tick.deg % 90 === 0 ? 'fill-[var(--tx-mid)]' : 'fill-[var(--ac)]'"
          font-family="system-ui, sans-serif"
        >
          {{ tick.label }}
        </text>
      </template>
      <template v-for="tick in ticks" :key="'d'+tick.deg">
        <text
          v-if="!tick.label && tick.isMajor"
          :x="tick.x" y="22"
          text-anchor="middle" font-size="7"
          class="fill-[var(--bd-muted)]"
          font-family="system-ui, sans-serif"
        >
          {{ tick.deg }}
        </text>
      </template>

      <polygon :points="`${CX},17 ${CX - 5},26 ${CX + 5},26`" class="fill-[var(--tx-mid)]" opacity="0.85" />

      <defs>
        <linearGradient id="cf-l" x1="0" y1="0" x2="1" y2="0">
          <stop offset="0%" stop-color="var(--bg-panel)" stop-opacity="1" />
          <stop offset="18%" stop-color="var(--bg-panel)" stop-opacity="0" />
        </linearGradient>
        <linearGradient id="cf-r" x1="0" y1="0" x2="1" y2="0">
          <stop offset="82%" stop-color="var(--bg-panel)" stop-opacity="0" />
          <stop offset="100%" stop-color="var(--bg-panel)" stop-opacity="1" />
        </linearGradient>
      </defs>
      <rect x="0" y="0" width="160" height="30" fill="url(#cf-l)" />
      <rect x="640" y="0" width="160" height="30" fill="url(#cf-r)" />
    </svg>

    <div class="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 flex items-baseline gap-0.5 pointer-events-none">
      <span class="text-[clamp(0.7rem,1.4vw,0.9rem)] font-extrabold text-[var(--tx-mid)] tabular-nums tracking-wide">
        {{ Math.round(headingDeg).toString().padStart(3, '0') }}°
      </span>
      <span class="text-[clamp(0.6rem,1.2vw,0.78rem)] font-bold text-[var(--ac)] tracking-wider">
        {{ compassDir }}
      </span>
    </div>
  </div>
</template>