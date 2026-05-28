import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { onUnmounted } from 'vue'

export function useTauriEvent<T = unknown>(event: string, handler: (payload: T) => void) {
  let unlisten: UnlistenFn | null = null

  listen<T>(event, (e) => {
    handler(e.payload)
  }).then((fn) => {
    unlisten = fn
  })

  onUnmounted(() => {
    unlisten?.()
  })

  return { unlisten }
}
