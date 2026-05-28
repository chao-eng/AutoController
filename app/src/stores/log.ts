import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface LogEntry {
  id: string
  level: 'Error' | 'Warn' | 'Info' | 'Debug' | 'Trace'
  message: string
  module: string
  timestamp: string
}

export const useLogStore = defineStore('log', () => {
  const entries = ref<LogEntry[]>([])
  const filter = ref<string | null>(null)
  const levelFilter = ref<string | null>(null)
  let unlisten: UnlistenFn | null = null

  function addEntry(entry: LogEntry) {
    entries.value.push(entry)
    if (entries.value.length > 10000) {
      entries.value = entries.value.slice(-5000)
    }
  }

  async function startListening() {
    if (unlisten) return
    unlisten = await listen<LogEntry>('log-entry', (event) => {
      addEntry(event.payload)
    })
  }

  function stopListening() {
    unlisten?.()
    unlisten = null
  }

  function clearEntries() {
    entries.value = []
  }

  const filteredEntries = () => {
    let result = entries.value
    if (levelFilter.value) {
      result = result.filter((e) => e.level === levelFilter.value)
    }
    if (filter.value) {
      const keyword = filter.value.toLowerCase()
      result = result.filter(
        (e) =>
          e.message.toLowerCase().includes(keyword) ||
          e.module.toLowerCase().includes(keyword)
      )
    }
    return result
  }

  return {
    entries,
    filter,
    levelFilter,
    addEntry,
    clearEntries,
    filteredEntries,
    startListening,
    stopListening,
  }
})
