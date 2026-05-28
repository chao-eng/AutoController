<script setup lang="ts">
import { HelpCircle } from '@lucide/vue'
import { ref } from 'vue'
import HelpModal from '../HelpModal.vue'

const showHelp = ref(false)
</script>

<template>
  <header class="app-header" data-tauri-drag-region>
    <div class="header-title" data-tauri-drag-region>
      <span class="title-text">手柄自动化控制台</span>
    </div>
    <div class="header-actions">
      <button class="header-btn" @click="showHelp = true" title="使用说明">
        <HelpCircle :size="14" />
      </button>
    </div>
    <HelpModal v-if="showHelp" @close="showHelp = false" />
  </header>
</template>

<style scoped>
.app-header {
  height: var(--header-height);
  background: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--space-md);
  flex-shrink: 0;
}

.header-title {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.title-text {
  font-family: var(--font-heading);
  font-size: 13px;
  font-weight: 600;
  color: var(--color-text-muted);
  letter-spacing: 0.5px;
}

.header-actions {
  display: flex;
  gap: 2px;
}

.header-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-dim);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.header-btn:hover {
  background: var(--color-surface-elevated);
  color: var(--color-text);
}
</style>
