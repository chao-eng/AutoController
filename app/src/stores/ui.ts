import { defineStore } from 'pinia'
import { ref } from 'vue'
import { toast } from 'vue-sonner'

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
  const activeDialog = ref<DialogOptions | null>(null)

  function showToast(message: string, type: 'success' | 'error' | 'info' | 'warning' = 'info', duration = 3000) {
    const toastFn = type === 'success' ? toast.success : type === 'error' ? toast.error : type === 'warning' ? toast.warning : toast.info
    toastFn(message, { duration })
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
    activeDialog,
    showToast,
    showAlert,
    showConfirm,
    showPrompt,
    closeDialog
  }
})