<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = withDefaults(defineProps<{
  tone?: 'neutral' | 'info' | 'success' | 'warning' | 'danger'
  title?: string
  description?: string
  class?: HTMLAttributes['class']
}>(), {
  tone: 'neutral',
})
</script>

<template>
  <div
    :data-tone="tone"
    :class="cn('status-banner flex items-start gap-2 rounded-lg border px-3 py-2 text-xs', props.class)"
  >
    <div class="mt-0.5 flex size-4 shrink-0 items-center justify-center">
      <slot name="icon">
        <span class="status-dot size-1.5 rounded-full" />
      </slot>
    </div>
    <div class="min-w-0 flex-1">
      <p v-if="title" class="font-semibold leading-5">
        {{ title }}
      </p>
      <p v-if="description" class="leading-5 opacity-90">
        {{ description }}
      </p>
      <slot />
    </div>
    <div v-if="$slots.actions" class="ml-2 flex shrink-0 items-center gap-2">
      <slot name="actions" />
    </div>
  </div>
</template>

<style scoped>
.status-banner {
  background: var(--status-neutral-bg);
  border-color: var(--status-neutral-border);
  color: var(--status-neutral-text);
}

.status-banner[data-tone="info"] {
  background: var(--status-info-bg);
  border-color: var(--status-info-border);
  color: var(--status-info-text);
}

.status-banner[data-tone="success"] {
  background: var(--status-success-bg);
  border-color: var(--status-success-border);
  color: var(--status-success-text);
}

.status-banner[data-tone="warning"] {
  background: var(--status-warning-bg);
  border-color: var(--status-warning-border);
  color: var(--status-warning-text);
}

.status-banner[data-tone="danger"] {
  background: var(--status-danger-bg);
  border-color: var(--status-danger-border);
  color: var(--status-danger-text);
}

.status-dot {
  background: currentColor;
}
</style>
