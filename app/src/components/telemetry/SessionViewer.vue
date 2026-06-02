<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useSessionsStore } from '@/stores/sessions'
import { useTelemetryStore } from '@/stores/telemetry'
import { carName } from '@/fh6-tel/lib/car-name'
import type { TelemetryPacket, SessionRow, SessionLap } from '@/fh6-tel/lib/types'
import AnalysisTab from './AnalysisTab.vue'

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
  <div class="overlay" role="dialog" aria-modal="true">
    <div class="viewer">
      <header>
        <div class="title-area">
          <template v-if="editing">
            <input
              class="name-input"
              v-model="draftName"
              :placeholder="defaultLabel"
              @keydown="handleKeydown"
              @blur="commitName"
            />
          </template>
          <template v-else>
            <button class="name-display" @click="startEdit" title="点击重命名">
              {{ displayName }}
              <span class="edit-hint">✎</span>
            </button>
          </template>
          <button
            class="star"
            :class="{ on: bookmarked }"
            @click="toggleBookmark"
            :title="bookmarked ? '取消收藏' : '收藏'"
          >
            {{ bookmarked ? '★' : '☆' }}
          </button>
        </div>
        <button class="close" @click="emit('close')">✕</button>
      </header>

      <div class="tabs">
        <button :class="{ active: tab === 'analysis' }" @click="tab = 'analysis'">统计分析</button>
        <button :class="{ active: tab === 'replay' }" @click="tab = 'replay'">数据回放</button>
      </div>

      <div class="content">
        <p v-if="loading" class="status">正在加载 {{ session.packetCount }} 个数据包…</p>
        <p v-else-if="packets.length === 0" class="status">此会话没有记录遥测数据。</p>
        <AnalysisTab v-else-if="tab === 'analysis'" :packets="packets" :laps="laps" :use-mph="useMph ?? true" />
        <div v-else class="replay-panel">
          <div class="meta">
            <div><span>车辆</span><strong>{{ carName(session.carOrdinal) }}</strong></div>
            <div><span>时长</span><strong>{{ formatClock(packets.length / 60) }}</strong></div>
            <div><span>数据样本</span><strong>{{ packets.length }}</strong></div>
            <div><span>最快圈速</span><strong>{{ session.bestLap ? formatClock(session.bestLap) : '—' }}</strong></div>
          </div>

          <div v-if="laps.length" class="laps">
            <div class="laps-title">单圈成绩</div>
            <div
              v-for="lap in laps" :key="lap.lapNumber"
              class="lap-row"
              :class="{ best: lap.lapNumber === bestLapNumber }"
            >
              <span>第 {{ lap.lapNumber + 1 }} 圈</span>
              <span class="lap-time">{{ formatClock(lap.lapTime) }}</span>
            </div>
          </div>

          <p class="replay-help">
            通过实时仪表盘回放此会话，您可以在时间轴上拖动、播放和调节播放速度。
          </p>
          <button class="replay-go" @click="beginReplay">▶ 在仪表盘上回放</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed; inset: 0; background: rgba(31,35,41,0.45);
  display: flex; align-items: center; justify-content: center; z-index: 120;
}
.viewer {
  width: min(900px, 94vw); height: min(800px, 92vh);
  background: var(--bg-panel); border: 1px solid var(--bd-subtle);
  border-radius: 10px; display: flex; flex-direction: column;
  box-shadow: 0 12px 48px rgba(31,35,41,0.12);
}
header {
  display: flex; align-items: center; justify-content: space-between;
  gap: 1rem; padding: 0.9rem 1.1rem; border-bottom: 1px solid var(--bd-dim);
}
.title-area {
  display: flex; align-items: center; gap: 0.6rem; min-width: 0; flex: 1;
}
.name-display {
  background: none; border: none; color: var(--tx-hi);
  font-size: 1rem; font-weight: 600; cursor: pointer; text-align: left;
  padding: 0.2rem 0.3rem; border-radius: 4px;
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.name-display:hover { background: var(--bg-elevated); }
.edit-hint { color: var(--tx-dim); font-size: 0.8rem; margin-left: 0.4rem; }
.name-input {
  flex: 1; background: var(--bg-elevated); border: 1px solid var(--ac);
  color: var(--tx-hi); font-size: 1rem; padding: 0.35rem 0.5rem; border-radius: 4px;
}
.star { background: none; border: none; cursor: pointer; font-size: 1.2rem; color: var(--tx-dim); line-height: 1; }
.star.on { color: #fbbf24; }
.close { background: none; border: none; color: var(--tx-dim); font-size: 1.1rem; cursor: pointer; }
.close:hover { color: var(--tx-hi); }
.tabs {
  display: flex; gap: 0.25rem; padding: 0.5rem 1rem 0;
  border-bottom: 1px solid var(--bd-dim);
}
.tabs button {
  background: none; border: none; border-bottom: 2px solid transparent;
  color: var(--tx-dim); font-size: 0.85rem; padding: 0.5rem 0.9rem; cursor: pointer;
}
.tabs button.active { color: var(--tx-hi); border-bottom-color: var(--ac); }
.content { flex: 1; overflow-y: auto; overflow-x: hidden; padding: 1rem; }
.status { color: var(--tx-dim); text-align: center; padding: 3rem; }

.replay-panel {
  display: flex; flex-direction: column; align-items: center;
  gap: 1.5rem; padding: 2rem 1rem;
}
.meta {
  display: grid; grid-template-columns: repeat(4, 1fr);
  gap: 1rem; width: 100%; max-width: 560px;
}
.meta > div {
  display: flex; flex-direction: column; align-items: center;
  background: var(--bg-card); border: 1px solid var(--bd-dim);
  border-radius: 8px; padding: 0.8rem 0.5rem;
}
.meta span { color: var(--tx-dim); font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.05em; }
.meta strong { color: var(--tx-hi); font-size: 1rem; margin-top: 0.25rem; }
.laps { width: 100%; max-width: 360px; display: flex; flex-direction: column; gap: 0.15rem; }
.laps-title {
  color: var(--tx-dim); font-size: 0.7rem;
  text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 0.25rem;
}
.lap-row {
  display: flex; justify-content: space-between;
  padding: 0.35rem 0.6rem; border-radius: 5px;
  background: var(--bg-card); border: 1px solid var(--bd-dim);
  color: var(--tx-mid); font-size: 0.82rem;
}
.lap-row.best { border-color: #a855f7; color: #a855f7; font-weight: 700; }
.lap-time { font-variant-numeric: tabular-nums; }
.replay-help { color: var(--tx-lo); font-size: 0.85rem; text-align: center; max-width: 420px; }
.replay-go {
  background: var(--ac); color: #fff; border: none;
  border-radius: 8px; padding: 0.7rem 1.6rem;
  font-size: 0.95rem; font-weight: 600; cursor: pointer;
}
.replay-go:hover { filter: brightness(1.1); }
</style>