<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSessionsStore } from '@/stores/sessions'
import { carName } from '@/fh6-tel/lib/car-name'
import type { SessionRow } from '@/fh6-tel/lib/types'

const emit = defineEmits<{
  close: []
  open: [session: SessionRow]
}>()

const sessionsStore = useSessionsStore()

onMounted(() => {
  sessionsStore.loadSessions()
})

function formatTime(seconds: number) {
  if (!seconds || seconds <= 0) return '—'
  const m = Math.floor(seconds / 60)
  const s = (seconds % 60).toFixed(3).padStart(6, '0')
  return `${m}:${s}`
}

function formatDate(ms: number) {
  return new Date(ms).toLocaleString()
}

async function handleDelete(session: SessionRow, e: MouseEvent) {
  e.stopPropagation()
  const label = session.name ?? formatDate(session.startedAt)
  if (!confirm(`确定要删除会话 "${label}" 吗？`)) return
  await sessionsStore.deleteSession(session.id)
}

async function toggleBookmark(session: SessionRow, e: MouseEvent) {
  e.stopPropagation()
  await sessionsStore.setSessionBookmark(session.id, !session.bookmarked)
}

async function handleClearAll() {
  const n = sessionsStore.sessions.length
  if (n === 0) return
  if (!confirm(`确定要删除所有 ${n} 个会话吗？此操作无法撤销。`)) return
  await sessionsStore.clearAllSessions()
}
</script>

<template>
  <div class="drawer">
    <div class="drawer-header">
      <h3 class="m-0 text-[var(--tx-hi)] text-base">历史会话</h3>
      <div class="flex items-center gap-1.5">
        <button
          class="bg-none border border-[var(--bd-muted)] text-[var(--tx-dim)] text-[0.72rem] px-1.5 py-0.5 rounded cursor-pointer disabled:opacity-40 disabled:cursor-default"
          :disabled="sessionsStore.sessions.length === 0"
          @click="handleClearAll"
        >清除全部</button>
        <button class="bg-none border-none text-[var(--tx-dim)] text-[1.1rem] cursor-pointer hover:text-[var(--tx-hi)]" @click="emit('close')">✕</button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto flex flex-col gap-1 p-2">
      <div v-if="sessionsStore.sessions.length === 0" class="text-[var(--tx-xdim)] text-[0.85rem] text-center py-8">
        暂无记录的会话数据。
      </div>
      <div v-else class="flex flex-col gap-[0.3rem]">
        <div
          v-for="session in sessionsStore.sessions" :key="session.id"
          class="flex items-center gap-2 px-3 py-2.5 rounded cursor-pointer border border-transparent bg-[var(--bg-elevated)] hover:border-[var(--ac)]"
          role="button"
          tabindex="0"
          @click="emit('open', session)"
          @keydown.enter="emit('open', session)"
        >
          <button
            class="bg-none border-none cursor-pointer text-[1.05rem] text-[var(--tx-dim)] leading-none shrink-0"
            :class="session.bookmarked ? 'text-amber-400' : ''"
            :title="session.bookmarked ? '取消收藏' : '收藏'"
            @click="(e: MouseEvent) => toggleBookmark(session, e)"
          >
            {{ session.bookmarked ? '★' : '☆' }}
          </button>
          <div class="flex flex-col gap-[0.1rem] flex-1 min-w-0">
            <span class="text-[0.85rem] font-semibold text-[var(--tx-mid)] overflow-hidden text-ellipsis whitespace-nowrap">
              {{ session.name ?? carName(session.carOrdinal) }}
            </span>
            <span class="text-[0.7rem] text-[var(--tx-dim)]">{{ formatDate(session.startedAt) }}</span>
            <span class="text-[0.75rem] font-bold text-purple-500">最快: {{ formatTime(session.bestLap ?? 0) }}</span>
          </div>
          <button
            class="bg-none border-none cursor-pointer text-[0.9rem] text-[var(--tx-dim)] shrink-0 hover:text-red-500"
            @click="(e: MouseEvent) => handleDelete(session, e)"
          >🗑</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.drawer {
  position: fixed; right: 0; top: 0; bottom: 0; width: 420px;
  background: var(--bg-panel); border-left: 1px solid var(--bd-dim);
  display: flex; flex-direction: column; z-index: 50;
  box-shadow: -4px 0 24px rgba(31,35,41,0.08);
}
.drawer-header {
  display: flex; justify-content: space-between; align-items: center;
  padding: 1rem; border-bottom: 1px solid var(--bd-dim);
}
</style>