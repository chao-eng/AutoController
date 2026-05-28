<script setup lang="ts">
import { useLogStore } from '../stores/log'
import { Trash2 } from '@lucide/vue'
import { ref } from 'vue'

const store = useLogStore()
const levelFilter = ref<string>('')

function setLevelFilter(level: string) {
  store.levelFilter = level || null
  levelFilter.value = level
}

function getLevelColor(level: string): string {
  switch (level) {
    case 'Error': return 'var(--color-error)'
    case 'Warn': return 'var(--color-warning)'
    case 'Info': return 'var(--color-info)'
    case 'Debug': return 'var(--color-text-dim)'
    case 'Trace': return 'var(--color-text-dim)'
    default: return 'var(--color-text-muted)'
  }
}
</script>

<template>
  <div class="log-viewer">
    <div class="page-header">
      <h2>日志查看</h2>
      <div class="header-actions">
        <div class="filter-group">
          <button
            v-for="level in ['', 'Error', 'Warn', 'Info', 'Debug']"
            :key="level"
            class="filter-btn"
            :class="{ active: levelFilter === level }"
            @click="setLevelFilter(level)"
          >
            {{ level || '全部' }}
          </button>
        </div>
        <button class="icon-btn" @click="store.clearEntries()" title="清空">
          <Trash2 :size="14" />
        </button>
      </div>
    </div>

    <div class="log-table">
      <div v-if="store.filteredEntries().length === 0" class="empty-state">
        暂无日志记录
      </div>
      <div
        v-for="entry in store.filteredEntries().slice(-500).reverse()"
        :key="entry.id"
        class="log-row"
      >
        <span class="log-time">{{ new Date(entry.timestamp).toLocaleTimeString() }}</span>
        <span class="log-level" :style="{ color: getLevelColor(entry.level) }">{{ entry.level }}</span>
        <span class="log-module">{{ entry.module }}</span>
        <span class="log-message">{{ entry.message }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.log-viewer {
  padding: var(--space-lg);
  height: 100%;
  display: flex;
  flex-direction: column;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--space-md);
}

.page-header h2 {
  font-size: 18px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.filter-group {
  display: flex;
  gap: 2px;
  background: var(--color-surface);
  border-radius: var(--radius-md);
  padding: 2px;
}

.filter-btn {
  padding: 4px 10px;
  font-size: 11px;
  color: var(--color-text-dim);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.filter-btn:hover {
  color: var(--color-text);
}

.filter-btn.active {
  background: var(--color-cta);
  color: white;
}

.icon-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  color: var(--color-text-dim);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.icon-btn:hover {
  background: var(--color-surface-elevated);
  color: var(--color-text);
}

.log-table {
  flex: 1;
  overflow-y: auto;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-lg);
  padding: var(--space-sm);
  font-family: var(--font-heading);
  font-size: 11px;
}

.empty-state {
  text-align: center;
  color: var(--color-text-dim);
  padding: var(--space-2xl);
  font-size: 13px;
}

.log-row {
  display: flex;
  gap: var(--space-sm);
  padding: 2px var(--space-sm);
  border-radius: 2px;
}

.log-row:hover {
  background: var(--color-surface-elevated);
}

.log-time {
  color: var(--color-text-dim);
  min-width: 72px;
}

.log-level {
  min-width: 40px;
  font-weight: 600;
}

.log-module {
  color: var(--color-info);
  min-width: 80px;
}

.log-message {
  color: var(--color-text-muted);
  flex: 1;
}
</style>
