<script setup lang="ts">
import { computed } from 'vue'
import { useTelemetryStore } from '@/stores/telemetry'

const telemetry = useTelemetryStore()
const pkt = computed(() => telemetry.displayPacket)
const rollDeg = computed(() => pkt.value ? pkt.value.roll * 180 / Math.PI : 0)
const pitchDeg = computed(() => pkt.value ? pkt.value.pitch * 180 / Math.PI : 0)

const PITCH_SCALE = 1.6
const pitchOffset = computed(() => Math.min(Math.max(pitchDeg.value * PITCH_SCALE, -50), 50))
const sphereRot = computed(() => -rollDeg.value)

const ROLL_TICKS = [-60, -45, -30, -20, -10, 0, 10, 20, 30, 45, 60]
</script>

<template>
  <div class="flex flex-col items-center gap-0.5 shrink-0">
    <svg
      viewBox="-50 -50 100 100"
      class="w-[clamp(66px,10.5vw,100px)] h-[clamp(66px,10.5vw,100px)] overflow-hidden rounded-full shrink-0"
    >
      <defs>
        <clipPath id="adi-clip">
          <circle cx="0" cy="0" r="46" />
        </clipPath>
      </defs>

      <circle cx="0" cy="0" r="48" class="fill-[var(--bg-panel)] stroke-[var(--bd-subtle)]" stroke-width="1.5" />

      <template v-for="tick in ROLL_TICKS" :key="tick">
        <line
          v-bind="(() => {
            const angle = (tick - 90) * Math.PI / 180
            const tickLen = tick === 0 ? 6 : Math.abs(tick) % 30 === 0 ? 5 : 3
            const inner = 42 - tickLen
            return {
              x1: 42 * Math.cos(angle),
              y1: 42 * Math.sin(angle),
              x2: inner * Math.cos(angle),
              y2: inner * Math.sin(angle),
            }
          })()"
          :class="tick === 0 ? 'stroke-[var(--tx-dim)]' : 'stroke-[var(--bd-subtle)]'"
          :stroke-width="tick === 0 ? 1.5 : 1"
          stroke-linecap="round"
        />
      </template>

      <g
        :style="{ transform: `rotate(${sphereRot}deg)`, transition: 'transform 40ms linear' }"
        clip-path="url(#adi-clip)"
      >
        <g
          :style="{ transform: `translateY(${pitchOffset}px)`, transition: 'transform 40ms linear' }"
        >
          <rect x="-200" y="-200" width="400" height="200" style="fill: var(--adi-sky)" />
          <rect x="-200" y="0" width="400" height="200" style="fill: var(--adi-ground)" />
          <line x1="-200" y1="0" x2="200" y2="0" class="stroke-[var(--tx-dim)]" stroke-width="1.2" />
          <template v-for="deg in [-25, -20, -15, -10, -5, 5, 10, 15, 20, 25]" :key="deg">
            <line
              v-bind="(() => {
                const y = -deg * PITCH_SCALE
                const w = Math.abs(deg) % 10 === 0 ? 14 : 8
                return { x1: -w, y1: y, x2: w, y2: y }
              })()"
              class="stroke-[var(--bd-strong)]"
              stroke-width="0.8"
              stroke-linecap="round"
            />
            <text
              v-if="Math.abs(deg) % 10 === 0"
              v-bind="(() => {
                const y = -deg * PITCH_SCALE
                return { x: Math.abs(deg) % 10 === 0 ? 16 : 10, y: y + 1.5 }
              })()"
              font-size="3.5" class="fill-[var(--bd-strong)]" font-family="system-ui"
            >
              {{ Math.abs(deg) }}
            </text>
          </template>
        </g>
      </g>

      <line x1="-38" y1="0" x2="-14" y2="0" class="stroke-[var(--tx-hi)]" stroke-width="2.5" stroke-linecap="round" />
      <line x1="14" y1="0" x2="38" y2="0" class="stroke-[var(--tx-hi)]" stroke-width="2.5" stroke-linecap="round" />
      <line x1="-14" y1="0" x2="-14" y2="5" class="stroke-[var(--tx-hi)]" stroke-width="2.5" stroke-linecap="round" />
      <line x1="14" y1="0" x2="14" y2="5" class="stroke-[var(--tx-hi)]" stroke-width="2.5" stroke-linecap="round" />
      <circle cx="0" cy="0" r="2.5" class="fill-[var(--tx-hi)]" />

      <g :style="{ transform: `rotate(${rollDeg}deg)`, transition: 'transform 40ms linear' }">
        <polygon points="0,-43 -3,-37 3,-37" class="fill-[var(--tx-hi)]" />
      </g>

      <circle cx="0" cy="0" r="46" fill="none" class="stroke-[var(--bd-muted)]" stroke-width="1" />
    </svg>
    <div class="flex gap-2">
      <span class="text-[clamp(0.42rem,0.85vw,0.54rem)] font-bold text-[var(--tx-xdim)] tabular-nums tracking-wide">
        横滚 {{ rollDeg.toFixed(1) }}°
      </span>
      <span class="text-[clamp(0.42rem,0.85vw,0.54rem)] font-bold text-[var(--tx-xdim)] tabular-nums tracking-wide">
        俯仰 {{ pitchDeg.toFixed(1) }}°
      </span>
    </div>
  </div>
</template>