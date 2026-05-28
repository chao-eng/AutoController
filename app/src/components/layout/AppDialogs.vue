<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useUIStore } from '../../stores/ui'
import { X, AlertCircle, AlertTriangle, CheckCircle2, Info, XCircle } from '@lucide/vue'

const uiStore = useUIStore()

// For prompt input
const promptInput = ref('')
const inputElement = ref<HTMLInputElement | null>(null)

// Auto-focus the input when prompt is displayed
watch(() => uiStore.activeDialog, async (newVal) => {
  if (newVal && newVal.type === 'prompt') {
    promptInput.value = newVal.defaultValue || ''
    await nextTick()
    if (inputElement.value) {
      inputElement.value.focus()
      inputElement.value.select()
    }
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
}
</script>

<template>
  <div class="ui-dialogs-container">
    <!-- Global Floating Toasts Stack -->
    <div class="toasts-stack">
      <TransitionGroup name="toast-fade">
        <div 
          v-for="toast in uiStore.toasts" 
          :key="toast.id" 
          class="toast-item"
          :class="`toast-${toast.type}`"
        >
          <div class="toast-icon">
            <CheckCircle2 v-if="toast.type === 'success'" :size="16" />
            <XCircle v-else-if="toast.type === 'error'" :size="16" />
            <AlertTriangle v-else-if="toast.type === 'warning'" :size="16" />
            <Info v-else :size="16" />
          </div>
          <div class="toast-message">{{ toast.message }}</div>
          <button class="toast-close" @click="uiStore.removeToast(toast.id)">
            <X :size="12" />
          </button>
        </div>
      </TransitionGroup>
    </div>

    <!-- Custom Modal Dialog Overlay -->
    <Transition name="modal-fade">
      <div v-if="uiStore.activeDialog" class="dialog-backdrop" @click.self="handleCancel">
        <div class="dialog-modal" :class="`dialog-modal-${uiStore.activeDialog.type}`">
          <div class="dialog-header">
            <div class="dialog-title-wrapper">
              <span class="dialog-icon-badge" :class="uiStore.activeDialog.type">
                <AlertCircle v-if="uiStore.activeDialog.type === 'alert'" :size="18" />
                <Info v-else-if="uiStore.activeDialog.type === 'confirm'" :size="18" />
                <AlertCircle v-else :size="18" />
              </span>
              <h3>{{ uiStore.activeDialog.title }}</h3>
            </div>
            <button class="dialog-close" @click="handleCancel">
              <X :size="16" />
            </button>
          </div>

          <div class="dialog-body">
            <p class="dialog-message">{{ uiStore.activeDialog.message }}</p>
            
            <!-- Input field for prompt dialog type -->
            <div v-if="uiStore.activeDialog.type === 'prompt'" class="prompt-input-wrapper">
              <input
                ref="inputElement"
                v-model="promptInput"
                class="dialog-input"
                :placeholder="uiStore.activeDialog.placeholder || '请输入...'"
                @keyup.enter="handleConfirm"
                @keyup.esc="handleCancel"
              />
            </div>
          </div>

          <div class="dialog-footer">
            <button 
              v-if="uiStore.activeDialog.type !== 'alert'" 
              class="btn-dialog-secondary" 
              @click="handleCancel"
            >
              取消
            </button>
            <button class="btn-dialog-primary" @click="handleConfirm">
              {{ uiStore.activeDialog.type === 'confirm' ? '确认' : '确定' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.ui-dialogs-container {
  position: relative;
  z-index: 99999;
}

/* Toast styling and animations */
.toasts-stack {
  position: fixed;
  top: 24px;
  right: 24px;
  display: flex;
  flex-direction: column;
  gap: var(--space-sm);
  z-index: 100000;
  max-width: 320px;
  pointer-events: none;
}

.toast-item {
  pointer-events: auto;
  display: flex;
  align-items: center;
  gap: var(--space-sm);
  padding: 10px 14px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  box-shadow: var(--shadow-lg);
  min-width: 240px;
}

.toast-message {
  font-size: 12px;
  color: var(--color-text);
  font-weight: 500;
  flex: 1;
  word-break: break-all;
}

.toast-close {
  color: var(--color-text-dim);
  cursor: pointer;
  opacity: 0.6;
  transition: opacity var(--transition-fast);
}

.toast-close:hover {
  opacity: 1;
}

.toast-icon {
  display: flex;
  align-items: center;
  justify-content: center;
}

/* Toast types theme coloring */
.toast-success {
  border-left: 3px solid var(--color-success);
}
.toast-success .toast-icon {
  color: var(--color-success);
}

.toast-error {
  border-left: 3px solid var(--color-error);
}
.toast-error .toast-icon {
  color: var(--color-error);
}

.toast-warning {
  border-left: 3px solid var(--color-warning);
}
.toast-warning .toast-icon {
  color: var(--color-warning);
}

.toast-info {
  border-left: 3px solid var(--color-cta);
}
.toast-info .toast-icon {
  color: var(--color-cta);
}

/* Toast transitions */
.toast-fade-enter-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.toast-fade-leave-active {
  transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}
.toast-fade-enter-from {
  opacity: 0;
  transform: translateX(30px) scale(0.9);
}
.toast-fade-leave-to {
  opacity: 0;
  transform: translateY(-10px) scale(0.95);
}

/* Dialog styling and animations */
.dialog-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 17, 21, 0.6);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 99999;
}

.dialog-modal {
  width: 420px;
  max-width: 90%;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.25);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: var(--space-md) var(--space-lg);
  border-bottom: 1px solid var(--color-border);
}

.dialog-title-wrapper {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.dialog-title-wrapper h3 {
  font-size: 14px;
  font-weight: 600;
  margin: 0;
  color: var(--color-text);
}

.dialog-icon-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 50%;
}

.dialog-icon-badge.alert {
  background: rgba(245, 74, 69, 0.1);
  color: var(--color-error);
}

.dialog-icon-badge.confirm {
  background: rgba(51, 112, 255, 0.1);
  color: var(--color-cta);
}

.dialog-icon-badge.prompt {
  background: rgba(51, 112, 255, 0.1);
  color: var(--color-cta);
}

.dialog-close {
  color: var(--color-text-dim);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.dialog-close:hover {
  background: var(--color-surface-elevated);
  color: var(--color-text);
}

.dialog-body {
  padding: var(--space-lg);
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.dialog-message {
  font-size: 13px;
  color: var(--color-text-muted);
  line-height: 1.5;
  margin: 0;
  white-space: pre-wrap;
}

.prompt-input-wrapper {
  margin-top: var(--space-xs);
}

.dialog-input {
  width: 100%;
  padding: 8px 12px;
  background: var(--color-surface-elevated);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text);
  font-size: 13px;
  font-family: inherit;
  outline: none;
  transition: border-color var(--transition-fast);
}

.dialog-input:focus {
  border-color: var(--color-cta);
  box-shadow: 0 0 0 2px rgba(51, 112, 255, 0.1);
}

.dialog-footer {
  padding: var(--space-md) var(--space-lg);
  background: var(--color-surface-elevated);
  border-top: 1px solid var(--color-border);
  display: flex;
  justify-content: flex-end;
  gap: var(--space-sm);
}

.btn-dialog-primary {
  padding: var(--space-sm) var(--space-lg);
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 600;
  background: var(--color-cta);
  color: white;
  cursor: pointer;
  transition: background var(--transition-fast);
}

.btn-dialog-primary:hover {
  background: var(--color-cta-hover);
}

.btn-dialog-secondary {
  padding: var(--space-sm) var(--space-lg);
  border-radius: var(--radius-md);
  font-size: 12px;
  font-weight: 500;
  background: transparent;
  color: var(--color-text-muted);
  border: 1px solid var(--color-border);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.btn-dialog-secondary:hover {
  border-color: var(--color-text);
  color: var(--color-text);
  background: var(--color-surface);
}

/* Modal transitions */
.modal-fade-enter-active {
  transition: opacity 0.25s ease;
}
.modal-fade-leave-active {
  transition: opacity 0.2s ease;
}
.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}

.modal-fade-enter-active .dialog-modal {
  animation: modalIn 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.modal-fade-leave-active .dialog-modal {
  animation: modalOut 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes modalIn {
  from { transform: scale(0.95) translateY(10px); opacity: 0; }
  to { transform: scale(1) translateY(0); opacity: 1; }
}

@keyframes modalOut {
  from { transform: scale(1) translateY(0); opacity: 1; }
  to { transform: scale(0.95) translateY(10px); opacity: 0; }
}
</style>
