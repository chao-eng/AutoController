<script setup lang="ts">
import { watch, onUnmounted } from 'vue'
import { useTelemetryStore } from '@/stores/telemetry'

const telemetry = useTelemetryStore()

const SPEEDS = [0.5, 1, 2, 4]
let timer: ReturnType<typeof setInterval> | null = null
let carry = 0

function clearTimer() {
  if (timer) {
    clearInterval(timer)
    timer = null
  }
}

function ensureLoop(playing: boolean) {
  clearTimer()
  carry = 0
  if (!playing) return
  timer = setInterval(() => {
    const r = telemetry.replay
    if (!r.active) return
    carry += r.speed
    const step = Math.floor(carry)
    carry -= step
    const next = r.index + step
    if (next >= r.packets.length - 1) {
      clearTimer()
      telemetry.replay.index = r.packets.length - 1
      telemetry.replay.playing = false
    } else {
      telemetry.replay.index = next
    }
  }, 1000 / 60)
}

watch(
  () => telemetry.replay.playing,
  (playing) => {
    ensureLoop(playing)
  }
)

onUnmounted(clearTimer)

function togglePlay() {
  const r = telemetry.replay
  const atEnd = r.index >= r.packets.length - 1
  telemetry.replay.playing = !r.playing
  if (atEnd) telemetry.replay.index = 0
}

function scrub(e: Event) {
  const v = Number((e.target as HTMLInputElement).value)
  telemetry.replay.index = v
  telemetry.replay.playing = false
}

function setSpeed(s: number) {
  telemetry.replay.speed = s
}

function fmt(idx: number) {
  const sec = idx / 60
  const m = Math.floor(sec / 60)
  const s = (sec % 60).toFixed(1).padStart(4, '0')
  return `${m}:${s}`
}
</script>

<template>
  <div v-if="telemetry.replay.active" class="replay-bar">
    <div class="left">
      <span class="badge">数据回放</span>
      <span class="label" :title="telemetry.replay.label">{{ telemetry.replay.label }}</span>
    </div>

    <div class="controls">
      <button class="play" @click="togglePlay">
        {{ telemetry.replay.playing ? '⏸' : '▶' }}
      </button>
      <span class="time">{{ fmt(telemetry.replay.index) }}</span>
      <input
        class="scrub"
        type="range"
        :min="0"
        :max="Math.max(telemetry.replay.packets.length - 1, 0)"
        :value="telemetry.replay.index"
        @input="scrub"
      />
      <span class="time">{{ fmt(Math.max(telemetry.replay.packets.length - 1, 0)) }}</span>
      <div class="speeds">
        <button
          v-for="s in SPEEDS" :key="s"
          :class="{ active: telemetry.replay.speed === s }"
          @click="setSpeed(s)"
        >
          {{ s }}×
        </button>
      </div>
    </div>

    <button class="exit" @click="telemetry.exitReplay()">退出回放</button>
  </div>
</template>

<style scoped>
.replay-bar {
  position: fixed;
  left: 0; right: 0; bottom: 0;
  z-index: 110;
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.5rem 1rem;
  background: var(--bg-panel);
  border-top: 1px solid var(--ac);
  box-shadow: 0 -4px 20px rgba(31,35,41,0.08);
}
.left {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  min-width: 0;
  flex: 0 1 240px;
}
.badge {
  background: var(--ac); color: #fff;
  font-size: 0.65rem; font-weight: 700;
  letter-spacing: 0.06em; padding: 0.15rem 0.4rem; border-radius: 3px;
}
.label {
  color: var(--tx-lo); font-size: 0.78rem;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.controls {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 0.6rem;
}
.play {
  background: var(--ac); color: #fff; border: none;
  border-radius: 50%; width: 2rem; height: 2rem;
  font-size: 0.85rem; cursor: pointer;
  flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  line-height: 1; padding: 0;
}
.time {
  color: var(--tx-dim); font-size: 0.72rem;
  font-variant-numeric: tabular-nums;
  min-width: 3rem; text-align: center;
}
.scrub { flex: 1; accent-color: var(--ac); cursor: pointer; }
.speeds { display: flex; gap: 0.2rem; }
.speeds button {
  background: var(--bg-elevated);
  border: 1px solid var(--bd-dim);
  color: var(--tx-dim);
  font-size: 0.7rem;
  padding: 0.2rem 0.4rem;
  border-radius: 4px;
  cursor: pointer;
}
.speeds button.active { border-color: var(--ac); color: var(--tx-hi); }
.exit {
  background: none;
  border: 1px solid var(--bd-subtle);
  color: var(--tx-lo);
  font-size: 0.75rem;
  padding: 0.35rem 0.7rem;
  border-radius: 4px;
  cursor: pointer;
}
.exit:hover { border-color: var(--bd-muted); }
</style>