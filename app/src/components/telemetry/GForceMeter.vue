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
const gDotColor = computed(() => gMag.value > 1.5 ? '#ff3b35' : gMag.value > 0.8 ? '#ffd166' : '#30f27a')
const isOverload = computed(() => gMag.value > 1.5)
const gLoadLabel = computed(() => isOverload.value ? `超载 ${gMag.value.toFixed(1)}G` : `${gMag.value.toFixed(1)}G`)
</script>

<template>
  <div
    class="g-section"
    :class="{ 'is-overload': isOverload }"
    :style="{ '--g-color': gDotColor }"
  >
    <div class="g-circle">
      <div v-if="isOverload" class="g-alert">超载</div>
      <div v-if="isOverload" class="g-shock g-shock-1" />
      <div v-if="isOverload" class="g-shock g-shock-2" />
      <div class="g-ring g-ring-1" />
      <div class="g-ring g-ring-2" />
      <div class="g-cross g-cross-h" />
      <div class="g-cross g-cross-v" />
      <div
        class="g-dot"
        :style="{
          left: gDotX + '%',
          top: gDotY + '%',
          background: gDotColor,
          boxShadow: `0 0 9px ${gDotColor}cc`,
        }"
      />
    </div>
    <div class="g-readout">
      <span class="g-axis">
        横向 {{ Math.abs(latG).toFixed(1) }}G
      </span>
      <span class="g-axis">
        纵向 {{ Math.abs(longG).toFixed(1) }}G
      </span>
      <span class="g-load" :class="{ danger: isOverload }">
        {{ gLoadLabel }}
      </span>
    </div>
  </div>
</template>

<style scoped>
.g-section {
  --g-color: #30f27a;
  display: flex;
  align-items: center;
  gap: 0.45rem;
  flex-shrink: 0;
  color: var(--tx-dim);
}

.g-circle {
  position: relative;
  width: 4.6rem;
  height: 4.6rem;
  flex-shrink: 0;
  overflow: hidden;
  border: 1px solid var(--bd-muted);
  border-radius: 999px;
  background:
    radial-gradient(circle at 50% 50%, color-mix(in srgb, var(--g-color) 12%, transparent) 0 2px, transparent 3px),
    radial-gradient(circle, var(--bg-elevated) 0%, var(--bg-panel) 100%);
  box-shadow:
    inset 0 0 18px color-mix(in srgb, var(--g-color) 8%, transparent),
    0 2px 10px rgba(31, 35, 41, 0.06);
}

.g-circle::before,
.g-circle::after {
  content: "";
  position: absolute;
  inset: 0.22rem;
  border-radius: inherit;
  pointer-events: none;
}

.g-circle::before {
  border: 1px dashed var(--bd-subtle);
}

.g-circle::after {
  opacity: 0;
  border: 1px solid var(--g-color);
}

.g-alert {
  position: absolute;
  left: 50%;
  top: 0.45rem;
  z-index: 3;
  transform: translateX(-50%);
  padding: 0.06rem 0.28rem;
  border: 1px solid rgba(255, 59, 53, 0.72);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.9);
  color: #ef2f2a;
  font-size: 0.5rem;
  font-weight: 900;
  line-height: 1.1;
  letter-spacing: 0.04em;
  box-shadow: 0 0 12px rgba(255, 59, 53, 0.25);
  animation: overload-alert 520ms steps(2, end) infinite;
}

.g-shock {
  position: absolute;
  inset: 0.26rem;
  z-index: 0;
  border: 2px solid rgba(255, 59, 53, 0.6);
  border-radius: inherit;
  pointer-events: none;
  opacity: 0;
}

.g-shock-1 {
  animation: overload-shock 820ms ease-out infinite;
}

.g-shock-2 {
  animation: overload-shock 820ms ease-out 240ms infinite;
}

.g-ring {
  position: absolute;
  top: 50%;
  left: 50%;
  border: 1px solid var(--bd-subtle);
  border-radius: inherit;
  transform: translate(-50%, -50%);
}

.g-ring-1 {
  width: 50%;
  height: 50%;
}

.g-ring-2 {
  width: 78%;
  height: 78%;
}

.g-cross {
  position: absolute;
  background: var(--bd-subtle);
}

.g-cross-h {
  top: 50%;
  left: 7%;
  width: 86%;
  height: 1px;
  transform: translateY(-50%);
}

.g-cross-v {
  left: 50%;
  top: 7%;
  width: 1px;
  height: 86%;
  transform: translateX(-50%);
}

.g-dot {
  position: absolute;
  width: 0.68rem;
  height: 0.68rem;
  border-radius: 999px;
  transform: translate(-50%, -50%);
  transition: left 40ms linear, top 40ms linear, background 160ms ease, box-shadow 160ms ease;
  z-index: 2;
}

.g-readout {
  display: flex;
  flex-direction: column;
  gap: 0.12rem;
}

.g-axis,
.g-load {
  white-space: nowrap;
  font-size: 0.68rem;
  font-weight: 800;
  color: var(--tx-xdim);
  font-variant-numeric: tabular-nums;
}

.g-load {
  color: var(--g-color, #30f27a);
}

.g-load.danger {
  color: #ef2f2a;
  text-shadow: 0 0 10px rgba(255, 59, 53, 0.24);
}

.g-section.is-overload .g-circle {
  border-color: rgba(255, 59, 53, 0.86);
  background:
    radial-gradient(circle at 50% 50%, rgba(255, 59, 53, 0.22) 0 3px, transparent 4px),
    radial-gradient(circle, rgba(255, 59, 53, 0.12) 0%, var(--bg-panel) 68%);
  box-shadow:
    0 0 0 2px rgba(255, 59, 53, 0.15),
    0 0 22px rgba(255, 59, 53, 0.42),
    inset 0 0 26px rgba(255, 59, 53, 0.18);
  animation: overload-jolt 360ms steps(2, end) infinite;
}

.g-section.is-overload .g-circle::after {
  animation: overload-pulse 760ms ease-out infinite;
}

.g-section.is-overload .g-dot {
  box-shadow: 0 0 14px rgba(255, 59, 53, 0.95), 0 0 0 5px rgba(255, 59, 53, 0.16) !important;
  animation: overload-dot 420ms ease-in-out infinite;
}

.g-section.is-overload .g-load {
  animation: overload-text 420ms ease-in-out infinite;
}

@keyframes overload-pulse {
  0% {
    opacity: 0.75;
    transform: scale(0.88);
  }
  100% {
    opacity: 0;
    transform: scale(1.45);
  }
}

@keyframes overload-dot {
  0%, 100% {
    filter: brightness(1);
  }
  50% {
    filter: brightness(1.65);
  }
}

@keyframes overload-text {
  0%, 100% {
    opacity: 0.76;
  }
  50% {
    opacity: 1;
  }
}

@keyframes overload-alert {
  0%, 100% {
    opacity: 0.88;
    transform: translateX(-50%) scale(1);
  }
  50% {
    opacity: 1;
    transform: translateX(-50%) scale(1.08);
  }
}

@keyframes overload-shock {
  0% {
    opacity: 0.64;
    transform: scale(0.72);
  }
  100% {
    opacity: 0;
    transform: scale(1.55);
  }
}

@keyframes overload-jolt {
  0%, 100% {
    transform: translate(0, 0);
  }
  50% {
    transform: translate(1px, -1px);
  }
}
</style>
