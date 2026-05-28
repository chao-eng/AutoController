<script setup lang="ts">
import { computed } from 'vue'

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

function handlePointerDown(e: PointerEvent) {
  const el = e.currentTarget as HTMLElement
  el.setPointerCapture(e.pointerId)
  updateValue(e)
}

function updateValue(e: PointerEvent) {
  const el = e.currentTarget as HTMLElement
  const rect = el.getBoundingClientRect()
  const x = e.clientX - rect.left
  const pct = Math.max(0, Math.min(1, x / rect.width))
  emit('update:modelValue', Math.round(pct * maxVal))
}

function handlePointerMove(e: PointerEvent) {
  const el = e.currentTarget as HTMLElement
  if (el.hasPointerCapture(e.pointerId)) {
    updateValue(e)
  }
}
</script>

<template>
  <div class="trigger-bar-container">
    <span v-if="label" class="trigger-label">{{ label }}</span>
    <div
      class="trigger-track"
      @pointerdown="handlePointerDown"
      @pointermove="handlePointerMove"
    >
      <div
        class="trigger-fill"
        :style="{ width: percentage + '%' }"
      ></div>
    </div>
    <span class="trigger-value">{{ modelValue }}</span>
  </div>
</template>

<style scoped>
.trigger-bar-container {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  user-select: none;
}

.trigger-label {
  font-size: 11px;
  color: var(--color-text-dim);
  font-family: var(--font-heading);
  min-width: 38px;
}

.trigger-track {
  flex: 1;
  height: 12px;
  background: var(--color-surface-elevated);
  border-radius: 6px;
  overflow: hidden;
  cursor: ew-resize;
  position: relative;
  border: 1px solid var(--color-border);
}

.trigger-fill {
  height: 100%;
  background: var(--color-cta);
  border-radius: 5px;
  transition: width 0.05s ease;
}

.trigger-value {
  font-size: 11px;
  color: var(--color-text-muted);
  font-family: var(--font-heading);
  min-width: 28px;
  text-align: right;
}
</style>
