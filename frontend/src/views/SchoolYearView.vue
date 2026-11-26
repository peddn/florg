<script setup>
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import FreePeriodForm from '../components/FreePeriodForm.vue'
import SchoolYearForm from '../components/SchoolYearForm.vue'
import { formatRange, kindLabel } from '../format'
import { useSchoolYearsStore } from '../stores/schoolYears'

const store = useSchoolYearsStore()
const route = useRoute()
const router = useRouter()

const year = computed(() => store.current)
const loadError = ref(null)
const busy = ref(false)
const error = ref(null)
// What is being edited: null, 'year', 'new' or the id of a free period.
const editing = ref(null)

const kindStyles = {
  holiday: 'bg-teal-50 text-teal-800',
  public_holiday: 'bg-amber-50 text-amber-800',
  other: 'bg-slate-100 text-slate-700',
}

const newPeriod = { name: '', kind: 'other', start_date: '', end_date: '' }

watch(
  () => route.params.id,
  async (id) => {
    editing.value = null
    const result = await store.loadCurrent(id)
    loadError.value = result.message
  },
  { immediate: true },
)

function edit(what) {
  error.value = null
  editing.value = what
}

// Runs a change and closes the form if it worked.
async function run(request) {
  busy.value = true
  error.value = null
  const result = await request
  busy.value = false
  if (result.ok) {
    editing.value = null
  } else {
    error.value = result.message
  }
  return result
}

function saveYear(input) {
  return run(store.update(year.value.id, input))
}

async function removeYear() {
  const question = `Schuljahr ${year.value.name} mit allen unterrichtsfreien Zeiten löschen?`
  if (!window.confirm(question)) {
    return
  }
  const result = await run(store.remove(year.value.id))
  if (result.ok) {
    await router.replace({ name: 'schoolYears' })
  }
}

function addPeriod(input) {
  return run(store.addFreePeriod(year.value.id, input))
}

function savePeriod(period, input) {
  return run(store.updateFreePeriod(year.value.id, period.id, input))
}

function removePeriod(period) {
  if (window.confirm(`„${period.name}“ löschen?`)) {
    return run(store.removeFreePeriod(year.value.id, period.id))
  }
}
</script>

<template>
  <main class="mx-auto max-w-3xl p-4">
    <RouterLink :to="{ name: 'schoolYears' }" class="text-sm text-teal-800 underline">
      Alle Schuljahre
    </RouterLink>

    <p v-if="loadError" class="mt-4 text-red-700" role="alert">{{ loadError }}</p>
    <p v-else-if="!year" class="mt-4 text-slate-500">Wird geladen …</p>

    <template v-else>
      <section class="mt-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
        <template v-if="editing === 'year'">
          <h1 class="text-lg font-medium">Schuljahr ändern</h1>
          <SchoolYearForm
            class="mt-3"
            :initial="year"
            submit-label="Speichern"
            :busy="busy"
            :error="error"
            @submit="saveYear"
            @cancel="edit(null)"
          />
        </template>
        <template v-else>
          <h1 class="text-2xl font-semibold">Schuljahr {{ year.name }}</h1>
          <p class="mt-1 text-slate-600">{{ formatRange(year.start_date, year.end_date) }}</p>
          <p class="mt-2 text-lg" data-testid="school-days">
            <span class="font-semibold">{{ year.school_days }}</span> Schultage
          </p>
          <div class="mt-3 flex gap-2">
            <button
              type="button"
              class="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium"
              @click="edit('year')"
            >
              Ändern
            </button>
            <button
              type="button"
              class="rounded-lg px-3 py-2 text-sm font-medium text-red-700"
              @click="removeYear"
            >
              Löschen
            </button>
          </div>
          <p v-if="error && editing === null" class="mt-2 text-sm text-red-700" role="alert">
            {{ error }}
          </p>
        </template>
      </section>

      <div class="mt-6 flex items-center justify-between gap-3">
        <h2 class="text-xl font-semibold">Unterrichtsfreie Zeiten</h2>
        <button
          v-if="editing !== 'new'"
          type="button"
          class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white active:bg-teal-800"
          @click="edit('new')"
        >
          Hinzufügen
        </button>
      </div>

      <section
        v-if="editing === 'new'"
        class="mt-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm"
      >
        <h3 class="text-lg font-medium">Neue unterrichtsfreie Zeit</h3>
        <FreePeriodForm
          class="mt-3"
          :initial="newPeriod"
          :min="year.start_date"
          :max="year.end_date"
          submit-label="Hinzufügen"
          :busy="busy"
          :error="error"
          @submit="addPeriod"
          @cancel="edit(null)"
        />
      </section>

      <p v-if="year.free_periods.length === 0" class="mt-3 text-slate-600">
        Es sind keine unterrichtsfreien Zeiten eingetragen.
      </p>
      <ul v-else class="mt-3 divide-y divide-slate-200 rounded-xl border border-slate-200 bg-white shadow-sm">
        <li v-for="period in year.free_periods" :key="period.id" class="p-4">
          <FreePeriodForm
            v-if="editing === period.id"
            :initial="period"
            :min="year.start_date"
            :max="year.end_date"
            submit-label="Speichern"
            :busy="busy"
            :error="error"
            @submit="(input) => savePeriod(period, input)"
            @cancel="edit(null)"
          />
          <div v-else class="flex items-start justify-between gap-3">
            <div>
              <p class="font-medium">
                {{ period.name }}
                <span
                  class="ml-1 rounded-full px-2 py-0.5 text-xs font-medium"
                  :class="kindStyles[period.kind]"
                >
                  {{ kindLabel(period.kind) }}
                </span>
              </p>
              <p class="mt-1 text-sm text-slate-600">
                {{ formatRange(period.start_date, period.end_date) }}
              </p>
            </div>
            <div class="flex shrink-0 gap-1">
              <button
                type="button"
                class="rounded-lg px-2 py-1 text-sm font-medium text-teal-800"
                :aria-label="`${period.name} ändern`"
                @click="edit(period.id)"
              >
                Ändern
              </button>
              <button
                type="button"
                class="rounded-lg px-2 py-1 text-sm font-medium text-red-700"
                :aria-label="`${period.name} löschen`"
                @click="removePeriod(period)"
              >
                Löschen
              </button>
            </div>
          </div>
        </li>
      </ul>
    </template>
  </main>
</template>
