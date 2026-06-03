<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useSessionsStore } from '@/stores/sessions'
import { useTelemetryStore } from '@/stores/telemetry'
import { carName } from '@/fh6-tel/lib/car-name'
import type { TelemetryPacket, SessionRow, SessionLap } from '@/fh6-tel/lib/types'
import AnalysisTab from './AnalysisTab.vue'
import { Dialog, DialogContent, DialogDescription, DialogClose } from '@/components/ui/dialog'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'

const props = defineProps<{
  session: SessionRow
  useMph?: boolean
}>()

const emit = defineEmits<{ close: [] }>()

const sessionsStore = useSessionsStore()
const telemetryStore = useTelemetryStore()

type Tab = 'analysis' | 'replay'
const tab = ref<Tab>('analysis')

const packets = ref<TelemetryPacket[]>([])
const laps = ref<SessionLap[]>([])
const loading = ref(true)

const bestLapNumber = computed(() =>
  laps.value.length
    ? laps.value.reduce((b, l) => (l.lapTime < b.lapTime ? l : b)).lapNumber
    : -1
)

const editing = ref(false)
const draftName = ref('')
const bookmarked = ref(props.session.bookmarked)

const defaultLabel = computed(() =>
  `${carName(props.session.carOrdinal)} — ${new Date(props.session.startedAt).toLocaleString()}`
)
const displayName = computed(() => props.session.name ?? defaultLabel.value)

onMounted(async () => {
  packets.value = await sessionsStore.loadSessionPackets(props.session.id)
  loading.value = false
  laps.value = await sessionsStore.loadSessionLaps(props.session.id)
})

function formatClock(seconds: number) {
  if (!isFinite(seconds) || seconds < 0) seconds = 0
  const m = Math.floor(seconds / 60)
  const s = (seconds % 60).toFixed(3).padStart(6, '0')
  return `${m}:${s}`
}

function startEdit() {
  draftName.value = props.session.name ?? ''
  editing.value = true
}

async function commitName() {
  editing.value = false
  const v = draftName.value.trim()
  await sessionsStore.renameSession(props.session.id, v.length ? v : null)
  props.session.name = v.length ? v : null
}

async function toggleBookmark() {
  bookmarked.value = !bookmarked.value
  props.session.bookmarked = bookmarked.value
  await sessionsStore.setSessionBookmark(props.session.id, bookmarked.value)
}

function beginReplay() {
  const label = displayName.value
  telemetryStore.startReplay(props.session.id, label, packets.value)
  emit('close')
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === 'Enter') commitName()
  if (e.key === 'Escape') editing.value = false
}
</script>

<template>
  <Dialog :open="true" @update:open="(v) => { if (!v) emit('close') }">
    <DialogContent
      :show-close-button="false"
      class="w-[92vw] max-w-[calc(100vw-1rem)] sm:max-w-[760px] lg:max-w-[900px] h-[92vh] max-h-[860px] p-0 flex flex-col gap-0 overflow-hidden rounded-xl"
    >
      <DialogDescription class="sr-only">会话详情视图，包含统计分析和数据回放功能</DialogDescription>
      <header class="flex items-center justify-between gap-4 px-4 py-3 border-b border-border shrink-0">
        <div class="flex items-center gap-2 min-w-0 flex-1">
          <template v-if="editing">
            <Input
              v-model="draftName"
              :placeholder="defaultLabel"
              class="flex-1 h-8 text-sm"
              @keydown="handleKeydown"
              @blur="commitName"
            />
          </template>
          <template v-else>
            <Button variant="ghost" class="min-w-0 text-sm font-semibold px-1.5 -ml-1.5 h-auto max-w-full" @click="startEdit" title="点击重命名">
              <span class="truncate">{{ displayName }}</span>
              <span class="text-muted-foreground text-xs ml-1.5 shrink-0">✎</span>
            </Button>
          </template>
          <Button
            variant="ghost"
            size="icon-sm"
            class="size-7 shrink-0"
            :class="bookmarked ? 'text-amber-400' : 'text-muted-foreground'"
            @click="toggleBookmark"
            :title="bookmarked ? '取消收藏' : '收藏'"
          >
            {{ bookmarked ? '★' : '☆' }}
          </Button>
        </div>
        <DialogClose as-child>
          <Button variant="ghost" size="icon-sm" class="size-7 shrink-0">✕</Button>
        </DialogClose>
      </header>

      <Tabs v-model="tab" class="flex flex-col flex-1 min-h-0">
        <TabsList class="px-4 pt-1 pb-0 border-b border-border rounded-none bg-transparent h-auto justify-start gap-0">
          <TabsTrigger value="analysis" class="px-3 py-1.5 text-sm rounded-none data-active:bg-transparent data-active:text-foreground data-active:after:opacity-100 after:absolute after:inset-x-0 after:bottom-0 after:h-0.5 after:bg-foreground after:opacity-0 after:transition-opacity">统计分析</TabsTrigger>
          <TabsTrigger value="replay" class="px-3 py-1.5 text-sm rounded-none data-active:bg-transparent data-active:text-foreground data-active:after:opacity-100 after:absolute after:inset-x-0 after:bottom-0 after:h-0.5 after:bg-foreground after:opacity-0 after:transition-opacity">数据回放</TabsTrigger>
        </TabsList>

        <div class="flex-1 min-h-0">
          <TabsContent value="analysis" class="h-full m-0 p-0 data-active:flex flex-col">
            <ScrollArea class="h-full">
              <p v-if="loading" class="text-muted-foreground text-center py-12">正在加载 {{ session.packetCount }} 个数据包…</p>
              <p v-else-if="packets.length === 0" class="text-muted-foreground text-center py-12">此会话没有记录遥测数据。</p>
              <AnalysisTab v-else :packets="packets" :laps="laps" :use-mph="useMph ?? true" />
            </ScrollArea>
          </TabsContent>
          <TabsContent value="replay" class="h-full m-0 p-0 data-active:flex flex-col">
            <ScrollArea class="h-full">
              <div class="flex flex-col items-center gap-6 py-8 px-4">
                <div class="grid grid-cols-4 gap-4 w-full max-w-[560px]">
                  <div class="flex flex-col items-center bg-card border border-border rounded-lg py-3 px-2">
                    <span class="text-muted-foreground text-[0.8rem] uppercase tracking-wider">车辆</span>
                    <strong class="text-foreground text-sm mt-1">{{ carName(session.carOrdinal) }}</strong>
                  </div>
                  <div class="flex flex-col items-center bg-card border border-border rounded-lg py-3 px-2">
                    <span class="text-muted-foreground text-[0.8rem] uppercase tracking-wider">时长</span>
                    <strong class="text-foreground text-sm mt-1">{{ formatClock(packets.length / 60) }}</strong>
                  </div>
                  <div class="flex flex-col items-center bg-card border border-border rounded-lg py-3 px-2">
                    <span class="text-muted-foreground text-[0.8rem] uppercase tracking-wider">数据样本</span>
                    <strong class="text-foreground text-sm mt-1">{{ packets.length }}</strong>
                  </div>
                  <div class="flex flex-col items-center bg-card border border-border rounded-lg py-3 px-2">
                    <span class="text-muted-foreground text-[0.8rem] uppercase tracking-wider">最快圈速</span>
                    <strong class="text-foreground text-sm mt-1">{{ session.bestLap ? formatClock(session.bestLap) : '—' }}</strong>
                  </div>
                </div>

                <div v-if="laps.length" class="w-full max-w-[360px] flex flex-col gap-1">
                  <div class="text-muted-foreground text-[0.8rem] uppercase tracking-wider mb-1">单圈成绩</div>
                  <div
                    v-for="lap in laps" :key="lap.lapNumber"
                    class="flex justify-between items-center px-2.5 py-1.5 rounded-md bg-card border"
                    :class="lap.lapNumber === bestLapNumber ? 'border-purple-500 text-purple-500 font-bold' : 'border-border text-foreground'"
                  >
                    <span>第 {{ lap.lapNumber + 1 }} 圈</span>
                    <span class="tabular-nums">{{ formatClock(lap.lapTime) }}</span>
                  </div>
                </div>

                <p class="text-muted-foreground text-[0.85rem] text-center max-w-[420px]">
                  通过实时仪表盘回放此会话，您可以在时间轴上拖动、播放和调节播放速度。
                </p>
                <Button class="px-6 py-2 text-sm font-semibold" @click="beginReplay">▶ 在仪表盘上回放</Button>
              </div>
            </ScrollArea>
          </TabsContent>
        </div>
      </Tabs>
    </DialogContent>
  </Dialog>
</template>
