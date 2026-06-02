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
  <div v-if="draft" class="fixed inset-0 bg-black/45 flex items-center justify-center z-50" role="dialog" aria-modal="true">
    <div class="bg-popover text-popover-foreground border border-border rounded-xl p-6 w-[420px] max-h-[88vh] overflow-y-auto flex flex-col gap-4">
      <h2 class="m-0 text-foreground text-[1.1rem]">设置</h2>

      <label class="flex flex-col gap-0.5 text-muted-foreground text-[0.85rem]">
        UDP 接收端口
        <input
          v-model.number="draft.port"
          type="number" min="1024" max="65535"
          class="bg-background border border-border rounded text-foreground px-2 py-1.5 text-[0.9rem] w-full"
        />
        <span class="text-[0.7rem] text-muted-foreground/70 mt-[0.15rem]">端口更改将在重新启动应用后生效。</span>
      </label>

      <label class="flex flex-col gap-0.5 text-muted-foreground text-[0.85rem]">
        速度单位
        <select
          v-model="draft.useMph"
          class="bg-background border border-border rounded text-foreground px-2 py-1.5 text-[0.9rem] w-full"
        >
          <option :value="true">mph</option>
          <option :value="false">km/h</option>
        </select>
      </label>

      <label class="flex flex-row items-center gap-2 text-muted-foreground text-[0.85rem]">
        <input type="checkbox" v-model="draft.autoRecord" class="accent-primary" />
        自动记录游戏会话
      </label>

      <fieldset class="border border-border/60 rounded-lg p-3 flex flex-col gap-2">
        <legend class="text-muted-foreground/70 text-[0.8rem] font-semibold px-1">轮胎温度区间 (°C)</legend>
        <label class="flex items-center gap-2 text-muted-foreground text-[0.85rem]">
          低温区间低于
          <input
            v-model.number="draft.tireTempCold"
            type="number"
            class="bg-background border border-border rounded text-foreground px-2 py-1 text-[0.9rem] w-[100px]"
          />
        </label>
        <label class="flex items-center gap-2 text-muted-foreground text-[0.85rem]">
          合适区间最高
          <input
            v-model.number="draft.tireTempOptimal"
            type="number"
            class="bg-background border border-border rounded text-foreground px-2 py-1 text-[0.9rem] w-[100px]"
          />
        </label>
        <label class="flex items-center gap-2 text-muted-foreground text-[0.85rem]">
          高温区间高于
          <input
            v-model.number="draft.tireTempHot"
            type="number"
            class="bg-background border border-border rounded text-foreground px-2 py-1 text-[0.9rem] w-[100px]"
          />
        </label>
      </fieldset>

      <div class="flex justify-end gap-2">
        <button
          @click="emit('close')"
          class="px-4 py-1.5 rounded-lg border border-border bg-card text-muted-foreground cursor-pointer text-[0.85rem] hover:bg-muted transition-colors"
        >取消</button>
        <button
          @click="save"
          class="px-4 py-1.5 rounded-lg border border-primary bg-primary text-primary-foreground cursor-pointer text-[0.85rem] hover:brightness-110 transition-colors"
        >保存</button>
      </div>
    </div>
  </div>
</template>