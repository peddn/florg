import { createRouter, createWebHistory } from 'vue-router'

import { useAuthStore } from '../stores/auth'
import ConfirmView from '../views/ConfirmView.vue'
import HomeView from '../views/HomeView.vue'
import LoginView from '../views/LoginView.vue'
import SchoolYearsView from '../views/SchoolYearsView.vue'
import SchoolYearView from '../views/SchoolYearView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'home', component: HomeView },
    { path: '/schuljahre', name: 'schoolYears', component: SchoolYearsView },
    { path: '/schuljahre/:id(\\d+)', name: 'schoolYear', component: SchoolYearView },
    { path: '/anmelden', name: 'login', component: LoginView, meta: { public: true } },
    // The path is part of every login link. Keep it in step with the backend.
    {
      path: '/anmelden/bestaetigen',
      name: 'confirm',
      component: ConfirmView,
      meta: { public: true },
    },
    // Unknown addresses lead to the start page.
    { path: '/:rest(.*)*', redirect: { name: 'home' } },
  ],
})

// Every page needs a session unless it is marked as public.
router.beforeEach(async (to) => {
  const auth = useAuthStore()
  if (auth.status === 'unknown') {
    await auth.load()
  }

  if (!to.meta.public && !auth.isSignedIn) {
    return { name: 'login' }
  }
  if (to.name === 'login' && auth.isSignedIn) {
    return { name: 'home' }
  }
})

export default router
