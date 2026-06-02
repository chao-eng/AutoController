<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useTelemetryStore } from '@/stores/telemetry'
import { carName } from '@/fh6-tel/lib/car-name'
import { CAR_CLASS_LABELS, DRIVETRAIN_LABELS } from '@/fh6-tel/lib/types'
import { ipc } from '@/fh6-tel/lib/ipc'

withDefaults(defineProps<{
  useMph?: boolean
  tiresVisible?: boolean
  onSettings?: () => void
  onSessions?: () => void
  onToggleTires?: () => void
}>(), {
  useMph: true,
  tiresVisible: true,
})

const telemetry = useTelemetryStore()
const pkt = computed(() => telemetry.displayPacket)
const connected = computed(() => telemetry.isConnected)

const carLabel = computed(() => pkt.value ? carName(pkt.value.carOrdinal) : '—')
const isUnknown = computed(() => carLabel.value.startsWith('Car #'))
const classLabel = computed(() => pkt.value ? (CAR_CLASS_LABELS[pkt.value.carClass] ?? '?') : '—')
const piLabel = computed(() => pkt.value ? String(pkt.value.carPi) : '—')
const driveLabel = computed(() => pkt.value ? (DRIVETRAIN_LABELS[pkt.value.drivetrainType] ?? '?') : '—')

const copied = ref(false)
const version = ref('')

onMounted(async () => {
  version.value = await ipc.getAppVersion()
})

async function copyOrdinal() {
  if (!pkt.value || !isUnknown.value) return
  await navigator.clipboard.writeText(String(pkt.value.carOrdinal))
  copied.value = true
  setTimeout(() => { copied.value = false }, 1800)
}

const classBadgeColors: Record<string, string> = {
  X: 'text-red-500 border-red-900',
  S2: 'text-orange-500 border-orange-900',
  S1: 'text-yellow-500 border-yellow-900',
  A: 'text-green-500 border-green-900',
  B: 'text-blue-500 border-blue-900',
  C: 'text-purple-500 border-purple-900',
  D: 'text-[var(--tx-lo)] border-[var(--bd-subtle)]',
}
</script>

<template>
  <header class="flex items-center justify-between px-4 h-12 bg-[var(--bg-panel)] border-b border-[var(--bd-dim)] shrink-0">
    <div class="flex items-center gap-1.5">
      <span
        class="w-2 h-2 rounded-full transition-colors duration-300"
        :class="connected ? 'bg-green-500 shadow-[0_0_6px_#22c55e]' : 'bg-red-500'"
      />
      <span class="text-[0.85rem] font-bold tracking-wider text-[var(--tx-dim)]">
        {{ connected ? '实时遥测' : '等待连接…' }}
      </span>
    </div>

    <div class="flex items-center gap-1.5 min-w-0 overflow-hidden">
      <span
        class="text-[clamp(0.8rem,1.8vw,1rem)] font-semibold text-[var(--tx-mid)] whitespace-nowrap overflow-hidden text-ellipsis max-w-[clamp(80px,22vw,260px)]"
        :class="isUnknown ? 'text-[var(--tx-dim)] cursor-copy hover:text-[var(--tx-lo)]' : ''"
        :title="isUnknown ? `车辆ID: ${pkt?.carOrdinal} — 点击复制` : undefined"
        @click="copyOrdinal"
      >
        {{ copied ? '已复制!' : carLabel }}
      </span>
      <span
        class="text-[clamp(0.65rem,1.4vw,0.8rem)] font-bold px-1.5 py-[0.15rem] border rounded flex-shrink-0 whitespace-nowrap"
        :class="classBadgeColors[classLabel] ?? 'text-[var(--tx-lo)] border-[var(--bd-muted)]'"
      >
        {{ classLabel }}
      </span>
      <span class="text-[clamp(0.65rem,1.4vw,0.8rem)] font-bold px-1.5 py-[0.15rem] border border-[var(--bd-muted)] rounded text-[var(--tx-lo)] flex-shrink-0 whitespace-nowrap">
        {{ piLabel }}
      </span>
      <span class="text-[clamp(0.65rem,1.4vw,0.8rem)] font-bold px-1.5 py-[0.15rem] border border-[var(--bd-muted)] rounded text-[var(--tx-lo)] flex-shrink-0 whitespace-nowrap">
        {{ driveLabel }}
      </span>
    </div>

    <div class="flex items-center gap-1.5">
      <button
        class="text-[0.65rem] font-bold tracking-wider px-1.5 py-[0.2rem] border rounded cursor-pointer transition-colors"
        :class="tiresVisible
          ? 'border-[var(--ac)] text-[var(--ac)]'
          : 'border-[var(--bd-muted)] text-[var(--tx-xdim)] hover:text-[var(--tx-mid)]'"
        @click="onToggleTires?.()"
        :title="tiresVisible ? '隐藏轮胎数据' : '显示轮胎数据'"
      >轮胎</button>
      <button class="bg-none border-none cursor-pointer text-[1.1rem] text-[var(--tx-dim)] px-2 py-1 rounded hover:bg-[var(--bg-elevated)] hover:text-[var(--tx-mid)]" @click="onSessions?.()" title="历史会话">⏱</button>
      <button class="bg-none border-none cursor-pointer text-[1.1rem] text-[var(--tx-dim)] px-2 py-1 rounded hover:bg-[var(--bg-elevated)] hover:text-[var(--tx-mid)]" @click="onSettings?.()" title="设置">⚙</button>
      <span v-if="version" class="text-[0.7rem] text-[var(--tx-xdim)] tracking-wide px-[0.1rem]">v{{ version }}</span>
    </div>
  </header>
</template>