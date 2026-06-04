<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
  title: string
  description?: string
  eyebrow?: string
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <header :class="cn('flex shrink-0 items-start justify-between gap-4', props.class)">
    <div class="min-w-0">
      <p v-if="eyebrow" class="mb-1 text-[11px] font-semibold uppercase text-muted-foreground">
        {{ eyebrow }}
      </p>
      <h1 class="truncate text-lg font-semibold leading-tight text-foreground">
        {{ title }}
      </h1>
      <p v-if="description" class="mt-1 max-w-2xl text-xs leading-relaxed text-muted-foreground">
        {{ description }}
      </p>
      <slot name="meta" />
    </div>
    <div v-if="$slots.actions" class="flex shrink-0 flex-wrap items-center justify-end gap-2">
      <slot name="actions" />
    </div>
  </header>
</template>
