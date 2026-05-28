import { createApp } from 'vue'
import { createPinia } from 'pinia'
import router from './router'
import App from './App.vue'
import './styles/base.css'

// 禁用鼠标右键菜单，提升原生桌面软件体验
window.addEventListener('contextmenu', (e) => {
  e.preventDefault()
})

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
