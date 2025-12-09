import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { attachConsole } from '@tauri-apps/plugin-log'
import App from './App.vue'
import router from './router'
import 'vant/lib/index.css'
import 'uno.css'

// Forward frontend logs to Tauri backend
attachConsole()

const app = createApp(App)
app.use(createPinia())
app.use(router)
app.mount('#app')
