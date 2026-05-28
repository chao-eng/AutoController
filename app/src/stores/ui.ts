import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface ToastMessage {
  id: string
  message: string
  type: 'success' | 'error' | 'info' | 'warning'
  duration: number
}

export interface DialogOptions {
  title: string
  message: string
  type: 'alert' | 'confirm' | 'prompt'
  defaultValue?: string
  placeholder?: string
  resolve: (value: any) => void
  reject: () => void
}

export const useUIStore = defineStore('ui', () => {
  const toasts = ref<ToastMessage[]>([])
  const activeDialog = ref<DialogOptions | null>(null)

  function showToast(message: string, type: ToastMessage['type'] = 'info', duration = 3000) {
    const id = Math.random().toString(36).substring(2, 9)
    const toast: ToastMessage = { id, message, type, duration }
    toasts.value.push(toast)
    setTimeout(() => {
      removeToast(id)
    }, duration)
  }

  function removeToast(id: string) {
    toasts.value = toasts.value.filter(t => t.id !== id)
  }

  function showAlert(title: string, message: string): Promise<void> {
    return new Promise((resolve) => {
      activeDialog.value = {
        title,
        message,
        type: 'alert',
        resolve,
        reject: () => resolve()
      }
    })
  }

  function showConfirm(title: string, message: string): Promise<boolean> {
    return new Promise((resolve) => {
      activeDialog.value = {
        title,
        message,
        type: 'confirm',
        resolve: () => resolve(true),
        reject: () => resolve(false)
      }
    })
  }

  function showPrompt(title: string, message: string, defaultValue = '', placeholder = ''): Promise<string | null> {
    return new Promise((resolve) => {
      activeDialog.value = {
        title,
        message,
        type: 'prompt',
        defaultValue,
        placeholder,
        resolve: (val) => resolve(val),
        reject: () => resolve(null)
      }
    })
  }

  function closeDialog(confirmValue?: any) {
    if (!activeDialog.value) return
    if (confirmValue !== undefined) {
      activeDialog.value.resolve(confirmValue)
    } else {
      activeDialog.value.reject()
    }
    activeDialog.value = null
  }

  return {
    toasts,
    activeDialog,
    showToast,
    removeToast,
    showAlert,
    showConfirm,
    showPrompt,
    closeDialog
  }
})
