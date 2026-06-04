<script setup lang="ts">
import { ref, computed, watch, onUnmounted, nextTick } from 'vue'
import uPlot from 'uplot'
import 'uplot/dist/uPlot.min.css'
import type { TelemetryPacket, SessionLap } from '@/fh6-tel/lib/types'
import { splitLaps, buildLapChips, metricGroups, buildChart, LAP_PALETTE } from '@/fh6-tel/lib/analysis'
import type { LapChip } from '@/fh6-tel/lib/analysis'

const props = defineProps<{
  packets: TelemetryPacket[]
  laps: SessionLap[]
  useMph: boolean
}>()

const chips = computed(() => buildLapChips(splitLaps(props.packets), props.laps))

const selectedKeys = ref<Set<string>>(new Set())
const colorAssignments = ref<Map<string, string>>(new Map())

watch(chips, (c) => {
  if (c.length && selectedKeys.value.size === 0) {
    const best = c.find((ch: { isBest: boolean }) => ch.isBest) ?? c[0]
    selectedKeys.value = new Set([best.key])
    colorAssignments.value = new Map([[best.key, LAP_PALETTE[0]]])
  }
}, { immediate: true })

const selected = computed(() =>
  chips.value.filter((c: { key: string }) => selectedKeys.value.has(c.key))
)

const coloredSelected = computed<LapChip[]>(() =>
  selected.value.map((chip) => ({
    ...chip,
    color: colorAssignments.value.get(chip.key) ?? LAP_PALETTE[0],
  }))
)

function toggle(key: string) {
  const nextKeys = new Set(selectedKeys.value)
  const nextColors = new Map(colorAssignments.value)
  if (nextKeys.has(key)) {
    if (nextKeys.size === 1) return
    nextKeys.delete(key)
    nextColors.delete(key)
  } else {
    nextKeys.add(key)
    const used = new Set(nextColors.values())
    const free = LAP_PALETTE.find((c: string) => !used.has(c))
      ?? LAP_PALETTE[(nextKeys.size - 1) % LAP_PALETTE.length]
    nextColors.set(key, free)
  }
  selectedKeys.value = nextKeys
  colorAssignments.value = nextColors
}

function formatLap(seconds: number) {
  const m = Math.floor(seconds / 60)
  const s = (seconds % 60).toFixed(1).padStart(4, '0')
  return `${m}:${s}`
}

const chartHost = ref<HTMLDivElement | null>(null)
let plots: uPlot[] = []
let resizeObserver: ResizeObserver | null = null

function destroyPlots() {
  for (const p of plots) p.destroy()
  plots = []
}

onUnmounted(() => {
  resizeObserver?.disconnect()
  destroyPlots()
})

function fmtElapsed(s: number) {
  const m = Math.floor(s / 60)
  return `${m}:${(s % 60).toFixed(2).padStart(5, '0')}`
}

function makeTooltip(
  metrics: string[],
  laps: Array<{ label: string; color: string }>,
): uPlot.Plugin {
  let el: HTMLDivElement
  const nm = metrics.length
  const colTemplate = `max-content ${laps.map(() => 'minmax(38px,auto)').join(' ')}`

  return {
    hooks: {
      init(u: uPlot) {
        el = document.createElement('div')
        el.className = 'u-tt'
        u.over.appendChild(el)
        u.over.addEventListener('mouseleave', () => { el.style.display = 'none' })
      },
      setCursor(u: uPlot) {
        const idx = u.cursor.idx
        if (idx == null) { el.style.display = 'none'; return }
        const x = (u.data[0] as number[])[idx]
        if (x == null) { el.style.display = 'none'; return }

        let grid = `<div></div>`
        for (const lap of laps) {
          grid += `<div class="u-tt-hdr">
            <span class="u-tt-sw" style="background:${lap.color}"></span>${lap.label}
          </div>`
        }

        for (let mi = 0; mi < nm; mi++) {
          grid += `<div class="u-tt-metric">${metrics[mi]}</div>`
          for (let li = 0; li < laps.length; li++) {
            const v = (u.data[li * nm + mi + 1] as (number | null)[])[idx]
            grid += `<div class="u-tt-val">${v != null ? v.toFixed(1) : '—'}</div>`
          }
        }

        el.innerHTML =
          `<div class="u-tt-time">${fmtElapsed(x)}</div>` +
          `<div class="u-tt-grid" style="grid-template-columns:${colTemplate}">${grid}</div>`
        el.style.display = 'block'

        const cx = u.cursor.left ?? 0
        const tw = el.offsetWidth
        const flip = cx + 16 + tw > u.over.clientWidth
        el.style.left = flip ? `${cx - tw - 4}px` : `${cx + 12}px`
        el.style.top = `${Math.max(0, (u.cursor.top ?? 0) - el.offsetHeight / 2)}px`
      },
    },
  }
}

function rebuildCharts() {
  const host = chartHost.value
  const sel = coloredSelected.value
  const groups = metricGroups(props.useMph)
  if (!host || sel.length === 0) {
    destroyPlots()
    return
  }

  destroyPlots()
  host.innerHTML = ''

  for (const g of groups) {
    const chart = buildChart(g, sel)

    const wrap = document.createElement('div')
    wrap.className = 'chart-block'
    const h = document.createElement('div')
    h.className = 'chart-title'
    h.textContent = g.title
    wrap.appendChild(h)
    const mount = document.createElement('div')
    wrap.appendChild(mount)
    host.appendChild(wrap)

    const opts: uPlot.Options = {
      width: host.clientWidth || 700,
      height: 180,
      legend: { show: false },
      cursor: { sync: { key: 'replay-analysis' } },
      plugins: [makeTooltip(
        g.metrics.map((m: { label: string }) => m.label),
        sel.map((c) => ({ label: `第${c.lapNumber + 1}圈`, color: c.color })),
      )],
      scales: { x: { time: false } },
      series: [
        { label: 'Elapsed (s)' },
        ...chart.series.map((s: { label: string; stroke: string; dash: number[] }) => ({
          label: s.label,
          stroke: s.stroke,
          dash: s.dash,
          width: 1.25,
        })),
      ],
      axes: [
        { stroke: 'var(--tx-dim)', grid: { stroke: 'var(--bd-subtle)' } },
        { stroke: 'var(--tx-dim)', grid: { stroke: 'var(--bd-subtle)' } },
      ],
    }

    const data = [chart.x, ...chart.series.map((s) => s.data)] as uPlot.AlignedData
    plots.push(new uPlot(opts, data, mount))
  }
}

watch(chartHost, (host) => {
  resizeObserver?.disconnect()
  resizeObserver = null
  if (!host) return

  resizeObserver = new ResizeObserver(() => {
    nextTick(() => rebuildCharts())
  })
  resizeObserver.observe(host)
})

watch([coloredSelected, chartHost, () => props.useMph], () => {
  nextTick(() => rebuildCharts())
}, { deep: true })
</script>

<template>
  <section class="analysis-workspace">
    <header class="analysis-toolbar">
      <div class="min-w-0">
        <p class="analysis-eyebrow">Lap Compare</p>
        <h2>圈速遥测对比</h2>
      </div>
      <span class="selection-count">{{ selected.length }} / {{ chips.length }} 圈已选</span>
    </header>

    <div v-if="chips.length" class="lap-strip">
      <div class="lap-strip-label">选择圈次</div>
      <div class="lap-chips">
        <button
          v-for="chip in chips" :key="chip.key"
          class="chip"
          :class="{ on: selectedKeys.has(chip.key) }"
          :style="{ '--chip': colorAssignments.get(chip.key) ?? 'var(--bd-muted)' }"
          @click="toggle(chip.key)"
        >
          <span class="dot"></span>
          <span class="chip-label">{{ chip.label }}</span>
          <span class="chip-time">{{ chip.lapTime != null ? formatLap(chip.lapTime) : 'partial' }}</span>
          <span v-if="chip.isBest" class="chip-best">最快</span>
        </button>
      </div>
    </div>

    <div v-else class="analysis-empty">此会话还没有可对比的圈段。</div>

    <div ref="chartHost" class="charts"></div>
  </section>
</template>

<style scoped>
.analysis-workspace {
  width: 100%;
  padding: 1.25rem;
}

.analysis-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  border: 1px solid var(--bd-dim);
  border-radius: 8px;
  background:
    linear-gradient(120deg, color-mix(in srgb, #3370ff 9%, transparent), transparent 52%),
    color-mix(in srgb, var(--bg-card) 86%, transparent);
  padding: 0.85rem 0.95rem;
}

.analysis-eyebrow {
  color: var(--tx-xdim);
  font-size: 0.66rem;
  font-weight: 800;
  letter-spacing: 0.12em;
  line-height: 1;
  text-transform: uppercase;
}

.analysis-toolbar h2 {
  margin-top: 0.28rem;
  color: var(--tx-hi);
  font-size: 1rem;
  font-weight: 760;
  line-height: 1.25;
}

.selection-count {
  flex-shrink: 0;
  border: 1px solid var(--bd-dim);
  border-radius: 999px;
  background: var(--bg-panel);
  color: var(--tx-dim);
  font-size: 0.72rem;
  font-variant-numeric: tabular-nums;
  font-weight: 750;
  padding: 0.25rem 0.55rem;
}

.lap-strip {
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr);
  gap: 0.8rem;
  align-items: start;
  margin-top: 0.85rem;
  border: 1px solid var(--bd-dim);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-card) 76%, transparent);
  padding: 0.75rem;
}

.lap-strip-label {
  padding-top: 0.45rem;
  color: var(--tx-dim);
  font-size: 0.7rem;
  font-weight: 800;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.lap-chips {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 0.45rem;
  min-width: 0;
}

.chip {
  display: inline-flex;
  align-items: center;
  min-width: 0;
  min-height: 2.35rem;
  gap: 0.45rem;
  background: var(--bg-panel);
  border: 1px solid var(--bd-subtle);
  border-radius: 7px;
  color: var(--tx-dim);
  font-size: 0.78rem;
  padding: 0.42rem 0.55rem;
  cursor: pointer;
  transition: border-color 120ms ease, background-color 120ms ease, color 120ms ease, transform 120ms ease;
}

.chip:hover {
  border-color: color-mix(in srgb, var(--chip) 50%, var(--bd-muted));
  color: var(--tx-mid);
}

.chip:active {
  transform: scale(0.99);
}

.chip.on {
  border-color: var(--chip);
  color: var(--tx-hi);
  background: color-mix(in srgb, var(--chip) 11%, var(--bg-panel));
}

.chip .dot {
  width: 0.55rem;
  height: 0.55rem;
  border-radius: 50%;
  background: var(--chip);
  opacity: 0.45;
  flex-shrink: 0;
}

.chip.on .dot { opacity: 1; }

.chip-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 700;
}

.chip-time {
  margin-left: auto;
  font-variant-numeric: tabular-nums;
  color: var(--tx-mid);
  font-weight: 760;
}

.chip-best {
  border-radius: 999px;
  background: color-mix(in srgb, #f59e0b 14%, transparent);
  color: color-mix(in srgb, #f59e0b 78%, black);
  font-size: 0.64rem;
  font-weight: 800;
  padding: 0.05rem 0.28rem;
}

.analysis-empty {
  margin-top: 0.85rem;
  border: 1px dashed var(--bd-muted);
  border-radius: 8px;
  color: var(--tx-dim);
  font-size: 0.85rem;
  padding: 1.25rem;
  text-align: center;
}

.charts {
  display: flex;
  flex-direction: column;
  gap: 0.9rem;
  margin-top: 1rem;
}

@media (max-width: 640px) {
  .analysis-workspace {
    padding: 1rem;
  }

  .analysis-toolbar,
  .lap-strip {
    grid-template-columns: 1fr;
  }

  .analysis-toolbar {
    flex-direction: column;
    align-items: flex-start;
  }

  .selection-count {
    align-self: flex-start;
  }

  .lap-strip-label {
    padding-top: 0;
  }

  .lap-chips {
    grid-template-columns: 1fr;
  }
}
</style>

<style>
.chart-block {
  overflow: hidden;
  background:
    linear-gradient(180deg, color-mix(in srgb, var(--bg-card) 86%, white), var(--bg-card)),
    var(--bg-card);
  border: 1px solid var(--bd-dim);
  border-radius: 8px;
  padding: 0.72rem 0.78rem 0.82rem;
}
.chart-title {
  color: var(--tx-dim);
  font-size: 0.8rem;
  font-weight: 760;
  margin-bottom: 0.5rem;
}
.uplot { background: transparent !important; }
.uplot .u-select {
  background: color-mix(in srgb, var(--ac) 22%, transparent);
  border: 1px solid var(--ac);
}
.u-tt {
  position: absolute;
  pointer-events: none;
  display: none;
  background: var(--bg-panel);
  border: 1px solid var(--bd-subtle);
  border-radius: 6px;
  padding: 0.35rem 0.55rem;
  font-size: 0.72rem;
  z-index: 10;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.45);
}
.u-tt-time {
  color: var(--tx-dim);
  font-variant-numeric: tabular-nums;
  font-size: 0.68rem;
  margin-bottom: 0.3rem;
}
.u-tt-grid {
  display: grid;
  column-gap: 0.55rem;
  row-gap: 0.12rem;
  align-items: center;
}
.u-tt-hdr {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.25rem;
  color: var(--tx-mid);
  font-weight: 600;
  padding-bottom: 0.15rem;
  border-bottom: 1px solid var(--bd-dim);
}
.u-tt-sw {
  width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0;
}
.u-tt-metric {
  color: var(--tx-mid); white-space: nowrap; padding-right: 0.3rem;
}
.u-tt-val {
  font-variant-numeric: tabular-nums;
  color: var(--tx-hi); font-weight: 600;
  text-align: right; white-space: nowrap;
}
</style>
