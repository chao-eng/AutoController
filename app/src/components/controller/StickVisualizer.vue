<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'

const props = defineProps<{
  modelValue: { x: number; y: number }
  size?: number
  label?: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: { x: number; y: number }]
}>()

const containerSize = computed(() => props.size || 140)
const center = computed(() => containerSize.value / 2)
const knobRadius = 12
const deadzone = 8

const isDragging = ref(false)
const knobX = ref(center.value)
const knobY = ref(center.value)

const normalizedX = computed(() => props.modelValue.x)
const normalizedY = computed(() => props.modelValue.y)

function updateKnobFromValue() {
  knobX.value = center.value + normalizedX.value * (center.value - knobRadius)
  knobY.value = center.value - normalizedY.value * (center.value - knobRadius)
}

function handlePointerDown(e: PointerEvent) {
  isDragging.value = true
  ;(e.target as HTMLElement).setPointerCapture(e.pointerId)
  handlePointerMove(e)
}

function handlePointerMove(e: PointerEvent) {
  if (!isDragging.value) return
  const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
  const x = e.clientX - rect.left - center.value
  const y = -(e.clientY - rect.top - center.value)
  const maxDist = center.value - knobRadius
  const dist = Math.sqrt(x * x + y * y)
  const clampedDist = Math.min(dist, maxDist)
  const angle = Math.atan2(y, x)
  const cx = (clampedDist / maxDist) * Math.cos(angle)
  const cy = (clampedDist / maxDist) * Math.sin(angle)
  knobX.value = center.value + cx * maxDist
  knobY.value = center.value - cy * maxDist
  emit('update:modelValue', {
    x: Math.abs(cx) < deadzone / maxDist ? 0 : Math.round(cx * 1000) / 1000,
    y: Math.abs(cy) < deadzone / maxDist ? 0 : Math.round(cy * 1000) / 1000,
  })
}

function handlePointerUp() {
  isDragging.value = false
  knobX.value = center.value
  knobY.value = center.value
  emit('update:modelValue', { x: 0, y: 0 })
}

onMounted(() => {
  updateKnobFromValue()
})

onUnmounted(() => {
  isDragging.value = false
})

watch(
  () => props.modelValue,
  () => {
    if (!isDragging.value) {
      updateKnobFromValue()
    }
  },
  { deep: true }
)
</script>

<template>
  <div class="stick-visualizer">
    <span v-if="label" class="stick-label">{{ label }}</span>
    <svg
      :width="containerSize"
      :height="containerSize"
      class="stick-svg"
      @pointerdown="handlePointerDown"
      @pointermove="handlePointerMove"
      @pointerup="handlePointerUp"
      @pointercancel="handlePointerUp"
    >
      <circle
        :cx="center"
        :cy="center"
        :r="center - 4"
        fill="none"
        stroke="var(--color-border)"
        stroke-width="1"
      />
      <circle
        :cx="center"
        :cy="center"
        :r="deadzone"
        fill="var(--color-surface-elevated)"
        opacity="0.5"
      />
      <line
        :x1="center"
        :y1="4"
        :x2="center"
        :y2="containerSize - 4"
        stroke="var(--color-border)"
        stroke-width="0.5"
        opacity="0.5"
      />
      <line
        :x1="4"
        :y1="center"
        :x2="containerSize - 4"
        :y2="center"
        stroke="var(--color-border)"
        stroke-width="0.5"
        opacity="0.5"
      />
      <circle
        :cx="knobX"
        :cy="knobY"
        :r="knobRadius"
        fill="var(--color-cta)"
        opacity="0.8"
        class="stick-knob"
      />
    </svg>
    <div class="stick-values">
      <span>X: {{ normalizedX.toFixed(2) }}</span>
      <span>Y: {{ normalizedY.toFixed(2) }}</span>
    </div>
  </div>
</template>

<style scoped>
.stick-visualizer {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-xs);
}

.stick-label {
  font-size: 11px;
  color: var(--color-text-dim);
  font-family: var(--font-heading);
}

.stick-svg {
  cursor: pointer;
  user-select: none;
  touch-action: none;
}

.stick-knob {
  transition: cx 0.05s ease, cy 0.05s ease;
}

.stick-values {
  display: flex;
  gap: var(--space-md);
  font-size: 10px;
  color: var(--color-text-dim);
  font-family: var(--font-heading);
}
</style>
