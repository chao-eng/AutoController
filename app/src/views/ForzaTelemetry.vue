<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue';
// @ts-ignore
import { mount, unmount } from 'svelte';
// @ts-ignore
import ForzaDashboard from '../fh6-tel/views/ForzaDashboard.svelte';

const containerRef = ref<HTMLElement | null>(null);
let svelteApp: any = null;

onMounted(() => {
  if (containerRef.value) {
    svelteApp = mount(ForzaDashboard, {
      target: containerRef.value,
      props: {}
    });
  }
});

onBeforeUnmount(() => {
  if (svelteApp) {
    unmount(svelteApp);
    svelteApp = null;
  }
});
</script>

<template>
  <div ref="containerRef" class="svelte-container"></div>
</template>

<style scoped>
.svelte-container {
  width: 100%;
  height: 100%;
  overflow: hidden;
}
</style>
