import { defineStore } from 'pinia'
import { ref } from 'vue'
import { ipc } from '../fh6-tel/lib/ipc'
import type { SessionRow, TelemetryPacket, AppSettings, SessionLap } from '../fh6-tel/lib/types'

export const useSessionsStore = defineStore('sessions', () => {
  const sessions = ref<SessionRow[]>([])
  const settings = ref<AppSettings | null>(null)

  async function loadSessions() {
    sessions.value = await ipc.getSessions()
  }

  async function loadSessionPackets(sessionId: number): Promise<TelemetryPacket[]> {
    return ipc.getSessionPackets(sessionId)
  }

  async function loadSessionLaps(sessionId: number): Promise<SessionLap[]> {
    return ipc.getSessionLaps(sessionId)
  }

  async function deleteSession(sessionId: number) {
    await ipc.deleteSession(sessionId)
    await loadSessions()
  }

  async function clearAllSessions() {
    await ipc.clearAllSessions()
    await loadSessions()
  }

  async function renameSession(sessionId: number, name: string | null) {
    await ipc.renameSession(sessionId, name)
    await loadSessions()
  }

  async function setSessionBookmark(sessionId: number, bookmarked: boolean) {
    await ipc.setSessionBookmark(sessionId, bookmarked)
    await loadSessions()
  }

  async function loadSettings(): Promise<AppSettings> {
    const s = await ipc.getSettings()
    settings.value = s
    return s
  }

  async function saveSettings(s: AppSettings) {
    await ipc.saveSettings(s)
    settings.value = s
  }

  return {
    sessions,
    settings,
    loadSessions,
    loadSessionPackets,
    loadSessionLaps,
    deleteSession,
    clearAllSessions,
    renameSession,
    setSessionBookmark,
    loadSettings,
    saveSettings,
  }
})