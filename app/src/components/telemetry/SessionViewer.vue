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
import { BarChart3, Car, Clock, Database, Pencil, Play, RotateCcw, Star, Trophy, X } from '@lucide/vue'

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
const startedAtLabel = computed(() => new Date(props.session.startedAt).toLocaleString())
const durationLabel = computed(() => formatClock(packets.value.length / 60))

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
      class="session-dialog w-[92vw] max-w-[calc(100vw-1rem)] sm:max-w-[780px] lg:max-w-[980px] h-[92vh] max-h-[860px] p-0 flex flex-col gap-0 overflow-hidden rounded-xl"
    >
      <DialogDescription class="sr-only">会话详情视图，包含统计分析和数据回放功能</DialogDescription>
      <header class="session-header shrink-0">
        <div class="min-w-0 flex-1">
          <div class="session-kicker">Session Review</div>
          <template v-if="editing">
            <Input
              v-model="draftName"
              :placeholder="defaultLabel"
              class="mt-1 h-8 max-w-[520px] text-sm"
              @keydown="handleKeydown"
              @blur="commitName"
            />
          </template>
          <template v-else>
            <Button variant="ghost" class="session-title-button" @click="startEdit" title="点击重命名">
              <span class="truncate">{{ displayName }}</span>
              <Pencil class="size-3.5 text-muted-foreground" />
            </Button>
          </template>
          <div class="session-meta">
            <span>{{ startedAtLabel }}</span>
            <span>{{ packets.length }} samples</span>
            <span>{{ laps.length }} laps</span>
          </div>
        </div>
        <div class="flex items-center gap-1.5">
          <Button
            variant="ghost"
            size="icon-sm"
            class="session-icon-button"
            :class="bookmarked ? 'text-amber-500' : 'text-muted-foreground'"
            @click="toggleBookmark"
            :title="bookmarked ? '取消收藏' : '收藏'"
          >
            <Star :fill="bookmarked ? 'currentColor' : 'none'" />
          </Button>
          <DialogClose as-child>
            <Button variant="ghost" size="icon-sm" class="session-icon-button" title="关闭">
              <X />
            </Button>
          </DialogClose>
        </div>
      </header>

      <Tabs v-model="tab" class="flex flex-col flex-1 min-h-0">
        <TabsList class="session-tabs" variant="line">
          <TabsTrigger value="analysis" class="session-tab">
            <BarChart3 class="size-4" />
            <span>统计分析</span>
          </TabsTrigger>
          <TabsTrigger value="replay" class="session-tab">
            <RotateCcw class="size-4" />
            <span>数据回放</span>
          </TabsTrigger>
        </TabsList>

        <div class="flex-1 min-h-0">
          <TabsContent value="analysis" class="h-full m-0 p-0 data-active:flex flex-col">
            <ScrollArea class="h-full">
              <div v-if="loading" class="session-state">
                <Database class="size-5" />
                <p>正在加载 {{ session.packetCount }} 个数据包…</p>
              </div>
              <div v-else-if="packets.length === 0" class="session-state">
                <Database class="size-5" />
                <p>此会话没有记录遥测数据。</p>
              </div>
              <AnalysisTab v-else :packets="packets" :laps="laps" :use-mph="useMph ?? true" />
            </ScrollArea>
          </TabsContent>
          <TabsContent value="replay" class="h-full m-0 p-0 data-active:flex flex-col">
            <ScrollArea class="h-full">
              <div class="replay-workspace">
                <div class="replay-summary-grid">
                  <div class="metric-card metric-card-wide">
                    <Car class="metric-icon" />
                    <span class="metric-label">车辆</span>
                    <strong class="metric-value">{{ carName(session.carOrdinal) }}</strong>
                  </div>
                  <div class="metric-card">
                    <Clock class="metric-icon" />
                    <span class="metric-label">时长</span>
                    <strong class="metric-value">{{ durationLabel }}</strong>
                  </div>
                  <div class="metric-card">
                    <Database class="metric-icon" />
                    <span class="metric-label">数据样本</span>
                    <strong class="metric-value">{{ packets.length }}</strong>
                  </div>
                  <div class="metric-card metric-card-best">
                    <Trophy class="metric-icon" />
                    <span class="metric-label">最快圈速</span>
                    <strong class="metric-value">{{ session.bestLap ? formatClock(session.bestLap) : '—' }}</strong>
                  </div>
                </div>

                <div class="replay-content-grid">
                  <section v-if="laps.length" class="lap-panel">
                    <div class="panel-heading">
                      <span>单圈成绩</span>
                      <span>{{ laps.length }} laps</span>
                    </div>
                    <div class="lap-rows">
                      <div
                        v-for="lap in laps" :key="lap.lapNumber"
                        class="lap-row"
                        :class="{ best: lap.lapNumber === bestLapNumber }"
                      >
                        <span class="lap-index">第 {{ lap.lapNumber + 1 }} 圈</span>
                        <span class="lap-time">{{ formatClock(lap.lapTime) }}</span>
                        <span v-if="lap.lapNumber === bestLapNumber" class="lap-best">最快</span>
                      </div>
                    </div>
                  </section>

                  <section class="replay-action-panel">
                    <div class="panel-heading">
                      <span>仪表盘回放</span>
                      <span>{{ durationLabel }}</span>
                    </div>
                    <p>
                      回放将接管实时仪表盘，并保留底部时间轴、播放与倍速控制。
                    </p>
                    <Button class="replay-button" @click="beginReplay">
                      <Play class="size-4 fill-current" />
                      <span>在仪表盘上回放</span>
                    </Button>
                  </section>
                </div>

                <div v-if="!laps.length" class="lap-panel lap-panel-empty">
                  <div class="panel-heading">
                    <span>单圈成绩</span>
                    <span>0 laps</span>
                  </div>
                  <p>此会话尚未记录完整单圈，但仍可用仪表盘回放采样数据。</p>
                </div>
              </div>
            </ScrollArea>
          </TabsContent>
        </div>
      </Tabs>
    </DialogContent>
  </Dialog>
</template>

<style scoped>
.session-dialog {
  --race-accent: #3370ff;
  --race-accent-soft: color-mix(in srgb, var(--race-accent) 10%, transparent);
  --race-amber: #f59e0b;
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--bg-panel) 96%, white) 0%, var(--bg-panel) 42%),
    var(--bg-panel);
  border: 1px solid var(--bd-muted);
  box-shadow: 0 24px 80px rgba(15, 23, 42, 0.18);
}

.session-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 1rem;
  padding: 1rem 1.25rem 0.85rem;
  border-bottom: 1px solid var(--bd-dim);
  background:
    linear-gradient(90deg, var(--race-accent-soft), transparent 42%),
    color-mix(in srgb, var(--bg-panel) 94%, var(--bg-elevated));
}

.session-kicker {
  color: var(--tx-xdim);
  font-size: 0.66rem;
  font-weight: 800;
  letter-spacing: 0.12em;
  line-height: 1;
  text-transform: uppercase;
}

.session-title-button {
  display: inline-flex;
  min-width: 0;
  max-width: 100%;
  height: auto;
  margin-top: 0.35rem;
  margin-left: -0.4rem;
  padding: 0.18rem 0.4rem;
  gap: 0.45rem;
  color: var(--tx-hi);
  font-size: 1rem;
  font-weight: 750;
  line-height: 1.2;
}

.session-title-button :deep(svg) {
  opacity: 0;
  transition: opacity 120ms ease;
}

.session-title-button:hover :deep(svg),
.session-title-button:focus-visible :deep(svg) {
  opacity: 1;
}

.session-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem 0.75rem;
  margin-top: 0.35rem;
  color: var(--tx-dim);
  font-size: 0.74rem;
  font-variant-numeric: tabular-nums;
}

.session-meta span {
  position: relative;
}

.session-meta span + span::before {
  position: absolute;
  left: -0.45rem;
  color: var(--bd-strong);
  content: '/';
}

.session-icon-button {
  width: 1.85rem;
  height: 1.85rem;
  border: 1px solid transparent;
  color: var(--tx-dim);
}

.session-icon-button:hover {
  border-color: var(--bd-muted);
  background: var(--bg-elevated);
}

.session-tabs {
  width: 100%;
  height: auto;
  justify-content: flex-start;
  gap: 0;
  padding: 0 1.25rem;
  border-bottom: 1px solid var(--bd-dim);
  background: var(--bg-panel);
}

.session-tab {
  flex: 0 0 auto;
  height: 2.55rem;
  padding: 0 1rem;
  gap: 0.45rem;
  border-radius: 0;
  color: var(--tx-dim);
}

.session-tab::after {
  background: var(--race-accent);
}

.session-tab[data-active],
.session-tab[data-state='active'] {
  color: var(--tx-hi);
}

.session-state {
  display: flex;
  min-height: 320px;
  align-items: center;
  justify-content: center;
  gap: 0.55rem;
  color: var(--tx-dim);
  font-size: 0.88rem;
}

.replay-workspace {
  display: flex;
  width: min(100%, 880px);
  flex-direction: column;
  gap: 1rem;
  margin: 0 auto;
  padding: 1.25rem;
}

.replay-summary-grid {
  display: grid;
  grid-template-columns: minmax(220px, 1.35fr) repeat(3, minmax(120px, 1fr));
  gap: 0.75rem;
}

.metric-card {
  position: relative;
  display: flex;
  min-height: 7rem;
  min-width: 0;
  flex-direction: column;
  justify-content: flex-end;
  overflow: hidden;
  border: 1px solid var(--bd-dim);
  border-radius: 8px;
  background:
    linear-gradient(160deg, color-mix(in srgb, var(--bg-card) 80%, white), var(--bg-card)),
    var(--bg-card);
  padding: 0.8rem;
}

.metric-card::before {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background: linear-gradient(90deg, transparent 0, transparent 68%, color-mix(in srgb, var(--race-accent) 8%, transparent));
  content: '';
}

.metric-card-best {
  border-color: color-mix(in srgb, var(--race-amber) 42%, var(--bd-dim));
  background:
    linear-gradient(160deg, color-mix(in srgb, var(--race-amber) 10%, var(--bg-card)), var(--bg-card)),
    var(--bg-card);
}

.metric-icon {
  position: absolute;
  top: 0.78rem;
  right: 0.78rem;
  width: 1.1rem;
  height: 1.1rem;
  color: var(--tx-xdim);
}

.metric-card-best .metric-icon {
  color: var(--race-amber);
}

.metric-label {
  z-index: 1;
  color: var(--tx-dim);
  font-size: 0.68rem;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.metric-value {
  z-index: 1;
  display: block;
  margin-top: 0.35rem;
  color: var(--tx-hi);
  font-size: 1rem;
  font-variant-numeric: tabular-nums;
  font-weight: 760;
  line-height: 1.28;
  overflow-wrap: anywhere;
}

.replay-content-grid {
  display: grid;
  grid-template-columns: minmax(270px, 0.95fr) minmax(280px, 1.05fr);
  align-items: stretch;
  gap: 0.85rem;
}

.lap-panel,
.replay-action-panel {
  min-width: 0;
  border: 1px solid var(--bd-dim);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-card) 86%, transparent);
  padding: 0.9rem;
}

.panel-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  margin-bottom: 0.75rem;
  color: var(--tx-dim);
  font-size: 0.7rem;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.lap-rows {
  display: flex;
  flex-direction: column;
  gap: 0.38rem;
}

.lap-row {
  display: grid;
  min-height: 2.45rem;
  grid-template-columns: minmax(0, 1fr) max-content max-content;
  align-items: center;
  gap: 0.55rem;
  border: 1px solid var(--bd-subtle);
  border-radius: 6px;
  background: var(--bg-panel);
  padding: 0.5rem 0.65rem;
  color: var(--tx-mid);
}

.lap-row.best {
  border-color: color-mix(in srgb, var(--race-amber) 52%, var(--bd-dim));
  background: color-mix(in srgb, var(--race-amber) 9%, var(--bg-panel));
  color: var(--tx-hi);
}

.lap-index {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 650;
}

.lap-time {
  color: var(--tx-hi);
  font-variant-numeric: tabular-nums;
  font-weight: 760;
}

.lap-best {
  border-radius: 999px;
  background: color-mix(in srgb, var(--race-amber) 16%, transparent);
  color: color-mix(in srgb, var(--race-amber) 78%, black);
  font-size: 0.66rem;
  font-weight: 800;
  padding: 0.1rem 0.38rem;
}

.replay-action-panel {
  display: flex;
  flex-direction: column;
  min-height: 12rem;
}

.replay-action-panel p,
.lap-panel-empty p {
  color: var(--tx-dim);
  font-size: 0.88rem;
  line-height: 1.7;
}

.replay-button {
  width: fit-content;
  height: 2.45rem;
  margin-top: auto;
  padding-inline: 1rem;
  gap: 0.55rem;
  font-weight: 760;
}

@media (max-width: 760px) {
  .session-header {
    padding: 0.9rem 1rem 0.75rem;
  }

  .session-tabs {
    padding-inline: 1rem;
  }

  .session-tab {
    flex: 1 1 0;
    padding-inline: 0.55rem;
  }

  .replay-workspace {
    padding: 1rem;
  }

  .replay-summary-grid,
  .replay-content-grid {
    grid-template-columns: 1fr 1fr;
  }

  .metric-card-wide,
  .lap-panel,
  .replay-action-panel,
  .lap-panel-empty {
    grid-column: 1 / -1;
  }
}

@media (max-width: 520px) {
  .session-header {
    align-items: flex-start;
    gap: 0.75rem;
  }

  .session-title-button {
    font-size: 0.92rem;
  }

  .session-meta {
    font-size: 0.68rem;
  }

  .replay-summary-grid {
    grid-template-columns: 1fr;
  }

  .metric-card {
    min-height: 5.8rem;
  }

  .lap-row {
    grid-template-columns: minmax(0, 1fr) max-content;
  }

  .lap-best {
    grid-column: 1 / -1;
    width: fit-content;
  }
}
</style>
