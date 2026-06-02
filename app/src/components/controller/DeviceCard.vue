<script setup lang="ts">
import type { DeviceInfo } from '../../types/controller'
import { Circle, Trash2, Wifi, WifiOff, Power } from '@lucide/vue'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Card, CardContent } from '@/components/ui/card'

defineProps<{
  device: DeviceInfo
}>()

const emit = defineEmits<{
  remove: [deviceId: string]
  toggleConnection: [deviceId: string]
}>()
</script>

<template>
  <Card
    :class="[
      'cursor-default transition-all duration-300 hover:border-blue-500 hover:shadow-md',
      device.connected ? '' : 'opacity-60'
    ]"
    size="sm"
  >
    <CardContent class="flex flex-col gap-3">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="text-sm font-semibold">Xbox 360</span>
          <span class="text-[11px] text-muted-foreground">#{{ device.index }}</span>
        </div>
        <div class="flex items-center gap-1.5">
          <Badge
            :variant="device.vigem_connected ? 'outline' : 'secondary'"
            :class="device.vigem_connected ? 'text-green-500' : 'text-yellow-500'"
            :title="device.vigem_connected ? 'ViGEmBus 已连接（系统可识别）' : '模拟模式（系统不可识别）'"
          >
            <Wifi v-if="device.vigem_connected" :size="10" />
            <WifiOff v-else :size="10" />
          </Badge>
          <Circle :size="8" :fill="device.connected ? '#00B65B' : '#8F959E'" class="text-transparent" />
          <Button variant="ghost" size="icon-xs" :class="device.connected ? '' : 'text-muted-foreground'" :title="device.connected ? '断开手柄（保留在列表）' : '连接手柄（挂载虚拟设备）'" @click="emit('toggleConnection', device.id)">
            <Power :size="13" />
          </Button>
          <Button variant="ghost" size="icon-xs" class="hover:text-destructive hover:bg-destructive/15" title="移除设备" @click="emit('remove', device.id)">
            <Trash2 :size="13" />
          </Button>
        </div>
      </div>
      <div class="flex flex-col gap-1.5 text-[11px]">
        <div class="flex items-center gap-2">
          <span class="min-w-[48px] text-muted-foreground">L摇杆</span>
          <span class="font-mono text-muted-foreground">X:{{ device.state.left_thumb_x }} Y:{{ device.state.left_thumb_y }}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="min-w-[48px] text-muted-foreground">R摇杆</span>
          <span class="font-mono text-muted-foreground">X:{{ device.state.right_thumb_x }} Y:{{ device.state.right_thumb_y }}</span>
        </div>
        <div class="flex items-center gap-2">
          <span class="min-w-[48px] text-muted-foreground">L扳机</span>
          <div class="h-1.5 flex-1 overflow-hidden rounded-full bg-muted">
            <div class="h-full rounded-full bg-primary transition-all duration-100" :style="{ width: (device.state.left_trigger / 255 * 100) + '%' }"></div>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <span class="min-w-[48px] text-muted-foreground">R扳机</span>
          <div class="h-1.5 flex-1 overflow-hidden rounded-full bg-muted">
            <div class="h-full rounded-full bg-primary transition-all duration-100" :style="{ width: (device.state.right_trigger / 255 * 100) + '%' }"></div>
          </div>
        </div>
      </div>
    </CardContent>
  </Card>
</template>