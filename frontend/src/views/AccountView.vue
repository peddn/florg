<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'

import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const router = useRouter()

const busy = ref(false)
const error = ref(null)

// Ends every session of the account, including this one (rule L9).
async function logoutAll() {
  busy.value = true
  error.value = null
  const result = await auth.logoutAll()
  busy.value = false
  if (auth.isSignedIn) {
    error.value = result.message
  } else {
    await router.replace({ name: 'login' })
  }
}
</script>

<template>
  <main class="mx-auto max-w-6xl p-4">
    <h1 class="text-2xl font-semibold">Konto</h1>

    <section class="mt-4 max-w-2xl rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
      <h2 class="text-lg font-medium">Sitzungen</h2>
      <p class="mt-1 text-slate-600">
        „Überall abmelden“ beendet sofort alle Sitzungen, auch auf anderen Geräten und auf diesem.
        Danach meldest du dich mit einem neuen Anmeldelink wieder an.
      </p>
      <button
        type="button"
        class="mt-4 rounded-lg border border-red-300 px-4 py-2 font-medium text-red-700 hover:bg-red-50 disabled:opacity-60"
        :disabled="busy"
        @click="logoutAll"
      >
        Überall abmelden
      </button>
      <p v-if="error" class="mt-3 text-sm text-red-700" role="alert">{{ error }}</p>
    </section>
  </main>
</template>
