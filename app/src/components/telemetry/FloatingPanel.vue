<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const props = defineProps<{
  id: string
  title: string
  defaultWidth?: number
  defaultTop?: number
  defaultBottom?: number
  resizable?: boolean
  hidden?: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const x = ref(0)
const y = ref(0)
const w = ref(200)
const ready = ref(false)

onMounted(() => {
  const saved = localStorage.getItem(props.id)
  if (saved) {
    try {
      const s = JSON.parse(saved)
      x.value = s.x
      y.value = s.y
      w.value = s.w
    } catch { /* fall through */ }
  }
  if (!saved) {
    w.value = props.defaultWidth ?? 200
    x.value = window.innerWidth - (props.defaultWidth ?? 200)
    y.value = props.defaultTop ?? window.innerHeight - (props.defaultWidth ?? 200) - (props.defaultBottom ?? 0)
  }
  ready.value = true
})

function persist() {
  localStorage.setItem(props.id, JSON.stringify({ x: x.value, y: y.value, w: w.value }))
}

const dragging = ref(false)
let dragStartX = 0, dragStartY = 0, originX = 0, originY = 0

function startDrag(e: PointerEvent) {
  e.preventDefault()
  dragging.value = true
  dragStartX = e.clientX
  dragStartY = e.clientY
  originX = x.value
  originY = y.value
  window.addEventListener('pointermove', onDragMove)
  window.addEventListener('pointerup', stopDrag, { once: true })
}

function onDragMove(e: PointerEvent) {
  x.value = Math.max(0, Math.min(window.innerWidth - w.value, originX + e.clientX - dragStartX))
  y.value = Math.max(0, Math.min(window.innerHeight - 40, originY + e.clientY - dragStartY))
}

function stopDrag() {
  dragging.value = false
  window.removeEventListener('pointermove', onDragMove)
  persist()
}

const resizing = ref(false)
let resizeStartX = 0, originW = 0

function startResize(e: PointerEvent) {
  e.preventDefault()
  e.stopPropagation()
  resizing.value = true
  resizeStartX = e.clientX
  originW = w.value
  window.addEventListener('pointermove', onResizeMove)
  window.addEventListener('pointerup', stopResize, { once: true })
}

function onResizeMove(e: PointerEvent) {
  w.value = Math.max(120, Math.min(window.innerWidth - x.value, originW + e.clientX - resizeStartX))
}

function stopResize() {
  resizing.value = false
  window.removeEventListener('pointermove', onResizeMove)
  persist()
}

onUnmounted(() => {
  window.removeEventListener('pointermove', onDragMove)
  window.removeEventListener('pointermove', onResizeMove)
})
</script>

<template>
  <div v-if="ready">
    <div
      class="fp"
      :class="{ dragging }"
      :style="{ left: x + 'px', top: y + 'px', width: w + 'px' }"
      role="dialog"
      :aria-label="title"
    >
      <div class="fp-header" @pointerdown="startDrag">
        <span class="fp-grip" aria-hidden="true">⠿</span>
        <span class="fp-title">{{ title }}</span>
        <div class="fp-actions" @pointerdown.stop>
          <slot name="actions" />
          <button class="fp-close" @click="emit('close')" :aria-label="'关闭 ' + title">✕</button>
        </div>
      </div>
      <div class="fp-body">
        <slot />
      </div>
      <div v-if="resizable" class="fp-resize" @pointerdown="startResize" aria-hidden="true">
        <svg viewBox="0 0 10 10" width="10" height="10" aria-hidden="true">
          <path d="M9 1L1 9M9 5L5 9" stroke="currentColor" stroke-width="1.5"/>
        </svg>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fp {
  position: fixed;
  z-index: 50;
  background: var(--bg-panel);
  border: 1px solid var(--bd-subtle);
  border-radius: 6px;
  box-shadow: 0 4px 20px rgba(31,35,41,0.08);
  display: flex;
  flex-direction: column;
  isolation: isolate;
  min-width: 120px;
}
.fp.dragging { opacity: 0.9; cursor: grabbing; user-select: none; }
.fp-header {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.25rem 0.4rem;
  border-bottom: 1px solid var(--bd-dim);
  cursor: grab;
  background: var(--bg-elevated);
  border-radius: 6px 6px 0 0;
  user-select: none;
  flex-shrink: 0;
}
.fp-grip { color: var(--tx-xdim); font-size: 0.75rem; line-height: 1; }
.fp-title {
  flex: 1;
  font-size: 0.7rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  color: var(--tx-dim);
}
.fp-actions { display: flex; align-items: center; gap: 0.35rem; }
.fp-close {
  background: none; border: none; color: var(--tx-xdim);
  font-size: 0.8rem; cursor: pointer; padding: 0 0.1rem; line-height: 1;
}
.fp-close:hover { color: var(--tx-hi); }
.fp-body { flex: 1; min-height: 0; overflow: hidden; }
.fp-resize {
  position: absolute; bottom: 3px; right: 4px;
  width: 12px; height: 12px; cursor: se-resize;
  color: var(--tx-xdim); opacity: 0.6;
}
.fp-resize:hover { opacity: 1; }
</style>