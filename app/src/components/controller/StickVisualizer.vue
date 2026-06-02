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
  <div class="flex flex-col items-center gap-1">
    <span v-if="label" class="text-[11px] text-muted-foreground">{{ label }}</span>
    <svg
      :width="containerSize"
      :height="containerSize"
      class="touch-none cursor-pointer select-none"
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
        stroke="var(--border)"
        stroke-width="1"
      />
      <circle
        :cx="center"
        :cy="center"
        :r="deadzone"
        fill="var(--muted)"
        opacity="0.5"
      />
      <line
        :x1="center"
        :y1="4"
        :x2="center"
        :y2="containerSize - 4"
        stroke="var(--border)"
        stroke-width="0.5"
        opacity="0.5"
      />
      <line
        :x1="4"
        :y1="center"
        :x2="containerSize - 4"
        :y2="center"
        stroke="var(--border)"
        stroke-width="0.5"
        opacity="0.5"
      />
      <circle
        :cx="knobX"
        :cy="knobY"
        :r="knobRadius"
        fill="var(--primary)"
        opacity="0.8"
        class="transition-[cx,cy] duration-50"
      />
    </svg>
    <div class="flex gap-4 font-mono text-[10px] text-muted-foreground">
      <span>X: {{ normalizedX.toFixed(2) }}</span>
      <span>Y: {{ normalizedY.toFixed(2) }}</span>
    </div>
  </div>
</template>