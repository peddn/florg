<script setup>
import { computed, ref } from 'vue'

import { fullName } from '../format'
import BusyButton from './BusyButton.vue'

const props = defineProps({
  // Students of the school year who are not members yet, in list order.
  candidates: { type: Array, required: true },
  busy: { type: Boolean, default: false },
  error: { type: String, default: null },
})
// Reports the ids of the chosen students.
const emit = defineEmits(['submit', 'cancel'])

// Filter: 'all', 'none' (without a class) or the id of a class.
const classFilter = ref('all')
const search = ref('')
const chosen = ref(new Set())

// Classes that occur among the candidates, in list order.
const classes = computed(() => {
  const seen = new Map()
  for (const student of props.candidates) {
    if (student.school_class_id !== null) {
      seen.set(student.school_class_id, student.school_class_name)
    }
  }
  return [...seen].map(([id, name]) => ({ id, name }))
})

const visible = computed(() => {
  const words = search.value.trim().toLowerCase()
  return props.candidates.filter((student) => {
    if (classFilter.value === 'none' && student.school_class_id !== null) {
      return false
    }
    if (typeof classFilter.value === 'number' && student.school_class_id !== classFilter.value) {
      return false
    }
    return !words || fullName(student).toLowerCase().includes(words)
  })
})

const allVisibleChosen = computed(
  () => visible.value.length > 0 && visible.value.every((student) => chosen.value.has(student.id)),
)

function toggle(id) {
  const next = new Set(chosen.value)
  if (!next.delete(id)) {
    next.add(id)
  }
  chosen.value = next
}

// Selects or clears everyone the filter currently shows.
function toggleVisible() {
  const next = new Set(chosen.value)
  const select = !allVisibleChosen.value
  for (const student of visible.value) {
    if (select) {
      next.add(student.id)
    } else {
      next.delete(student.id)
    }
  }
  chosen.value = next
}
</script>

<template>
  <p v-if="candidates.length === 0" class="text-slate-700">
    Alle Schüler dieses Schuljahres sind schon in der Lerngruppe.
  </p>
  <div v-else class="grid gap-3">
    <div class="flex flex-wrap items-end gap-3">
      <label class="block text-sm font-medium text-slate-700">
        Klasse
        <select
          v-model="classFilter"
          class="mt-1 block rounded-lg border border-slate-300 bg-white px-3 py-2 text-base"
        >
          <option value="all">alle</option>
          <option v-for="entry in classes" :key="entry.id" :value="entry.id">
            {{ entry.name }}
          </option>
          <option value="none">ohne Klasse</option>
        </select>
      </label>
      <label class="block text-sm font-medium text-slate-700">
        Suche
        <input
          v-model="search"
          type="search"
          placeholder="Name"
          class="mt-1 block w-48 rounded-lg border border-slate-300 px-3 py-2 text-base"
        />
      </label>
      <button
        type="button"
        class="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium hover:bg-slate-100 disabled:opacity-60"
        :disabled="visible.length === 0"
        @click="toggleVisible"
      >
        {{ allVisibleChosen ? 'Auswahl der Angezeigten aufheben' : 'Alle Angezeigten auswählen' }}
      </button>
    </div>

    <ul class="max-h-[45dvh] divide-y divide-slate-200 overflow-auto rounded-lg border border-slate-200">
      <li v-for="student in visible" :key="student.id">
        <label class="flex cursor-pointer items-center gap-3 px-3 py-2 hover:bg-slate-50">
          <input
            type="checkbox"
            class="size-4 accent-teal-700"
            :checked="chosen.has(student.id)"
            @change="toggle(student.id)"
          />
          <span class="grow">
            <span class="font-medium">{{ student.last_name }}</span>
            {{ student.first_name }}
          </span>
          <span class="text-sm text-slate-500">{{ student.school_class_name ?? 'ohne Klasse' }}</span>
        </label>
      </li>
      <li v-if="visible.length === 0" class="px-3 py-2 text-slate-600">
        Kein Schüler passt zum Filter.
      </li>
    </ul>

    <p v-if="error" class="text-sm text-red-700" role="alert">{{ error }}</p>
    <div class="mt-2 flex items-center justify-end gap-2">
      <span class="mr-auto text-sm text-slate-600" data-testid="chosen-count">
        {{ chosen.size }} ausgewählt
      </span>
      <button
        type="button"
        class="rounded-lg px-4 py-2 font-medium text-slate-600 hover:bg-slate-100"
        @click="emit('cancel')"
      >
        Abbrechen
      </button>
      <BusyButton
        class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800"
        :busy="busy"
        :disabled="chosen.size === 0"
        @click="emit('submit', [...chosen])"
      >
        Hinzufügen
      </BusyButton>
    </div>
  </div>
</template>
