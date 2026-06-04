<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useTelemetryStore } from '@/stores/telemetry'
import { useSessionsStore } from '@/stores/sessions'
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

onMounted(async () => {
  await sessionsStore.loadSettings()
  await telemetryStore.startTelemetryListener({
    onError: (m: string) => addToast(m),
    onBindFailed: (m: string) => addToast(m),
  })
})

onUnmounted(() => {
  telemetryStore.stopTelemetryListener()
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
  <div class="flex h-full w-full flex-col">
    <TopBar
      :use-mph="settings?.useMph ?? true"
      :on-settings="() => showSettings = true"
      :on-sessions="() => showSessions = !showSessions"
      :tires-visible="settings?.tiresVisible ?? true"
      :on-toggle-tires="handleToggleTires"
    />
    <CompassBar />

    <div class="min-h-0 flex-1 overflow-hidden">
      <div class="h-full w-full overflow-hidden" :style="{ background: 'var(--bg-body)' }">
        <CenterPanel
          :use-mph="settings?.useMph ?? true"
          :max-speed="settings?.dashboardMaxSpeed ?? 180"
        />
      </div>
    </div>

    <FloatingPanel
      v-if="settings?.tiresVisible ?? true"
      id="fh6-tires"
      title="轮胎数据"
      :default-width="200"
      :default-top="64"
      @close="async () => { if (settings) await sessionsStore.saveSettings({ ...settings, tiresVisible: false }) }"
    >
      <TireWidget
        :tire-temp-cold="settings?.tireTempCold ?? 60"
        :tire-temp-optimal="settings?.tireTempOptimal ?? 85"
        :tire-temp-hot="settings?.tireTempHot ?? 110"
      />
    </FloatingPanel>

    <div class="h-[clamp(2.5rem,5.5vh,4rem)] shrink-0">
      <LapBar />
    </div>
  </div>

  <SessionDrawer
    v-if="showSessions"
    @close="showSessions = false"
    @open="(session: SessionRow) => viewerSession = session"
  />

  <SessionViewer
    v-if="viewerSession"
    :session="viewerSession"
    :use-mph="settings?.useMph ?? true"
    @close="viewerSession = null"
  />

  <ReplayBar />

  <div v-if="toasts.length > 0" class="fixed bottom-16 left-1/2 z-200 flex -translate-x-1/2 flex-col gap-2 pointer-events-none">
    <div v-for="toast in toasts" :key="toast.id" class="max-w-[420px] rounded-md border border-[#F54A45] bg-white px-4 py-2 text-center text-xs text-[#F54A45] shadow-lg">
      {{ toast.message }}
    </div>
  </div>

  <SettingsDialog v-if="showSettings" @close="showSettings = false" />
</template>
