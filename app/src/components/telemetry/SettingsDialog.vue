<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useSessionsStore } from '@/stores/sessions'
import type { AppSettings } from '@/fh6-tel/lib/types'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

const emit = defineEmits<{ close: [] }>()

const sessionsStore = useSessionsStore()

const draft = ref<AppSettings | null>(null)
const DASHBOARD_MAX_SPEED_MIN = 40
const DASHBOARD_MAX_SPEED_MAX = 600
const DASHBOARD_MAX_SPEED_DEFAULT = 180

function normalizeDashboardMaxSpeed(value: unknown) {
  const speed = typeof value === 'number' && Number.isFinite(value)
    ? Math.round(value)
    : DASHBOARD_MAX_SPEED_DEFAULT
  return Math.min(Math.max(speed, DASHBOARD_MAX_SPEED_MIN), DASHBOARD_MAX_SPEED_MAX)
}

watch(() => sessionsStore.settings, (s) => {
  if (s && !draft.value) {
    draft.value = {
      ...s,
      dashboardMaxSpeed: normalizeDashboardMaxSpeed(s.dashboardMaxSpeed),
    }
  }
}, { immediate: true })

const useMphModel = computed({
  get: () => draft.value?.useMph ? 'true' : 'false',
  set: (v: string) => { if (draft.value) draft.value.useMph = v === 'true' },
})
const speedUnitLabel = computed(() => draft.value?.useMph ? 'mph' : 'km/h')

async function save() {
  if (!draft.value) return
  const next = {
    ...draft.value,
    dashboardMaxSpeed: normalizeDashboardMaxSpeed(draft.value.dashboardMaxSpeed),
  }
  draft.value = next
  await sessionsStore.saveSettings(next)
  emit('close')
}
</script>

<template>
  <Dialog :open="true" @update:open="(v) => { if (!v) emit('close') }">
    <DialogContent class="sm:max-w-[420px]">
      <DialogHeader>
        <DialogTitle>设置</DialogTitle>
        <DialogDescription>配置遥测数据接收参数与显示选项</DialogDescription>
      </DialogHeader>
      <div v-if="draft" class="flex flex-col gap-4">
        <div class="flex flex-col gap-2">
          <Label class="text-muted-foreground">UDP 接收端口</Label>
          <Input
            v-model.number="draft.port"
            type="number" min="1024" max="65535"
          />
          <span class="text-xs text-muted-foreground/70">端口更改将在重新启动应用后生效。</span>
        </div>

        <div class="flex flex-col gap-2">
          <Label class="text-muted-foreground">速度单位</Label>
          <Select v-model="useMphModel">
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="true">mph</SelectItem>
              <SelectItem value="false">km/h</SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div class="flex flex-col gap-2">
          <Label class="text-muted-foreground">仪表盘最大速度</Label>
          <div class="flex items-center gap-2">
            <Input
              v-model.number="draft.dashboardMaxSpeed"
              type="number"
              :min="DASHBOARD_MAX_SPEED_MIN"
              :max="DASHBOARD_MAX_SPEED_MAX"
              step="10"
            />
            <span class="min-w-[3rem] text-sm font-semibold text-muted-foreground">{{ speedUnitLabel }}</span>
          </div>
          <span class="text-xs text-muted-foreground/70">用于控制速度仪表盘的刻度上限。</span>
        </div>

        <div class="flex items-center gap-2">
          <Checkbox id="autoRecord" v-model="draft.autoRecord" />
          <Label for="autoRecord" class="text-muted-foreground">自动记录游戏会话</Label>
        </div>

        <fieldset class="border border-border/60 rounded-lg p-3 flex flex-col gap-3">
          <legend class="text-muted-foreground/70 text-xs font-semibold px-1">轮胎温度区间 (°C)</legend>
          <div class="flex items-center gap-2">
            <Label class="min-w-[100px] text-muted-foreground text-sm">低温区间低于</Label>
            <Input
              v-model.number="draft.tireTempCold"
              type="number"
              class="w-[100px]"
            />
          </div>
          <div class="flex items-center gap-2">
            <Label class="min-w-[100px] text-muted-foreground text-sm">合适区间最高</Label>
            <Input
              v-model.number="draft.tireTempOptimal"
              type="number"
              class="w-[100px]"
            />
          </div>
          <div class="flex items-center gap-2">
            <Label class="min-w-[100px] text-muted-foreground text-sm">高温区间高于</Label>
            <Input
              v-model.number="draft.tireTempHot"
              type="number"
              class="w-[100px]"
            />
          </div>
        </fieldset>
      </div>
      <DialogFooter>
        <Button variant="outline" @click="emit('close')">取消</Button>
        <Button @click="save">保存</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
