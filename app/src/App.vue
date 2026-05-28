<script setup lang="ts">
import AppSidebar from './components/layout/AppSidebar.vue'
import AppHeader from './components/layout/AppHeader.vue'
import StatusBar from './components/layout/StatusBar.vue'
import AppDialogs from './components/layout/AppDialogs.vue'
import { useLogStore } from './stores/log'
import { useControllerStore } from './stores/controller'
import { useScriptStore } from './stores/script'
import { useSchedulerStore } from './stores/scheduler'
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const logStore = useLogStore()
const controllerStore = useControllerStore()
const scriptStore = useScriptStore()
const schedulerStore = useSchedulerStore()

const deviceCount = computed(() => controllerStore.devices.length)
const cpuUsage = ref('0%')
const memUsage = ref('0 MB')
let unlistenResources: UnlistenFn | null = null

onMounted(async () => {
  logStore.startListening()
  controllerStore.init()
  scriptStore.startListening()
  schedulerStore.startListening()

  try {
    unlistenResources = await listen<{ cpu: string; memory: string }>('system-resources', (event) => {
      cpuUsage.value = event.payload.cpu
      memUsage.value = event.payload.memory
    })
  } catch (e) {
    console.error('Failed to listen to system resources event:', e)
  }
})

onUnmounted(() => {
  logStore.stopListening()
  controllerStore.cleanup()
  scriptStore.stopListening()
  schedulerStore.stopListening()

  if (unlistenResources) {
    unlistenResources()
  }
})
</script>

<template>
  <div class="app-layout">
    <AppSidebar />
    <div class="app-main">
      <AppHeader />
      <div class="app-content">
        <router-view />
      </div>
      <StatusBar
        :deviceCount="deviceCount"
        :cpuUsage="cpuUsage"
        :memUsage="memUsage"
        version="0.1.0"
      />
    </div>
    <!-- Custom Dialogs/Toasts overlays -->
    <AppDialogs />
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100%;
  width: 100%;
}

.app-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.app-content {
  flex: 1;
  overflow: hidden;
  background: var(--color-background);
}
</style>
