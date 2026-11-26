<script setup>
import { ref } from 'vue'

import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()

// 'idle' | 'sending' | 'mail' | 'log'
const state = ref('idle')
const error = ref(null)

async function requestLink() {
  state.value = 'sending'
  error.value = null
  const result = await auth.requestLink()
  if (result.ok) {
    state.value = result.data.delivery === 'log' ? 'log' : 'mail'
  } else {
    state.value = 'idle'
    error.value = result.message
  }
}
</script>

<template>
  <main class="mx-auto flex min-h-dvh max-w-md flex-col justify-center gap-6 p-6">
    <header>
      <h1 class="text-4xl font-semibold tracking-tight text-teal-700">florg</h1>
      <p class="mt-1 text-slate-600">Fachlehrer Organizer</p>
    </header>

    <section class="rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
      <h2 class="text-lg font-medium">Anmelden</h2>

      <template v-if="state === 'mail'">
        <p class="mt-2 text-slate-700" data-testid="login-sent">
          Der Anmeldelink ist unterwegs an die hinterlegte E-Mail-Adresse.
        </p>
        <p class="mt-2 text-sm text-slate-500">
          Er gilt 15 Minuten. Öffne ihn in diesem Browser.
        </p>
      </template>

      <template v-else-if="state === 'log'">
        <p class="mt-2 text-amber-800" data-testid="login-log">
          Der Mailversand ist nicht eingerichtet. Der Anmeldelink steht im Server-Protokoll.
        </p>
        <p class="mt-2 text-sm text-slate-500">
          Er gilt 15 Minuten. Öffne ihn in diesem Browser.
        </p>
      </template>

      <template v-else>
        <p class="mt-2 text-slate-700">
          Du bekommst einen Anmeldelink an die hinterlegte E-Mail-Adresse.
        </p>
        <button
          type="button"
          class="mt-4 w-full rounded-lg bg-teal-700 px-4 py-3 font-medium text-white active:bg-teal-800 disabled:opacity-60"
          :disabled="state === 'sending'"
          @click="requestLink"
        >
          {{ state === 'sending' ? 'Wird angefordert …' : 'Anmeldelink anfordern' }}
        </button>
      </template>

      <p v-if="error" class="mt-3 text-sm text-red-700" role="alert" data-testid="login-error">
        {{ error }}
      </p>
    </section>
  </main>
</template>
