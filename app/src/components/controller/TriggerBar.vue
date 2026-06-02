<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps<{
  modelValue: number
  label?: string
  max?: number
}>()

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

const maxVal = props.max || 255
const percentage = computed(() => ((props.modelValue / maxVal) * 100).toFixed(0))

const trackRef = ref<HTMLElement | null>(null)

function handlePointerDown(e: PointerEvent) {
  if (!trackRef.value) return
  trackRef.value.setPointerCapture(e.pointerId)
  updateValue(e)
}

function updateValue(e: PointerEvent) {
  if (!trackRef.value) return
  const rect = trackRef.value.getBoundingClientRect()
  const x = e.clientX - rect.left
  const pct = Math.max(0, Math.min(1, x / rect.width))
  emit('update:modelValue', Math.round(pct * maxVal))
}

function handlePointerMove(e: PointerEvent) {
  if (!trackRef.value) return
  if (trackRef.value.hasPointerCapture(e.pointerId)) {
    updateValue(e)
  }
}
</script>

<template>
  <div class="flex items-center gap-2 select-none">
    <span v-if="label" class="min-w-[38px] text-[11px] text-muted-foreground">{{ label }}</span>
    <div
      ref="trackRef"
      class="relative h-3 flex-1 cursor-ew-resize overflow-hidden rounded-full border border-border bg-muted touch-none"
      @pointerdown="handlePointerDown"
      @pointermove="handlePointerMove"
    >
      <div
        class="h-full rounded-[5px] bg-primary transition-[width] duration-50"
        :style="{ width: percentage + '%' }"
      ></div>
    </div>
    <span class="min-w-[28px] text-right font-mono text-[11px] text-muted-foreground">{{ modelValue }}</span>
  </div>
</template>