<script setup lang="ts">
import { ref, watch } from 'vue'
import { useSessionsStore } from '@/stores/sessions'
import type { AppSettings } from '@/fh6-tel/lib/types'

const emit = defineEmits<{ close: [] }>()

const sessionsStore = useSessionsStore()

const draft = ref<AppSettings | null>(null)

watch(() => sessionsStore.settings, (s) => {
  if (s && !draft.value) {
    draft.value = { ...s }
  }
}, { immediate: true })

async function save() {
  if (!draft.value) return
  await sessionsStore.saveSettings(draft.value)
  emit('close')
}
</script>

<template>
  <div v-if="draft" class="overlay" role="dialog" aria-modal="true">
    <div class="modal">
      <h2 class="m-0 text-[var(--tx-hi)] text-[1.1rem]">设置</h2>

      <label class="flex flex-col gap-0.5 text-[var(--tx-mid)] text-[0.85rem]">
        UDP 接收端口
        <input
          v-model.number="draft.port"
          type="number" min="1024" max="65535"
          class="bg-[var(--bg-body)] border border-[var(--bd-muted)] rounded text-[var(--tx-hi)] px-2 py-1.5 text-[0.9rem] w-full"
        />
        <span class="text-[0.7rem] text-[var(--tx-dim)] mt-[0.15rem]">端口更改将在重新启动应用后生效。</span>
      </label>

      <label class="flex flex-col gap-0.5 text-[var(--tx-mid)] text-[0.85rem]">
        速度单位
        <select
          v-model="draft.useMph"
          class="bg-[var(--bg-body)] border border-[var(--bd-muted)] rounded text-[var(--tx-hi)] px-2 py-1.5 text-[0.9rem] w-full"
        >
          <option :value="true">mph</option>
          <option :value="false">km/h</option>
        </select>
      </label>

      <label class="flex flex-row items-center gap-2 text-[var(--tx-mid)] text-[0.85rem]">
        <input type="checkbox" v-model="draft.autoRecord" />
        自动记录游戏会话
      </label>

      <fieldset class="border border-[var(--bd-dim)] rounded p-3 flex flex-col gap-2">
        <legend class="text-[var(--tx-dim)] text-[0.8rem] font-semibold px-1">轮胎温度区间 (°C)</legend>
        <label class="flex items-center gap-2 text-[var(--tx-mid)] text-[0.85rem]">
          低温区间低于
          <input
            v-model.number="draft.tireTempCold"
            type="number"
            class="bg-[var(--bg-body)] border border-[var(--bd-muted)] rounded text-[var(--tx-hi)] px-2 py-1 text-[0.9rem] w-[100px]"
          />
        </label>
        <label class="flex items-center gap-2 text-[var(--tx-mid)] text-[0.85rem]">
          合适区间最高
          <input
            v-model.number="draft.tireTempOptimal"
            type="number"
            class="bg-[var(--bg-body)] border border-[var(--bd-muted)] rounded text-[var(--tx-hi)] px-2 py-1 text-[0.9rem] w-[100px]"
          />
        </label>
        <label class="flex items-center gap-2 text-[var(--tx-mid)] text-[0.85rem]">
          高温区间高于
          <input
            v-model.number="draft.tireTempHot"
            type="number"
            class="bg-[var(--bg-body)] border border-[var(--bd-muted)] rounded text-[var(--tx-hi)] px-2 py-1 text-[0.9rem] w-[100px]"
          />
        </label>
      </fieldset>

      <div class="flex justify-end gap-2">
        <button
          @click="emit('close')"
          class="px-4 py-1.5 rounded border border-[var(--bd-muted)] bg-[var(--bg-elevated)] text-[var(--tx-mid)] cursor-pointer text-[0.85rem] hover:brightness-110"
        >取消</button>
        <button
          @click="save"
          class="px-4 py-1.5 rounded border border-[var(--ac)] bg-[var(--ac)] text-[var(--bg-body)] cursor-pointer text-[0.85rem] hover:brightness-110"
        >保存</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed; inset: 0; background: rgba(31,35,41,0.45);
  display: flex; align-items: center; justify-content: center; z-index: 100;
}
.modal {
  background: var(--bg-elevated); border: 1px solid var(--bd-muted); border-radius: 10px;
  padding: 1.5rem; width: 420px; max-height: 88vh; overflow-y: auto;
  display: flex; flex-direction: column; gap: 1rem;
}
</style>