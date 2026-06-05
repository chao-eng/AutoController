<script setup lang="ts">
import { useLogStore } from '../stores/log'
import { Trash2 } from '@lucide/vue'
import { ref } from 'vue'
import { Button } from '@/components/ui/button'
import PageShell from '@/components/layout/PageShell.vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import EmptyState from '@/components/layout/EmptyState.vue'

const store = useLogStore()
const levelFilter = ref<string>('')

function setLevelFilter(level: string) {
  store.levelFilter = level || null
  levelFilter.value = level
}

function getLevelColor(level: string): string {
  switch (level) {
    case 'Error': return 'text-red-500'
    case 'Warn': return 'text-yellow-500'
    case 'Info': return 'text-blue-500'
    case 'Debug': return 'text-muted-foreground'
    case 'Trace': return 'text-muted-foreground'
    default: return 'text-muted-foreground'
  }
}
</script>

<template>
  <PageShell :scroll="false">
    <PageHeader
      title="日志查看"
      description="查看最近运行日志，并按级别快速筛选问题线索。"
    >
      <template #actions>
        <div class="flex gap-0.5 bg-muted rounded-md p-0.5">
          <button
            v-for="level in ['', 'Error', 'Warn', 'Info', 'Debug']"
            :key="level"
            class="px-2.5 py-1 text-[11px] text-muted-foreground rounded-sm transition-colors hover:text-foreground"
            :class="{ 'bg-primary text-primary-foreground hover:text-primary-foreground': levelFilter === level }"
            @click="setLevelFilter(level)"
          >
            {{ level || '全部' }}
          </button>
        </div>
        <Button variant="ghost" size="icon" class="size-7" @click="store.clearEntries()" title="清空">
          <Trash2 :size="14" />
        </Button>
      </template>
    </PageHeader>

    <div class="flex-1 overflow-y-auto bg-card border border-border rounded-xl p-2 font-mono text-[11px]">
      <EmptyState
        v-if="store.filteredEntries().length === 0"
        title="暂无日志记录"
        description="运行脚本、调度任务或设备操作后，最新日志会显示在这里。"
        class="min-h-[240px] font-sans"
      />
      <div
        v-for="entry in store.filteredEntries().slice(-500).reverse()"
        :key="entry.id"
        class="flex gap-3 px-2 py-0.5 rounded-sm hover:bg-muted/50"
      >
        <span class="text-muted-foreground min-w-[72px] shrink-0">{{ new Date(entry.timestamp).toLocaleTimeString() }}</span>
        <span class="min-w-[40px] shrink-0 font-semibold" :class="getLevelColor(entry.level)">{{ entry.level }}</span>
        <span class="text-blue-500 min-w-[160px] shrink-0 truncate" :title="entry.module">{{ entry.module }}</span>
        <span class="text-muted-foreground flex-1 min-w-0 break-all whitespace-pre-wrap">{{ entry.message }}</span>
      </div>
    </div>
  </PageShell>
</template>
