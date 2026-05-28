<script setup lang="ts">
import { HelpCircle } from '@lucide/vue'
import { ref } from 'vue'
import HelpModal from '../HelpModal.vue'
import { openUrl } from '@tauri-apps/plugin-opener'

const showHelp = ref(false)

async function openGithub() {
  try {
    await openUrl('https://github.com/chao-eng/AutoController')
  } catch (err) {
    console.error('无法在系统浏览器中打开网页:', err)
  }
}
</script>

<template>
  <header class="app-header" data-tauri-drag-region>
    <div class="header-title" data-tauri-drag-region>
      <span class="title-text">手柄自动化控制台</span>
    </div>
    <div class="header-actions">
      <button class="header-btn" @click="openGithub" title="项目 GitHub 源码">
        <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 0 0-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0 0 20 4.77 5.07 5.07 0 0 0 19.91 1S18.73.65 16 2.48a13.38 13.38 0 0 0-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 0 0 5 4.77a5.44 5.44 0 0 0-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 0 0 9 18.13V22"></path>
        </svg>
      </button>
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
