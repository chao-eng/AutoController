<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import {
  Monitor,
  FileCode2,
  Settings,
  CalendarClock,
  ScrollText,
  Bell,
  EyeOff,
} from '@lucide/vue'
import ForzaIcon from '../icons/ForzaIcon.vue'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip'

const router = useRouter()
const route = useRoute()

const navItems = [
  { icon: Monitor, label: '设备', path: '/devices' },
  { icon: FileCode2, label: '脚本', path: '/scripts' },
  { icon: Settings, label: '配置', path: '/config' },
  { icon: CalendarClock, label: '任务', path: '/scheduler' },
  { icon: Bell, label: '通知', path: '/notifications' },
  { icon: EyeOff, label: '防失去焦点', path: '/nofocus' },
  { icon: ForzaIcon, label: 'Forza 遥测', path: '/forza-telemetry' },
  { icon: ScrollText, label: '日志', path: '/logs' },
]

const activePath = computed(() => route.path)

function navigate(path: string) {
  router.push(path)
}
</script>

<template>
  <aside class="flex h-full w-[52px] flex-col items-center border-r border-border bg-background py-4 flex-shrink-0">
    <div class="mb-4 flex h-10 w-10 items-center justify-center text-primary">
      <svg width="28" height="28" viewBox="0 0 512 512" fill="none" xmlns="http://www.w3.org/2000/svg">
        <defs>
          <linearGradient id="side-titanium" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#FFFFFF" />
            <stop offset="35%" stop-color="#F4F4F5" />
            <stop offset="70%" stop-color="#E4E4E7" />
            <stop offset="100%" stop-color="#D4D4D8" />
          </linearGradient>
          <linearGradient id="side-electric" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#3B82F6" />
            <stop offset="100%" stop-color="#60A5FA" />
          </linearGradient>
          <linearGradient id="side-slate" x1="0%" y1="0%" x2="0%" y2="100%">
            <stop offset="0%" stop-color="#27272A" />
            <stop offset="100%" stop-color="#09090B" />
          </linearGradient>
          <linearGradient id="side-ring" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stop-color="#A1A1AA" />
            <stop offset="100%" stop-color="#52525B" />
          </linearGradient>
        </defs>
        <path fill="url(#side-slate)" d="M 256,132 C 190,132 115,142 85,172 C 55,202 35,285 43,345 C 51,405 85,446 140,436 C 190,426 215,356 256,360 C 297,360 317,426 367,436 C 422,446 456,405 464,345 C 472,285 452,202 422,172 C 392,142 317,132 256,132 Z" />
        <path fill="url(#side-titanium)" stroke="#E4E4E7" stroke-width="2" d="M 256,140 C 210,140 160,150 120,180 C 90,202 75,260 76,310 C 77,355 105,395 136,395 C 170,395 195,335 256,335 C 317,335 342,395 376,395 C 407,395 435,355 436,310 C 437,260 422,202 392,180 C 352,150 302,140 256,140 Z" />
        <path stroke="url(#side-electric)" stroke-width="8" stroke-linecap="round" d="M 124,152 Q 190,138 256,138" />
        <path stroke="url(#side-electric)" stroke-width="8" stroke-linecap="round" d="M 388,152 Q 322,138 256,138" />
        <circle cx="164" cy="236" r="46" fill="url(#side-slate)" stroke="url(#side-ring)" stroke-width="4"/>
        <circle cx="164" cy="236" r="10" fill="url(#side-electric)"/>
        <circle cx="304" cy="304" r="46" fill="url(#side-slate)" stroke="url(#side-ring)" stroke-width="4"/>
        <circle cx="304" cy="304" r="10" fill="url(#side-electric)"/>
        <circle cx="152" cy="320" r="36" fill="#09090B" stroke="url(#side-ring)" stroke-width="3"/>
        <path fill="url(#side-electric)" d="M 145,302 H 159 V 319 H 176 V 333 H 159 V 350 H 145 V 333 H 128 V 319 H 145 Z" />
        <circle cx="360" cy="187" r="18" fill="#09090B" stroke="url(#side-ring)" stroke-width="3"/>
        <circle cx="335" cy="212" r="18" fill="#09090B" stroke="url(#side-ring)" stroke-width="3"/>
        <circle cx="385" cy="212" r="18" fill="#09090B" stroke="url(#side-ring)" stroke-width="3"/>
        <circle cx="360" cy="237" r="18" fill="#09090B" stroke="url(#side-ring)" stroke-width="3"/>
        <circle cx="256" cy="196" r="24" fill="#09090B" stroke="url(#side-electric)" stroke-width="5"/>
        <circle cx="256" cy="196" r="7" fill="url(#side-electric)" />
      </svg>
    </div>
    <nav class="flex flex-col gap-1">
      <TooltipProvider v-for="item in navItems" :key="item.path">
        <Tooltip>
          <TooltipTrigger as-child>
            <button
              class="flex h-10 w-10 items-center justify-center rounded-md text-muted-foreground transition-all hover:bg-accent hover:text-accent-foreground"
              :class="{ 'text-primary bg-primary/10': activePath === item.path }"
              @click="navigate(item.path)"
            >
              <component :is="item.icon" :size="20" :stroke-width="1.5" />
            </button>
          </TooltipTrigger>
          <TooltipContent side="right">
            <p>{{ item.label }}</p>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </nav>
  </aside>
</template>