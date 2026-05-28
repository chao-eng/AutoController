<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import {
  Monitor,
  Gamepad2,
  FileCode2,
  Settings,
  CalendarClock,
  ScrollText,
} from '@lucide/vue'

const router = useRouter()
const route = useRoute()

const navItems = [
  { icon: Monitor, label: '设备', path: '/devices' },
  { icon: FileCode2, label: '脚本', path: '/scripts' },
  { icon: Settings, label: '配置', path: '/config' },
  { icon: CalendarClock, label: '任务', path: '/scheduler' },
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
      <Gamepad2 :size="24" :stroke-width="2" />
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
