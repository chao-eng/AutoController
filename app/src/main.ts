import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import './styles/tailwind.css'
import './styles/base.css'

// 生产环境禁用鼠标右键菜单，提升原生桌面软件体验；开发环境允许右键操作（方便调试）
if (import.meta.env.PROD) {
  window.addEventListener('contextmenu', (e) => {
    e.preventDefault()
  })
}

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
