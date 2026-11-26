import { createRouter, createWebHistory } from 'vue-router'

import { useAuthStore } from '../stores/auth'
import AccountView from '../views/AccountView.vue'
import ConfirmView from '../views/ConfirmView.vue'
import LoginView from '../views/LoginView.vue'
import SchoolClassesView from '../views/SchoolClassesView.vue'
import SchoolYearsView from '../views/SchoolYearsView.vue'
import SchoolYearView from '../views/SchoolYearView.vue'
import StudentsView from '../views/StudentsView.vue'
import SubjectsView from '../views/SubjectsView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    // There is no start page yet. Until there is one, the school years open.
    { path: '/', name: 'home', redirect: { name: 'schoolYears' } },
    { path: '/schuljahre', name: 'schoolYears', component: SchoolYearsView },
    // Pages of one school year carry its id as `yearId`. The header reads it.
    { path: '/schuljahre/:yearId(\\d+)', name: 'schoolYear', component: SchoolYearView },
    { path: '/schuljahre/:yearId(\\d+)/schueler', name: 'students', component: StudentsView },
    { path: '/faecher', name: 'subjects', component: SubjectsView },
    { path: '/klassen', name: 'schoolClasses', component: SchoolClassesView },
    { path: '/konto', name: 'account', component: AccountView },
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
