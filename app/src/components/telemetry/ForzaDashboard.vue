<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useTelemetryStore } from '@/stores/telemetry'
import { useSessionsStore } from '@/stores/sessions'
import { isDesktop } from '@/fh6-tel/lib/ipc'
import TopBar from './TopBar.vue'
import CompassBar from './CompassBar.vue'
import CenterPanel from './CenterPanel.vue'
import TireWidget from './TireWidget.vue'
import FloatingPanel from './FloatingPanel.vue'
import LapBar from './LapBar.vue'
import SessionDrawer from './SessionDrawer.vue'
import SessionViewer from './SessionViewer.vue'
import ReplayBar from './ReplayBar.vue'
import SettingsDialog from './SettingsDialog.vue'
import type { SessionRow } from '@/fh6-tel/lib/types'

const telemetryStore = useTelemetryStore()
const sessionsStore = useSessionsStore()

const showSessions = ref(false)
const showSettings = ref(false)
const viewerSession = ref<SessionRow | null>(null)

interface Toast { id: number; message: string }
const toasts = ref<Toast[]>([])
let nextToastId = 0

function addToast(message: string) {
  const id = nextToastId++
  toasts.value = [...toasts.value, { id, message }]
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }, 4000)
}

interface PendingUpdate {
  version: string
  install: () => Promise<void>
}
const pendingUpdate = ref<PendingUpdate | null>(null)
const updateInstalling = ref(false)

onMounted(async () => {
  await sessionsStore.loadSettings()
  await telemetryStore.startTelemetryListener({
    onError: (m: string) => addToast(m),
    onBindFailed: (m: string) => addToast(m),
  })
  if (isDesktop) {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const info = await invoke<{ version: string; is_deb: boolean } | null>('check_for_update')
      if (info) {
        pendingUpdate.value = {
          version: info.version,
          install: async () => {
            updateInstalling.value = true
            await invoke('install_update', { isDeb: info.is_deb })
          },
        }
      }
    } catch {
      // Offline or update endpoint unreachable — ignore
    }
  }
})

// Replaying takes over the live dashboard — get the overlays out of the way.
watch(() => telemetryStore.replay.active, (active) => {
  if (active) {
    showSessions.value = false
    viewerSession.value = null
  }
})

const settings = computed(() => sessionsStore.settings)

async function handleToggleTires() {
  const s = settings.value
  if (s) await sessionsStore.saveSettings({ ...s, tiresVisible: !s.tiresVisible })
}
</script>

<template>
  <div v-if="pendingUpdate" class="update-bar">
    <span>Update v{{ pendingUpdate.version }} available</span>
    <button class="update-install" :disabled="updateInstalling" @click="pendingUpdate?.install()">
      {{ updateInstalling ? 'Installing…' : 'Install & restart' }}
    </button>
    <button class="update-dismiss" @click="pendingUpdate = null">✕</button>
  </div>

  <div class="dashboard">
    <TopBar
      :use-mph="settings?.useMph ?? true"
      @on-settings="showSettings = true"
      @on-sessions="showSessions = !showSessions"
      :tires-visible="settings?.tiresVisible ?? true"
      @on-toggle-tires="handleToggleTires"
    />
    <CompassBar />

    <div class="main">
      <div class="center-area">
        <CenterPanel :use-mph="settings?.useMph ?? true" />
      </div>
    </div>

    <FloatingPanel
      v-if="settings?.tiresVisible ?? true"
      id="fh6-tires"
      title="轮胎数据"
      :default-width="200"
      :default-top="64"
      @on-close="async () => { if (settings) await sessionsStore.saveSettings({ ...settings, tiresVisible: false }) }"
    >
      <TireWidget
        :tire-temp-cold="settings?.tireTempCold ?? 60"
        :tire-temp-optimal="settings?.tireTempOptimal ?? 85"
        :tire-temp-hot="settings?.tireTempHot ?? 110"
      />
    </FloatingPanel>

    <div class="lap-bar">
      <LapBar />
    </div>
  </div>

  <SessionDrawer
    v-if="showSessions"
    @on-close="showSessions = false"
    @on-open="(session: SessionRow) => viewerSession = session"
  />

  <SessionViewer
    v-if="viewerSession"
    :session="viewerSession"
    :use-mph="settings?.useMph ?? true"
    @close="viewerSession = null"
  />

  <ReplayBar />

  <div v-if="toasts.length > 0" class="toast-stack">
    <div v-for="toast in toasts" :key="toast.id" class="toast">{{ toast.message }}</div>
  </div>

  <SettingsDialog v-if="showSettings" @close="showSettings = false" />
</template>

<style scoped>
.dashboard {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}
.main {
  flex: 1;
  min-height: 0;
  overflow: hidden;
}
.center-area {
  background: var(--bg-body);
  overflow: hidden;
  width: 100%;
  height: 100%;
}
.lap-bar {
  height: clamp(2.5rem, 5.5vh, 4rem);
  flex-shrink: 0;
}
.update-bar {
  position: fixed; top: 0; left: 0; right: 0; z-index: 300;
  display: flex; align-items: center; gap: 0.75rem;
  padding: 0.35rem 1rem;
  background: #3370FF; border-bottom: none;
  font-size: 0.78rem; color: #fff;
}
.update-bar span { flex: 1; }
.update-install {
  background: rgba(255,255,255,0.2); color: #fff;
  border: 1px solid rgba(255,255,255,0.3); border-radius: 4px;
  padding: 0.2rem 0.65rem; font-size: 0.75rem; cursor: pointer;
}
.update-install:disabled { opacity: 0.6; cursor: default; }
.update-dismiss {
  background: none; border: none; color: rgba(255,255,255,0.7);
  font-size: 0.85rem; cursor: pointer; padding: 0 0.25rem;
}
.update-dismiss:hover { color: #fff; }
.toast-stack {
  position: fixed; bottom: 4rem; left: 50%; transform: translateX(-50%);
  display: flex; flex-direction: column; gap: 0.5rem; z-index: 200;
  pointer-events: none;
}
.toast {
  background: #FFFFFF; border: 1px solid #F54A45; border-radius: 6px;
  box-shadow: 0 4px 12px rgba(31,35,41,0.1);
  color: #F54A45; font-size: 0.8rem; padding: 0.5rem 1rem;
  max-width: 420px; text-align: center;
}
</style>