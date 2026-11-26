<script setup>
import { ref } from 'vue'

import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()

const email = ref('')
// 'form' | 'sending' | 'mail' | 'log'
const state = ref('form')
const error = ref(null)

async function requestLink() {
  state.value = 'sending'
  error.value = null
  const result = await auth.requestLink(email.value)
  if (result.ok) {
    state.value = result.data.delivery === 'log' ? 'log' : 'mail'
  } else {
    state.value = 'form'
    error.value = result.message
  }
}

function startOver() {
  state.value = 'form'
}
</script>

<template>
  <main class="mx-auto flex min-h-[calc(100dvh-2rem)] max-w-md flex-col justify-center gap-6 p-6">
    <header>
      <h1 class="text-4xl font-semibold tracking-tight text-teal-700">florg</h1>
      <p class="mt-1 text-slate-600">Fachlehrer Organizer</p>
    </header>

    <section class="rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
      <h2 class="text-lg font-medium">Anmelden</h2>

      <form v-if="state === 'form' || state === 'sending'" @submit.prevent="requestLink">
        <p class="mt-2 text-slate-700">
          Gib die E-Mail-Adresse deines Kontos ein. Du bekommst einen Anmeldelink.
        </p>
        <label for="email" class="mt-4 block text-sm font-medium text-slate-700">
          E-Mail-Adresse
        </label>
        <input
          id="email"
          v-model.trim="email"
          type="email"
          name="email"
          required
          autocomplete="email"
          inputmode="email"
          autocapitalize="none"
          spellcheck="false"
          class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-3 text-base focus:border-teal-700 focus:outline-2 focus:outline-teal-700"
        />
        <button
          type="submit"
          class="mt-4 w-full rounded-lg bg-teal-700 px-4 py-3 font-medium text-white active:bg-teal-800 disabled:opacity-60"
          :disabled="state === 'sending'"
        >
          {{ state === 'sending' ? 'Wird angefordert …' : 'Anmeldelink anfordern' }}
        </button>
        <p v-if="error" class="mt-3 text-sm text-red-700" role="alert" data-testid="login-error">
          {{ error }}
        </p>
      </form>

      <div v-else role="status">
        <p v-if="state === 'mail'" class="mt-2 text-slate-700" data-testid="login-sent">
          Falls die Adresse zum Konto gehört, ist ein Anmeldelink unterwegs.
        </p>
        <p v-else class="mt-2 text-amber-800" data-testid="login-log">
          Der Mailversand ist nicht eingerichtet. Falls die Adresse zum Konto gehört, steht der
          Anmeldelink im Server-Protokoll.
        </p>
        <p class="mt-2 text-sm text-slate-500">
          Der Link gilt 15 Minuten. Öffne ihn in diesem Browser.
        </p>
        <p v-if="state === 'mail'" class="mt-2 text-sm text-slate-500">
          Keine Mail bekommen? Prüfe die Adresse und den Spam-Ordner. Es sind höchstens 3 Links in
          15 Minuten möglich.
        </p>
        <button
          type="button"
          class="mt-4 w-full px-4 py-2 text-sm text-slate-500 underline underline-offset-2"
          @click="startOver"
        >
          Adresse erneut eingeben
        </button>
      </div>
    </section>
  </main>
</template>
