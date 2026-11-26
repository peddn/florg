<script setup>
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

// A link is highlighted on its own page and on every page below it.
const links = [{ to: { name: 'schoolYears' }, path: '/schuljahre', label: 'Schuljahre' }]

function isActive(link) {
  return route.path === link.path || route.path.startsWith(`${link.path}/`)
}

const error = ref(null)

// Signs out this device only (rule L9).
async function logout() {
  error.value = null
  const result = await auth.logout()
  if (auth.isSignedIn) {
    error.value = result.message
  } else {
    await router.replace({ name: 'login' })
  }
}
</script>

<template>
  <header class="border-b border-slate-200 bg-white">
    <nav class="mx-auto flex max-w-6xl items-center gap-1 px-4 py-2" aria-label="Hauptnavigation">
      <RouterLink
        :to="{ name: 'home' }"
        class="mr-3 text-xl font-semibold tracking-tight text-teal-700"
      >
        florg
      </RouterLink>
      <RouterLink
        v-for="link in links"
        :key="link.label"
        :to="link.to"
        class="rounded-lg px-3 py-2 text-sm font-medium"
        :class="isActive(link) ? 'bg-teal-50 text-teal-800' : 'text-slate-600 hover:bg-slate-100'"
        :aria-current="isActive(link) ? 'page' : undefined"
      >
        {{ link.label }}
      </RouterLink>

      <span class="grow" />

      <RouterLink
        :to="{ name: 'account' }"
        class="rounded-lg px-3 py-2 text-sm font-medium text-slate-600 hover:bg-slate-100"
        exact-active-class="bg-teal-50 text-teal-800 hover:bg-teal-50"
      >
        Konto
      </RouterLink>
      <button
        type="button"
        class="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-100"
        @click="logout"
      >
        Abmelden
      </button>
    </nav>
    <p v-if="error" class="mx-auto max-w-6xl px-4 pb-2 text-right text-sm text-red-700" role="alert">
      {{ error }}
    </p>
  </header>
</template>
