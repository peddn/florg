<script setup>
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const route = useRoute()
const router = useRouter()

// The token sits behind the "#", so it never reaches a server log.
const token = new URLSearchParams(route.hash.slice(1)).get('token')

const sending = ref(false)
const error = ref(token ? null : 'Dieser Anmeldelink ist unvollständig.')

// The link itself does not sign in. Only this button does, so that mail
// scanners that follow links cannot use up the token.
async function confirm() {
  sending.value = true
  error.value = null
  const result = await auth.confirm(token)
  if (result.ok) {
    // Replacing the entry also drops the token from the address bar.
    await router.replace({ name: 'home' })
  } else {
    sending.value = false
    error.value = result.message
  }
}
</script>

<template>
  <main class="mx-auto flex min-h-[calc(100dvh-2rem)] max-w-md flex-col justify-center gap-6 p-6">
    <header>
      <h1 class="text-4xl font-semibold tracking-tight text-teal-700">florg</h1>
      <p class="mt-1 text-slate-600">Fachlehrer Organizer</p>
    </header>

    <section class="rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
      <h2 class="text-lg font-medium">Anmeldung bestätigen</h2>

      <template v-if="token && !error">
        <p class="mt-2 text-slate-700">Möchtest du dich auf diesem Gerät anmelden?</p>
        <button
          type="button"
          class="mt-4 w-full rounded-lg bg-teal-700 px-4 py-3 font-medium text-white active:bg-teal-800 disabled:opacity-60"
          :disabled="sending"
          @click="confirm"
        >
          {{ sending ? 'Wird angemeldet …' : 'Jetzt anmelden' }}
        </button>
      </template>

      <template v-else>
        <p class="mt-2 text-red-700" role="alert" data-testid="confirm-error">{{ error }}</p>
        <RouterLink
          :to="{ name: 'login' }"
          class="mt-4 block w-full rounded-lg bg-teal-700 px-4 py-3 text-center font-medium text-white active:bg-teal-800"
        >
          Neuen Link anfordern
        </RouterLink>
      </template>
    </section>
  </main>
</template>
