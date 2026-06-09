<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useUIStore } from '../../stores/ui'
import { AlertCircle, Info } from '@lucide/vue'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'

const uiStore = useUIStore()

const promptInput = ref('')
const inputElement = ref<HTMLInputElement | null>(null)

watch(() => uiStore.activeDialog, async (newVal) => {
  if (newVal && newVal.type === 'prompt') {
    promptInput.value = newVal.defaultValue || ''
    await nextTick()
    inputElement.value?.focus()
    inputElement.value?.select()
  }
})

function handleConfirm() {
  if (!uiStore.activeDialog) return
  if (uiStore.activeDialog.type === 'prompt') {
    uiStore.closeDialog(promptInput.value)
  } else {
    uiStore.closeDialog(true)
  }
}

function handleCancel() {
  uiStore.closeDialog()
  promptInput.value = ''
}

function renderDialogMessage(message: string): string {
  if (!message) return ''
  let safeText = message
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
  safeText = safeText.replace(/^#### (.*$)/gim, '<h4 style="margin:10px 0 4px 0;font-size:12.5px;font-weight:700;color:var(--foreground)">$1</h4>')
  safeText = safeText.replace(/^### (.*$)/gim, '<h3 style="margin:14px 0 6px 0;font-size:13.5px;font-weight:700;color:var(--foreground);border-left:3px solid var(--primary);padding-left:8px;">$1</h3>')
  safeText = safeText.replace(/^## (.*$)/gim, '<h2 style="margin:16px 0 8px 0;font-size:14.5px;font-weight:700;color:var(--foreground);">$1</h2>')
  safeText = safeText.replace(/^# (.*$)/gim, '<h1 style="margin:18px 0 10px 0;font-size:15.5px;font-weight:700;color:var(--foreground);">$1</h1>')
  safeText = safeText.replace(/\*\*(.*?)\*\*/g, '<strong style="font-weight:600;color:var(--foreground);">$1</strong>')
  safeText = safeText.replace(/^\s*[-*]\s+(.*$)/gim, '<div style="display:flex;align-items:flex-start;gap:6px;margin:6px 0 6px 12px;font-size:12px;line-height:1.5;color:var(--muted-foreground);"><span style="color:var(--primary);margin-top:2px;">•</span><span>$1</span></div>')
  safeText = safeText.replace(/\n/g, '<br />')
  return safeText
}


</script>

<template>
  <Dialog :open="!!uiStore.activeDialog" @update:open="(v) => { if (!v) handleCancel() }">
    <DialogContent class="flex max-h-[82vh] flex-col overflow-hidden sm:max-w-[520px]">
      <DialogHeader v-if="uiStore.activeDialog" class="shrink-0 pr-8">
        <div class="flex items-center gap-2">
          <span
            class="flex size-7 items-center justify-center rounded-full"
            :class="uiStore.activeDialog.type === 'alert' ? 'bg-destructive/10 text-destructive' : 'bg-primary/10 text-primary'"
          >
            <AlertCircle v-if="uiStore.activeDialog.type === 'alert'" :size="18" />
            <Info v-else :size="18" />
          </span>
          <DialogTitle>{{ uiStore.activeDialog.title }}</DialogTitle>
        </div>
      </DialogHeader>
      <div
        v-if="uiStore.activeDialog"
        class="-mx-4 min-h-0 overflow-y-auto px-6 pb-1 pr-5"
      >
        <DialogDescription
          class="text-sm leading-relaxed"
          v-html="renderDialogMessage(uiStore.activeDialog.message)"
        />
        <div v-if="uiStore.activeDialog.type === 'prompt'" class="mt-3">
          <Input
            ref="inputElement"
            v-model="promptInput"
            :placeholder="uiStore.activeDialog.placeholder || '请输入...'"
            @keyup.enter="handleConfirm"
            @keyup.esc="handleCancel"
          />
        </div>
      </div>
      <DialogFooter class="shrink-0">
        <Button v-if="uiStore.activeDialog?.type !== 'alert'" variant="outline" @click="handleCancel">
          取消
        </Button>
        <Button @click="handleConfirm">
          {{ uiStore.activeDialog?.type === 'confirm' ? '确认' : '确定' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
