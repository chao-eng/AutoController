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
} from '@lucide/vue'

const router = useRouter()
const route = useRoute()

const navItems = [
  { icon: Monitor, label: '设备', path: '/devices' },
  { icon: FileCode2, label: '脚本', path: '/scripts' },
  { icon: Settings, label: '配置', path: '/config' },
  { icon: CalendarClock, label: '任务', path: '/scheduler' },
  { icon: Bell, label: '通知', path: '/notifications' },
  { icon: ScrollText, label: '日志', path: '/logs' },
]


const activePath = computed(() => route.path)

function navigate(path: string) {
  router.push(path)
}
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-logo">
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
        <!-- Back shell -->
        <path fill="url(#side-slate)" d="M 256,132 C 190,132 115,142 85,172 C 55,202 35,285 43,345 C 51,405 85,446 140,436 C 190,426 215,356 256,360 C 297,360 317,426 367,436 C 422,446 456,405 464,345 C 472,285 452,202 422,172 C 392,142 317,132 256,132 Z" />
        <!-- Front shell -->
        <path fill="url(#side-titanium)" stroke="#E4E4E7" stroke-width="2" d="M 256,140 C 210,140 160,150 120,180 C 90,202 75,260 76,310 C 77,355 105,395 136,395 C 170,395 195,335 256,335 C 317,335 342,395 376,395 C 407,395 435,355 436,310 C 437,260 422,202 392,180 C 352,150 302,140 256,140 Z" />
        <!-- Glowing LED seams -->
        <path stroke="url(#side-electric)" stroke-width="8" stroke-linecap="round" d="M 124,152 Q 190,138 256,138" />
        <path stroke="url(#side-electric)" stroke-width="8" stroke-linecap="round" d="M 388,152 Q 322,138 256,138" />
        <!-- Joysticks -->
        <circle cx="164" cy="236" r="46" fill="url(#side-slate)" stroke="url(#side-ring)" stroke-width="4"/>
        <circle cx="164" cy="236" r="10" fill="url(#side-electric)"/>
        <circle cx="304" cy="304" r="46" fill="url(#side-slate)" stroke="url(#side-ring)" stroke-width="4"/>
        <circle cx="304" cy="304" r="10" fill="url(#side-electric)"/>
        <!-- D-PAD -->
        <circle cx="152" cy="320" r="36" fill="#09090B" stroke="url(#side-ring)" stroke-width="3"/>
        <path fill="url(#side-electric)" d="M 145,302 H 159 V 319 H 176 V 333 H 159 V 350 H 145 V 333 H 128 V 319 H 145 Z" />
        <!-- ABXY Buttons -->
        <circle cx="360" cy="187" r="18" fill="#09090B" stroke="url(#side-ring)" stroke-width="3"/>
        <circle cx="335" cy="212" r="18" fill="#09090B" stroke="url(#side-ring)" stroke-width="3"/>
        <circle cx="385" cy="212" r="18" fill="#09090B" stroke="url(#side-ring)" stroke-width="3"/>
        <circle cx="360" cy="237" r="18" fill="#09090B" stroke="url(#side-ring)" stroke-width="3"/>
        <!-- Guide Button -->
        <circle cx="256" cy="196" r="24" fill="#09090B" stroke="url(#side-electric)" stroke-width="5"/>
        <circle cx="256" cy="196" r="7" fill="url(#side-electric)" />
      </svg>
    </div>
    <nav class="sidebar-nav">
      <button
        v-for="item in navItems"
        :key="item.path"
        class="sidebar-item"
        :class="{ active: activePath === item.path }"
        @click="navigate(item.path)"
        :title="item.label"
      >
        <component :is="item.icon" :size="20" :stroke-width="1.5" />
      </button>
    </nav>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  height: 100%;
  background: var(--color-surface);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: var(--space-md) 0;
  flex-shrink: 0;
}

.sidebar-logo {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-cta);
  margin-bottom: var(--space-lg);
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: var(--space-xs);
}

.sidebar-item {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  color: var(--color-text-dim);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.sidebar-item:hover {
  color: var(--color-text);
  background: var(--color-surface-elevated);
}

.sidebar-item.active {
  color: var(--color-cta);
  background: rgba(51, 112, 255, 0.1);
}
</style>
