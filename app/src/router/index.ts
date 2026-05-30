import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      redirect: '/devices',
    },
    {
      path: '/devices',
      name: 'devices',
      component: () => import('../views/DeviceMonitor.vue'),
    },

    {
      path: '/scripts',
      name: 'scripts',
      component: () => import('../views/ScriptEditor.vue'),
    },
    {
      path: '/config',
      name: 'config',
      component: () => import('../views/ConfigPanel.vue'),
    },
    {
      path: '/scheduler',
      name: 'scheduler',
      component: () => import('../views/TaskScheduler.vue'),
    },
    {
      path: '/notifications',
      name: 'notifications',
      component: () => import('../views/NotificationConfig.vue'),
    },
    {
      path: '/nofocus',
      name: 'nofocus',
      component: () => import('../views/NoFocusLoss.vue'),
    },
    {
      path: '/logs',
      name: 'logs',
      component: () => import('../views/LogViewer.vue'),
    },
  ],
})

export default router
