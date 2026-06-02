<script setup lang="ts">
import AppSidebar from './components/layout/AppSidebar.vue'
import AppHeader from './components/layout/AppHeader.vue'
import StatusBar from './components/layout/StatusBar.vue'
import AppDialogs from './components/layout/AppDialogs.vue'
import { useLogStore } from './stores/log'
import { useControllerStore } from './stores/controller'
import { useScriptStore } from './stores/script'
import { useSchedulerStore } from './stores/scheduler'
import { useUIStore } from './stores/ui'
import { open } from '@tauri-apps/plugin-shell'
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

const route = useRoute()
const isMapWindow = computed(() => route.path === '/forza-map')
const isForzaTelemetry = computed(() => route.path === '/forza-telemetry')

const logStore = useLogStore()
const controllerStore = useControllerStore()
const scriptStore = useScriptStore()
const schedulerStore = useSchedulerStore()
const uiStore = useUIStore()

const deviceCount = computed(() => controllerStore.devices.length)
const cpuUsage = ref('0%')
const memUsage = ref('0 MB')
const appVersion = __APP_VERSION__   // 由 Vite define 在构建时从 package.json 自动注入
let unlistenResources: UnlistenFn | null = null

// 比较语义化版本号，若 latest > current 则返回 true
function isNewerVersion(current: string, latest: string): boolean {
  const clean = (v: string) => v.replace(/^[vV]/, '').trim()
  const curParts = clean(current).split('.').map(Number)
  const latParts = clean(latest).split('.').map(Number)
  
  for (let i = 0; i < Math.max(curParts.length, latParts.length); i++) {
    const curVal = curParts[i] || 0
    const latVal = latParts[i] || 0
    if (latVal > curVal) return true
    if (latVal < curVal) return false
  }
  return false
}

// 异步检测 GitHub 版本
async function checkForUpdates() {
  try {
    const response = await fetch('https://api.github.com/repos/chao-eng/AutoController/releases/latest')
    if (!response.ok) return
    const data = await response.json()
    if (!data || !data.tag_name) return

    if (isNewerVersion(appVersion, data.tag_name)) {
      const confirmed = await uiStore.showConfirm(
        '发现新版本 🎉',
        `发现新版本 ${data.tag_name} (当前版本 v${appVersion})。\n\n更新内容:\n${data.body || '暂无详细描述'}\n\n是否立即前往 GitHub 下载更新？`
      )
      if (confirmed) {
        await open(data.html_url || 'https://github.com/chao-eng/AutoController/releases/latest')
      }
    }
  } catch (e) {
    // 异常静默，不影响用户正常使用
    console.warn('自动版本检查失败(可能由于网络连接 GitHub 受限):', e)
  }
}


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

  // 启动 3 秒后执行新版本静默检测
  setTimeout(() => {
    checkForUpdates()
  }, 3000)
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
    <AppSidebar v-if="!isMapWindow" />
    <div class="app-main">
      <AppHeader v-if="!isMapWindow && !isForzaTelemetry" />
      <div class="app-content">
        <router-view />
      </div>
      <StatusBar
        v-if="!isMapWindow && !isForzaTelemetry"
        :deviceCount="deviceCount"
        :cpuUsage="cpuUsage"
        :memUsage="memUsage"
        :version="appVersion"
      />
    </div>
    <!-- Custom Dialogs/Toasts overlays -->
    <AppDialogs v-if="!isMapWindow" />
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
