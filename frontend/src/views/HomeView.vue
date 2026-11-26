<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import { useAuthStore } from '../stores/auth'
import { useHealthStore } from '../stores/health'

const auth = useAuthStore()
const health = useHealthStore()
const router = useRouter()

const logoutError = ref(null)

// Runs one of the two logout actions and returns to the login page.
async function signOut(action) {
  logoutError.value = null
  const result = await action()
  if (auth.isSignedIn) {
    logoutError.value = result.message
  } else {
    await router.replace({ name: 'login' })
  }
}

const labels = {
  unknown: 'Verbindung wird geprüft …',
  ok: 'Backend erreichbar',
  degraded: 'Backend erreichbar, Datenbank nicht',
  unreachable: 'Backend nicht erreichbar',
}

const colors = {
  unknown: 'bg-slate-400',
  ok: 'bg-emerald-500',
  degraded: 'bg-amber-500',
  unreachable: 'bg-red-500',
}

const label = computed(() => labels[health.status])
const color = computed(() => colors[health.status])

onMounted(() => health.check())
</script>

<template>
  <main class="mx-auto flex max-w-md flex-col gap-6 p-6">
    <header>
      <h1 class="text-4xl font-semibold tracking-tight text-teal-700">florg</h1>
      <p class="mt-1 text-slate-600">Fachlehrer Organizer</p>
    </header>

    <section class="rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
      <div class="flex items-center gap-3">
        <span class="size-3 shrink-0 rounded-full" :class="color" aria-hidden="true" />
        <span data-testid="health-label">{{ label }}</span>
      </div>
      <p v-if="health.version" class="mt-2 text-sm text-slate-500">
        Version {{ health.version }}
      </p>
      <button
        type="button"
        class="mt-4 rounded-lg bg-teal-700 px-4 py-2 text-sm font-medium text-white active:bg-teal-800"
        @click="health.check()"
      >
        Erneut prüfen
      </button>
    </section>

    <section class="rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
      <p class="text-slate-700">Du bist angemeldet.</p>
      <button
        type="button"
        class="mt-4 w-full rounded-lg bg-teal-700 px-4 py-3 font-medium text-white active:bg-teal-800"
        @click="signOut(auth.logout)"
      >
        Abmelden
      </button>
      <button
        type="button"
        class="mt-3 w-full px-4 py-2 text-sm text-slate-500 underline underline-offset-2"
        @click="signOut(auth.logoutAll)"
      >
        Überall abmelden
      </button>
      <p v-if="logoutError" class="mt-3 text-sm text-red-700" role="alert">
        {{ logoutError }}
      </p>
    </section>
  </main>
</template>
