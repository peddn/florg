<script setup>
import { computed, onBeforeUnmount, onMounted } from 'vue'

import { pendingChanges } from '../api'
import { useDelayed } from '../delayed'
import { useHealthStore } from '../stores/health'
import BusySpinner from './BusySpinner.vue'

// How often the connection is checked while the page is open.
const CHECK_INTERVAL_MS = 60_000

const health = useHealthStore()

// Shown while a change is on its way to the server.
const saving = useDelayed(() => pendingChanges.value > 0)

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

let timer = null

// A tab that comes back to the front shows a fresh state at once.
function checkWhenVisible() {
  if (document.visibilityState === 'visible') {
    health.check()
  }
}

onMounted(() => {
  health.check()
  timer = window.setInterval(checkWhenVisible, CHECK_INTERVAL_MS)
  document.addEventListener('visibilitychange', checkWhenVisible)
})

onBeforeUnmount(() => {
  window.clearInterval(timer)
  document.removeEventListener('visibilitychange', checkWhenVisible)
})
</script>

<template>
  <footer
    class="fixed inset-x-0 bottom-0 z-10 flex h-8 items-center justify-end gap-4 border-t border-slate-200 bg-white px-4 text-xs text-slate-600"
  >
    <span v-if="saving" class="flex items-center gap-2 text-teal-800" role="status">
      <BusySpinner />
      Speichert …
    </span>
    <button
      type="button"
      class="flex items-center gap-2"
      title="Verbindung erneut prüfen"
      @click="health.check()"
    >
      <span class="size-2.5 shrink-0 rounded-full" :class="color" aria-hidden="true" />
      <span data-testid="health-label">{{ label }}</span>
      <span v-if="health.version" class="text-slate-400">· Version {{ health.version }}</span>
    </button>
  </footer>
</template>
