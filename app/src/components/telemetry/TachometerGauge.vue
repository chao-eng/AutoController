<script setup lang="ts">
import { computed } from 'vue'
import { useTelemetryStore } from '@/stores/telemetry'
import GForceMeter from './GForceMeter.vue'
import AttitudeIndicator from './AttitudeIndicator.vue'
import SteeringWheel from './SteeringWheel.vue'

const props = withDefaults(defineProps<{ useMph?: boolean; maxSpeed?: number }>(), {
  useMph: true,
  maxSpeed: 180,
})

const telemetry = useTelemetryStore()
const pkt = computed(() => telemetry.displayPacket)
const speed = computed(() => props.useMph ? Math.round(telemetry.speedMph) : Math.round(telemetry.speedKph))
const unit = computed(() => props.useMph ? 'MPH' : 'Km/h')
const rpm = computed(() => Math.min(Math.max(telemetry.rpmPercent, 0), 100))
const isRedline = computed(() => rpm.value > 90)
const isGearTooHigh = computed(() => {
  const p = pkt.value
  if (!p) return false

  const isDriveGear = p.gear >= 2 && p.gear !== 11
  const lowRpm = rpm.value < 34
  const heavyThrottle = throttleFrac.value > 0.55
  const moving = p.speedMs > 8
  const notBraking = brakeFrac.value < 0.1
  const clutchEngaged = clutchFrac.value < 0.2

  return isDriveGear && lowRpm && heavyThrottle && moving && notBraking && clutchEngaged
})

const gearLabel = computed(() => {
  if (!pkt.value) return '—'
  if (pkt.value.gear === 0) return 'R'
  if (pkt.value.gear === 11) return 'N'
  return String(pkt.value.gear)
})

const CX = 180
const CY = 190
const R = 134
const START_ANGLE = 160
const SWEEP_ANGLE = 220
const CIRCLE_LENGTH = 2 * Math.PI * R
const ARC_LENGTH = CIRCLE_LENGTH * (SWEEP_ANGLE / 360)
const SPEED_LABEL_MIN = 20
const SPEED_SCALE_DEFAULT_MAX = 180

function polarPoint(angleDeg: number, radius: number) {
  const angle = angleDeg * Math.PI / 180
  return {
    x: CX + radius * Math.cos(angle),
    y: CY + radius * Math.sin(angle),
  }
}

const speedScaleMax = computed(() => {
  const max = props.maxSpeed
  return Math.max(SPEED_LABEL_MIN + 20, Number.isFinite(max) ? Math.round(max) : SPEED_SCALE_DEFAULT_MAX)
})

const speedProgress = computed(() => {
  if (speed.value <= 0) return 0
  return Math.min(Math.max((speed.value - SPEED_LABEL_MIN) / (speedScaleMax.value - SPEED_LABEL_MIN), 0), 1)
})
const speedArc = computed(() => ARC_LENGTH * speedProgress.value)

const speedLabelStep = computed(() => {
  const range = speedScaleMax.value - SPEED_LABEL_MIN
  if (range <= 180) return 20
  if (range <= 360) return 40
  if (range <= 540) return 60
  return 100
})

const tickMarks = computed(() => {
  const step = speedLabelStep.value / 4
  const marks = []
  const max = speedScaleMax.value
  for (let value = SPEED_LABEL_MIN, index = 0; value <= max; value += step, index += 1) {
    const pct = (value - SPEED_LABEL_MIN) / (max - SPEED_LABEL_MIN)
    const angle = START_ANGLE + pct * SWEEP_ANGLE
    const major = index % 4 === 0 || value === max
    const inner = polarPoint(angle, major ? 108 : 118)
    const outer = polarPoint(angle, 132)
    marks.push({
      value,
      major,
      x1: inner.x,
      y1: inner.y,
      x2: outer.x,
      y2: outer.y,
    })
  }
  return marks
})

const labelMarks = computed(() => {
  const values = []
  const max = speedScaleMax.value
  const step = speedLabelStep.value
  for (let value = SPEED_LABEL_MIN; value <= max; value += step) {
    values.push(value)
  }
  if (values[values.length - 1] !== max) {
    if (max - values[values.length - 1] < step / 2) values[values.length - 1] = max
    else values.push(max)
  }
  return values.map((value) => {
    const pct = (value - SPEED_LABEL_MIN) / (max - SPEED_LABEL_MIN)
    const angle = START_ANGLE + pct * SWEEP_ANGLE
    const point = polarPoint(angle, 86)
    return { value, x: point.x, y: point.y }
  })
})

const boost = computed(() => pkt.value?.boost ?? 0)
const boostActive = computed(() => boost.value > 0.5)
const throttleFrac = computed(() => (pkt.value?.throttle ?? 0) / 255)
const brakeFrac = computed(() => (pkt.value?.brake ?? 0) / 255)
const clutchFrac = computed(() => (pkt.value?.clutch ?? 0) / 255)
const handbrakeOn = computed(() => (pkt.value?.handbrake ?? 0) > 127)
const rpmReadout = computed(() => pkt.value ? Math.round(pkt.value.currentEngineRpm).toLocaleString() : '—')
</script>

<template>
  <div class="dash-cluster" :class="{ 'is-redline': isRedline, 'is-gear-too-high': isGearTooHigh }">
    <div class="speed-stage">
      <svg viewBox="0 0 360 260" class="speedometer" aria-label="Forza speedometer">
        <circle
          class="arc-base"
          :cx="CX"
          :cy="CY"
          :r="R"
          :stroke-dasharray="`${ARC_LENGTH} ${CIRCLE_LENGTH - ARC_LENGTH}`"
          :transform="`rotate(${START_ANGLE}, ${CX}, ${CY})`"
        />
        <circle
          class="arc-progress"
          :cx="CX"
          :cy="CY"
          :r="R"
          :stroke-dasharray="`${speedArc} ${CIRCLE_LENGTH - speedArc}`"
          :transform="`rotate(${START_ANGLE}, ${CX}, ${CY})`"
        />
        <template v-for="tick in tickMarks" :key="tick.value">
          <line
            class="speed-tick"
            :class="{ major: tick.major, lit: tick.value <= speed }"
            :x1="tick.x1"
            :y1="tick.y1"
            :x2="tick.x2"
            :y2="tick.y2"
          />
        </template>

        <template v-for="label in labelMarks" :key="label.value">
          <text class="speed-label" :x="label.x" :y="label.y" text-anchor="middle" dominant-baseline="middle">
            {{ label.value }}
          </text>
        </template>

        <g v-if="isRedline" class="overload-hud">
          <circle class="overload-wave overload-wave-1" :cx="CX" :cy="CY" r="52" />
          <circle class="overload-wave overload-wave-2" :cx="CX" :cy="CY" r="52" />
        </g>
        <g v-else-if="isGearTooHigh" class="shift-hud">
          <circle class="shift-wave shift-wave-1" :cx="CX" :cy="CY" r="52" />
          <circle class="shift-wave shift-wave-2" :cx="CX" :cy="CY" r="52" />
          <path class="shift-chevron" d="M 169 167 L 180 178 L 191 167" />
          <path class="shift-chevron shift-chevron-2" d="M 169 181 L 180 192 L 191 181" />
        </g>
        <circle class="center-ring" :cx="CX" :cy="CY" r="48" />
        <text class="speed-value" :x="CX" :y="CY - 3" text-anchor="middle">
          {{ speed }}
        </text>
        <text class="speed-unit" :x="CX" :y="CY + 29" text-anchor="middle">
          {{ unit }}
        </text>
        <text v-if="isRedline" class="overload-label" :x="CX" :y="CY + 55" text-anchor="middle">
          超载
        </text>
        <text v-else-if="isGearTooHigh" class="shift-label" :x="CX" :y="CY + 55" text-anchor="middle">
          降挡
        </text>
      </svg>
    </div>

    <div class="control-deck">
      <div class="support-gauges">
        <GForceMeter />
        <AttitudeIndicator />
        <SteeringWheel />
      </div>

      <div class="pedal-stack">
        <div class="input-row">
          <span class="input-label">油门</span>
          <div class="input-track">
            <div class="input-fill thr" :style="{ width: throttleFrac * 100 + '%' }" />
          </div>
          <span class="input-val">{{ pkt ? Math.round(throttleFrac * 100) : '—' }}</span>
        </div>

        <div class="input-row">
          <span class="input-label">刹车</span>
          <div class="input-track">
            <div class="input-fill brk" :style="{ width: brakeFrac * 100 + '%' }" />
          </div>
          <span class="input-val">{{ pkt ? Math.round(brakeFrac * 100) : '—' }}</span>
        </div>

        <div class="input-row">
          <span class="input-label">离合</span>
          <div class="input-track">
            <div class="input-fill clt" :style="{ width: clutchFrac * 100 + '%' }" />
          </div>
          <span class="input-val">{{ pkt ? Math.round(clutchFrac * 100) : '—' }}</span>
        </div>

        <div class="status-row">
          <div class="status-chip">
            <span class="status-label">挡位</span>
            <span class="gear-pill" :class="{ warning: isGearTooHigh }">{{ gearLabel }}</span>
            <span class="shift-note" :class="{ active: isGearTooHigh }" :aria-hidden="!isGearTooHigh">拖挡</span>
          </div>

          <div class="status-chip">
            <span class="status-label">手刹</span>
            <span class="status-led" :class="{ 'hb-on': handbrakeOn }" />
            <span class="status-value" :class="{ active: handbrakeOn }">
              {{ handbrakeOn ? '开' : '关' }}
            </span>
          </div>

          <div class="status-chip">
            <span class="status-label">增压</span>
            <span class="status-led boost" :class="{ 'bst-on': boostActive }" />
            <span class="status-value" :class="{ active: boostActive }">
              {{ pkt ? boost.toFixed(1) : '—' }}
            </span>
            <span class="status-unit">PSI</span>
          </div>

          <div class="rpm-readout">
            <span class="rpm-num">{{ rpmReadout }}</span>
            <span class="rpm-unit">RPM</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dash-cluster {
  --speed-accent: #008f5f;
  --speed-accent-glow: rgba(0, 143, 95, 0.34);
  --speed-warning: #ef4444;
  --shift-warning: #f59e0b;
  --shift-warning-glow: rgba(245, 158, 11, 0.38);
  --speed-idle: #d8dce0;
  position: relative;
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  overflow: hidden;
  background: var(--bg-body);
}

.dash-cluster::after {
  content: "";
  position: absolute;
  inset: 0;
  pointer-events: none;
  background:
    radial-gradient(circle at 50% 45%, rgba(239, 68, 68, 0.12), transparent 30%),
    linear-gradient(90deg, rgba(239, 68, 68, 0.08), transparent 28%, transparent 72%, rgba(239, 68, 68, 0.08));
  opacity: 0;
  transition: opacity 100ms ease;
}

.dash-cluster.is-redline::after {
  opacity: 1;
  animation: overload-screen-flash 520ms steps(2, end) infinite;
}

.dash-cluster.is-gear-too-high:not(.is-redline)::after {
  background:
    radial-gradient(circle at 50% 45%, rgba(245, 158, 11, 0.13), transparent 30%),
    linear-gradient(90deg, rgba(245, 158, 11, 0.08), transparent 28%, transparent 72%, rgba(245, 158, 11, 0.08));
  opacity: 1;
  animation: shift-screen-pulse 900ms ease-in-out infinite;
}

.speed-stage {
  position: relative;
  z-index: 1;
  flex: 1;
  min-height: 0;
  display: flex;
  justify-content: center;
  padding: clamp(0.4rem, 2vh, 1rem) 0.75rem 0;
}

.is-redline .speed-stage {
  animation: overload-panel-jolt 360ms steps(2, end) infinite;
}

.speedometer {
  display: block;
  width: min(100%, 560px);
  height: 100%;
  overflow: visible;
}

.arc-base,
.arc-progress {
  fill: none;
  stroke-linecap: butt;
}

.arc-base {
  stroke: var(--speed-idle);
  stroke-width: 2.4;
}

.arc-progress {
  stroke: var(--speed-accent);
  stroke-width: 5;
  opacity: 1;
  filter: drop-shadow(0 0 4px var(--speed-accent-glow));
  transition: stroke-dasharray 60ms linear;
}

.speed-tick {
  stroke: var(--speed-idle);
  stroke-width: 1.7;
  stroke-linecap: square;
  transition: stroke 60ms linear, stroke-width 60ms linear, filter 60ms linear;
}

.speed-tick.major {
  stroke-width: 2.3;
}

.speed-tick.lit {
  stroke: var(--speed-accent);
  stroke-width: 2;
  filter: drop-shadow(0 0 2px var(--speed-accent-glow));
}

.speed-tick.major.lit {
  stroke-width: 2.7;
}

.speed-label {
  fill: var(--tx-hi);
  font-family: var(--font-heading), "DIN Condensed", "Arial Narrow", sans-serif;
  font-size: 12px;
  font-weight: 800;
  font-variant-numeric: tabular-nums;
}

.center-ring {
  fill: color-mix(in srgb, var(--bg-body) 84%, white);
  stroke: color-mix(in srgb, var(--tx-hi) 12%, transparent);
  stroke-width: 4;
}

.is-redline .center-ring {
  stroke: var(--speed-warning);
  stroke-width: 5;
  filter: drop-shadow(0 0 10px rgba(239, 68, 68, 0.38));
  animation: overload-ring 520ms ease-in-out infinite;
}

.is-gear-too-high:not(.is-redline) .center-ring {
  stroke: var(--shift-warning);
  stroke-width: 5;
  filter: drop-shadow(0 0 10px var(--shift-warning-glow));
  animation: shift-ring 900ms ease-in-out infinite;
}

.speed-value {
  fill: var(--tx-hi);
  font-family: var(--font-heading), "DIN Condensed", "Arial Narrow", sans-serif;
  font-size: 37px;
  font-weight: 900;
  font-variant-numeric: tabular-nums;
}

.is-redline .speed-value {
  fill: var(--speed-warning);
  animation: overload-text-flash 420ms steps(2, end) infinite;
}

.speed-unit {
  fill: var(--tx-xdim);
  font-family: var(--font-heading), "DIN Condensed", "Arial Narrow", sans-serif;
  font-size: 14px;
  font-weight: 800;
}

.is-redline .speed-unit {
  fill: #f87171;
}

.overload-wave {
  fill: none;
  stroke: var(--speed-warning);
  stroke-width: 3;
  transform-origin: 180px 190px;
  opacity: 0;
  filter: drop-shadow(0 0 9px rgba(239, 68, 68, 0.48));
}

.overload-wave-1 {
  animation: overload-wave 820ms ease-out infinite;
}

.overload-wave-2 {
  animation: overload-wave 820ms ease-out 240ms infinite;
}

.overload-label {
  fill: var(--speed-warning);
  font-family: var(--font-heading), "DIN Condensed", "Arial Narrow", sans-serif;
  font-size: 12px;
  font-weight: 900;
  letter-spacing: 0.08em;
  paint-order: stroke;
  stroke: rgba(255, 255, 255, 0.75);
  stroke-width: 2px;
  animation: overload-text-flash 420ms steps(2, end) infinite;
}

.shift-wave {
  fill: none;
  stroke: var(--shift-warning);
  stroke-width: 2.6;
  transform-origin: 180px 190px;
  opacity: 0;
  filter: drop-shadow(0 0 8px var(--shift-warning-glow));
}

.shift-wave-1 {
  animation: shift-wave 1100ms ease-out infinite;
}

.shift-wave-2 {
  animation: shift-wave 1100ms ease-out 320ms infinite;
}

.shift-chevron {
  fill: none;
  stroke: var(--shift-warning);
  stroke-width: 5;
  stroke-linecap: round;
  stroke-linejoin: round;
  filter: drop-shadow(0 0 6px var(--shift-warning-glow));
  animation: shift-chevron 680ms ease-in-out infinite;
}

.shift-chevron-2 {
  opacity: 0.62;
  animation-delay: 110ms;
}

.shift-label {
  fill: var(--shift-warning);
  font-family: var(--font-heading), "DIN Condensed", "Arial Narrow", sans-serif;
  font-size: 12px;
  font-weight: 900;
  letter-spacing: 0;
  paint-order: stroke;
  stroke: rgba(255, 255, 255, 0.75);
  stroke-width: 2px;
  animation: shift-text-pulse 680ms ease-in-out infinite;
}

.control-deck {
  display: grid;
  grid-template-columns: auto minmax(15rem, 1fr);
  gap: 1rem;
  align-items: end;
  flex-shrink: 0;
  width: 100%;
  padding: 0 1rem 0.85rem;
}

.support-gauges {
  display: flex;
  align-items: center;
  gap: 0.85rem;
  min-width: 0;
}

.pedal-stack {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
  min-width: 0;
  padding: 0.7rem 0.8rem;
  border: 1px solid var(--bd-dim);
  border-radius: 8px;
  background: var(--bg-panel);
  box-shadow: 0 4px 18px rgba(31, 35, 41, 0.06);
}

.input-row {
  display: grid;
  grid-template-columns: 2.3rem minmax(0, 1fr) 2.4rem;
  align-items: center;
  gap: 0.55rem;
}

.input-label,
.input-val,
.status-label,
.status-value,
.status-unit,
.rpm-unit {
  font-size: 0.76rem;
  font-weight: 800;
  color: var(--tx-xdim);
  font-variant-numeric: tabular-nums;
}

.input-track {
  height: 0.68rem;
  overflow: hidden;
  border: 1px solid var(--bd-subtle);
  border-radius: 999px;
  background: var(--bg-track);
}

.input-fill {
  height: 100%;
  border-radius: inherit;
  transition: width 40ms linear;
}

.input-fill.thr {
  background: #22c55e;
}

.input-fill.brk {
  background: #ef4444;
}

.input-fill.clt {
  background: #64748b;
}

.input-val {
  text-align: right;
}

.status-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.7rem 1rem;
  padding-top: 0.1rem;
}

.status-chip,
.rpm-readout {
  display: flex;
  align-items: center;
  gap: 0.45rem;
}

.gear-pill {
  min-width: 2rem;
  padding: 0.08rem 0.45rem;
  border: 1px solid var(--bd-muted);
  border-radius: 6px;
  color: var(--tx-mid);
  background: var(--bg-elevated);
  text-align: center;
  font-size: 1rem;
  font-weight: 900;
  line-height: 1.2;
  font-variant-numeric: tabular-nums;
}

.gear-pill.warning {
  border-color: rgba(245, 158, 11, 0.72);
  color: #b45309;
  background: rgba(245, 158, 11, 0.13);
  box-shadow:
    0 0 0 1px rgba(245, 158, 11, 0.08),
    0 0 12px rgba(245, 158, 11, 0.28);
  animation: shift-pill-pulse 680ms ease-in-out infinite;
}

.shift-note {
  width: 2rem;
  color: var(--shift-warning);
  font-size: 0.68rem;
  font-weight: 900;
  line-height: 1;
  opacity: 0;
  transition: opacity 160ms ease;
}

.shift-note.active {
  opacity: 1;
  animation: shift-text-pulse 680ms ease-in-out infinite;
}

.status-led {
  width: 0.68rem;
  height: 0.68rem;
  flex-shrink: 0;
  border: 1px solid var(--bd-muted);
  border-radius: 999px;
  background: var(--bg-elevated);
}

.status-led.hb-on {
  border-color: #f97316;
  background: #f97316;
  box-shadow: 0 0 8px rgba(249, 115, 22, 0.45);
}

.status-led.bst-on {
  border-color: #f59e0b;
  background: #f59e0b;
  box-shadow: 0 0 8px rgba(245, 158, 11, 0.45);
}

.status-value.active {
  color: #f97316;
}

.rpm-readout {
  margin-left: auto;
}

.rpm-num {
  color: var(--tx-hi);
  font-size: 1rem;
  font-weight: 900;
  font-variant-numeric: tabular-nums;
}

.is-redline .rpm-num,
.is-redline .rpm-unit {
  color: var(--speed-warning);
  animation: overload-text-flash 420ms steps(2, end) infinite;
}

.is-redline .pedal-stack {
  border-color: rgba(239, 68, 68, 0.32);
  box-shadow:
    0 0 0 1px rgba(239, 68, 68, 0.08),
    0 8px 22px rgba(239, 68, 68, 0.12);
}

@keyframes overload-wave {
  0% {
    opacity: 0.58;
    transform: scale(0.84);
  }
  100% {
    opacity: 0;
    transform: scale(1.75);
  }
}

@keyframes overload-ring {
  0%, 100% {
    opacity: 0.9;
  }
  50% {
    opacity: 1;
    filter: drop-shadow(0 0 16px rgba(239, 68, 68, 0.55));
  }
}

@keyframes overload-text-flash {
  0%, 100% {
    opacity: 0.78;
  }
  50% {
    opacity: 1;
  }
}

@keyframes overload-panel-jolt {
  0%, 100% {
    transform: translate(0, 0);
  }
  50% {
    transform: translate(1px, -1px);
  }
}

@keyframes overload-screen-flash {
  0%, 100% {
    opacity: 0.34;
  }
  50% {
    opacity: 0.58;
  }
}

@keyframes shift-wave {
  0% {
    opacity: 0.48;
    transform: scale(0.86);
  }
  100% {
    opacity: 0;
    transform: scale(1.55);
  }
}

@keyframes shift-ring {
  0%, 100% {
    opacity: 0.88;
  }
  50% {
    opacity: 1;
    filter: drop-shadow(0 0 15px var(--shift-warning-glow));
  }
}

@keyframes shift-chevron {
  0%, 100% {
    opacity: 0.62;
    transform: translateY(0);
  }
  50% {
    opacity: 1;
    transform: translateY(4px);
  }
}

@keyframes shift-text-pulse {
  0%, 100% {
    opacity: 0.7;
  }
  50% {
    opacity: 1;
  }
}

@keyframes shift-pill-pulse {
  0%, 100% {
    filter: brightness(1);
  }
  50% {
    filter: brightness(1.18);
  }
}

@keyframes shift-screen-pulse {
  0%, 100% {
    opacity: 0.28;
  }
  50% {
    opacity: 0.42;
  }
}

@media (max-width: 760px) {
  .control-deck {
    grid-template-columns: 1fr;
    gap: 0.7rem;
  }

  .support-gauges {
    justify-content: space-between;
  }
}
</style>
