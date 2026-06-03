<script setup lang="ts">
import { onMounted } from 'vue'
import { useSessionsStore } from '@/stores/sessions'
import { carName } from '@/fh6-tel/lib/car-name'
import type { SessionRow } from '@/fh6-tel/lib/types'
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetDescription, SheetClose } from '@/components/ui/sheet'
import { Button } from '@/components/ui/button'

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
  <Sheet :open="true" @update:open="(v) => { if (!v) emit('close') }">
    <SheetContent side="right" class="w-[420px] sm:max-w-[420px] flex flex-col p-0">
      <SheetHeader class="flex-row items-center justify-between px-4 py-3 border-b border-border">
        <SheetTitle class="text-base">历史会话</SheetTitle>
        <div class="flex items-center gap-1.5">
          <Button
            variant="outline"
            size="sm"
            class="h-7 text-[0.72rem] px-1.5 py-0.5"
            :disabled="sessionsStore.sessions.length === 0"
            @click="handleClearAll"
          >清除全部</Button>
          <SheetClose as-child>
            <Button variant="ghost" size="icon-sm" class="size-7">✕</Button>
          </SheetClose>
        </div>
      </SheetHeader>

      <SheetDescription class="sr-only">历史会话列表，可查看、收藏和删除已记录的遥测会话</SheetDescription>

      <div class="flex-1 overflow-y-auto flex flex-col gap-1 p-2">
        <div v-if="sessionsStore.sessions.length === 0" class="text-muted-foreground text-[0.85rem] text-center py-8">
          暂无记录的会话数据。
        </div>
        <div v-else class="flex flex-col gap-[0.3rem]">
          <div
            v-for="session in sessionsStore.sessions" :key="session.id"
            class="flex items-center gap-2 px-3 py-2.5 rounded cursor-pointer border border-transparent bg-card hover:border-primary"
            role="button"
            tabindex="0"
            @click="emit('open', session)"
            @keydown.enter="emit('open', session)"
          >
            <button
              class="bg-none border-none cursor-pointer text-[1.05rem] text-muted-foreground leading-none shrink-0"
              :class="session.bookmarked ? 'text-amber-400' : ''"
              :title="session.bookmarked ? '取消收藏' : '收藏'"
              @click="(e: MouseEvent) => toggleBookmark(session, e)"
            >
              {{ session.bookmarked ? '★' : '☆' }}
            </button>
            <div class="flex flex-col gap-[0.1rem] flex-1 min-w-0">
              <span class="text-[0.85rem] font-semibold text-foreground overflow-hidden text-ellipsis whitespace-nowrap">
                {{ session.name ?? carName(session.carOrdinal) }}
              </span>
              <span class="text-[0.8rem] text-muted-foreground">{{ formatDate(session.startedAt) }}</span>
              <span class="text-[0.85rem] font-bold text-purple-500">最快: {{ formatTime(session.bestLap ?? 0) }}</span>
            </div>
            <button
              class="bg-none border-none cursor-pointer text-[0.9rem] text-muted-foreground shrink-0 hover:text-destructive"
              @click="(e: MouseEvent) => handleDelete(session, e)"
            >🗑</button>
          </div>
        </div>
      </div>
    </SheetContent>
  </Sheet>
</template>