import { createPinia } from 'pinia'
import { createApp } from 'vue'

import { UNAUTHENTICATED_EVENT } from './api'
import App from './App.vue'
import router from './router'
import { useAuthStore } from './stores/auth'
import './style.css'

const pinia = createPinia()

// A session that ended while the app was open leads back to the login.
window.addEventListener(UNAUTHENTICATED_EVENT, () => {
  useAuthStore(pinia).status = 'signedOut'
  router.replace({ name: 'login' })
})

createApp(App).use(pinia).use(router).mount('#app')
