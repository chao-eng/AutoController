<script setup lang="ts">
import TachometerGauge from './TachometerGauge.vue'

defineProps<{ useMph?: boolean; maxSpeed?: number }>()
</script>

<template>
  <div class="flex flex-col h-full min-h-0 overflow-hidden">
    <TachometerGauge :use-mph="useMph" :max-speed="maxSpeed" />
  </div>
</template>
