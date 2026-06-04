<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
  title: string
  description?: string
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div
    :class="cn(
      'flex flex-col items-center justify-center rounded-lg border border-dashed border-border bg-card/40 px-6 py-10 text-center',
      props.class,
    )"
  >
    <div v-if="$slots.icon" class="mb-3 text-muted-foreground/70">
      <slot name="icon" />
    </div>
    <h3 class="text-sm font-semibold text-foreground">
      {{ title }}
    </h3>
    <p v-if="description" class="mt-1 max-w-md text-xs leading-relaxed text-muted-foreground">
      {{ description }}
    </p>
    <div v-if="$slots.actions" class="mt-4 flex items-center justify-center gap-2">
      <slot name="actions" />
    </div>
  </div>
</template>
