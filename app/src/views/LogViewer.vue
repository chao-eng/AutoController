<script setup lang="ts">
import { useLogStore } from '../stores/log'
import { Trash2 } from '@lucide/vue'
import { ref } from 'vue'
import { Button } from '@/components/ui/button'

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
  <div class="h-full overflow-y-auto p-6 flex flex-col">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-lg font-semibold">日志查看</h2>
      <div class="flex items-center gap-2">
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
      </div>
    </div>

    <div class="flex-1 overflow-y-auto bg-card border border-border rounded-xl p-2 font-mono text-[11px]">
      <div v-if="store.filteredEntries().length === 0" class="text-center text-muted-foreground py-12 text-sm">
        暂无日志记录
      </div>
      <div
        v-for="entry in store.filteredEntries().slice(-500).reverse()"
        :key="entry.id"
        class="flex gap-2 px-2 py-0.5 rounded-sm hover:bg-muted/50"
      >
        <span class="text-muted-foreground min-w-[72px]">{{ new Date(entry.timestamp).toLocaleTimeString() }}</span>
        <span class="min-w-[40px] font-semibold" :class="getLevelColor(entry.level)">{{ entry.level }}</span>
        <span class="text-blue-500 min-w-[80px]">{{ entry.module }}</span>
        <span class="text-muted-foreground flex-1">{{ entry.message }}</span>
      </div>
    </div>
  </div>
</template>