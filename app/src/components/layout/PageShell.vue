<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = withDefaults(defineProps<{
  class?: HTMLAttributes['class']
  scroll?: boolean
}>(), {
  scroll: true,
})
</script>

<template>
  <section
    :class="cn(
      'flex h-full min-h-0 flex-col gap-4 bg-background p-4 lg:p-6',
      props.scroll ? 'overflow-y-auto' : 'overflow-hidden',
      props.class,
    )"
  >
    <slot />
  </section>
</template>
