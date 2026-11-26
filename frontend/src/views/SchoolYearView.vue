<script setup>
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import ConfirmDialog from '../components/ConfirmDialog.vue'
import FreePeriodForm from '../components/FreePeriodForm.vue'
import ModalDialog from '../components/ModalDialog.vue'
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

// The open window, or null: { type, period? } with the types 'editYear',
// 'deleteYear', 'newPeriod', 'editPeriod' and 'deletePeriod'.
const dialog = ref(null)

const kindStyles = {
  holiday: 'bg-teal-50 text-teal-800',
  public_holiday: 'bg-amber-50 text-amber-800',
  other: 'bg-slate-100 text-slate-700',
}

const newPeriod = { name: '', kind: 'other', start_date: '', end_date: '' }

// Columns of the table of free periods on wide screens.
const columns = 'md:grid-cols-[minmax(0,1fr)_7rem_minmax(0,24rem)_10rem]'

watch(
  () => route.params.id,
  async (id) => {
    dialog.value = null
    const result = await store.loadCurrent(id)
    loadError.value = result.message
  },
  { immediate: true },
)

function show(type, period = null) {
  error.value = null
  dialog.value = { type, period }
}

function close() {
  dialog.value = null
}

// Binds one window to `dialog`: open while its type is the current one.
function isOpen(type) {
  return dialog.value?.type === type
}

function setOpen(open) {
  if (!open) {
    close()
  }
}

// Runs a change and closes the window if it worked.
async function run(request) {
  busy.value = true
  error.value = null
  const result = await request
  busy.value = false
  if (result.ok) {
    close()
  } else {
    error.value = result.message
  }
  return result
}

function saveYear(input) {
  return run(store.update(year.value.id, input))
}

async function removeYear() {
  const result = await run(store.remove(year.value.id))
  if (result.ok) {
    await router.replace({ name: 'schoolYears' })
  }
}

function addPeriod(input) {
  return run(store.addFreePeriod(year.value.id, input))
}

function savePeriod(input) {
  return run(store.updateFreePeriod(year.value.id, dialog.value.period.id, input))
}

function removePeriod() {
  return run(store.removeFreePeriod(year.value.id, dialog.value.period.id))
}
</script>

<template>
  <main class="mx-auto max-w-6xl p-4">
    <RouterLink :to="{ name: 'schoolYears' }" class="text-sm text-teal-800 underline">
      Alle Schuljahre
    </RouterLink>

    <p v-if="loadError" class="mt-4 text-red-700" role="alert">{{ loadError }}</p>
    <p v-else-if="!year" class="mt-4 text-slate-500">Wird geladen …</p>

    <template v-else>
      <section class="mt-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
        <h1 class="text-2xl font-semibold">Schuljahr {{ year.name }}</h1>
        <p class="mt-1 text-slate-600">{{ formatRange(year.start_date, year.end_date) }}</p>
        <p class="mt-2 text-lg" data-testid="school-days">
          <span class="font-semibold">{{ year.school_days }}</span> Schultage
        </p>
        <div class="mt-3 flex gap-2">
          <button
            type="button"
            class="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium hover:bg-slate-100"
            @click="show('editYear')"
          >
            Ändern
          </button>
          <button
            type="button"
            class="rounded-lg px-3 py-2 text-sm font-medium text-red-700 hover:bg-red-50"
            @click="show('deleteYear')"
          >
            Löschen
          </button>
        </div>
      </section>

      <div class="mt-6 flex items-center justify-between gap-3">
        <h2 class="text-xl font-semibold">Unterrichtsfreie Zeiten</h2>
        <button
          type="button"
          class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800"
          @click="show('newPeriod')"
        >
          Hinzufügen
        </button>
      </div>

      <p v-if="year.free_periods.length === 0" class="mt-3 text-slate-600">
        Es sind keine unterrichtsfreien Zeiten eingetragen.
      </p>
      <!-- A table on wide screens, stacked entries on a phone. -->
      <div v-else class="mt-3 rounded-xl border border-slate-200 bg-white shadow-sm">
        <div
          class="hidden border-b border-slate-200 px-4 py-2 text-sm font-medium text-slate-500 md:grid md:gap-4"
          :class="columns"
          aria-hidden="true"
        >
          <span>Name</span>
          <span>Art</span>
          <span>Zeitraum</span>
          <span />
        </div>
        <ul class="divide-y divide-slate-200">
          <li v-for="period in year.free_periods" :key="period.id" class="px-4 py-3">
            <div class="grid gap-1 md:items-center md:gap-4" :class="columns">
              <span class="font-medium">{{ period.name }}</span>
              <span>
                <span
                  class="rounded-full px-2 py-0.5 text-xs font-medium"
                  :class="kindStyles[period.kind]"
                >
                  {{ kindLabel(period.kind) }}
                </span>
              </span>
              <span class="text-sm text-slate-600 md:text-base md:text-slate-900">
                {{ formatRange(period.start_date, period.end_date) }}
              </span>
              <span class="flex gap-1 md:justify-end">
                <button
                  type="button"
                  class="rounded-lg px-2 py-1 text-sm font-medium text-teal-800 hover:bg-teal-50"
                  :aria-label="`${period.name} ändern`"
                  @click="show('editPeriod', period)"
                >
                  Ändern
                </button>
                <button
                  type="button"
                  class="rounded-lg px-2 py-1 text-sm font-medium text-red-700 hover:bg-red-50"
                  :aria-label="`${period.name} löschen`"
                  @click="show('deletePeriod', period)"
                >
                  Löschen
                </button>
              </span>
            </div>
          </li>
        </ul>
      </div>

      <ModalDialog
        :open="isOpen('editYear')"
        title="Schuljahr ändern"
        @update:open="setOpen"
      >
        <SchoolYearForm
          :initial="year"
          submit-label="Speichern"
          :busy="busy"
          :error="error"
          @submit="saveYear"
          @cancel="close"
        />
      </ModalDialog>

      <ModalDialog
        :open="isOpen('newPeriod')"
        title="Neue unterrichtsfreie Zeit"
        @update:open="setOpen"
      >
        <FreePeriodForm
          :initial="newPeriod"
          :min="year.start_date"
          :max="year.end_date"
          submit-label="Hinzufügen"
          :busy="busy"
          :error="error"
          @submit="addPeriod"
          @cancel="close"
        />
      </ModalDialog>

      <ModalDialog
        :open="isOpen('editPeriod')"
        title="Unterrichtsfreie Zeit ändern"
        @update:open="setOpen"
      >
        <FreePeriodForm
          v-if="dialog?.period"
          :initial="dialog.period"
          :min="year.start_date"
          :max="year.end_date"
          submit-label="Speichern"
          :busy="busy"
          :error="error"
          @submit="savePeriod"
          @cancel="close"
        />
      </ModalDialog>

      <ConfirmDialog
        :open="isOpen('deleteYear')"
        title="Schuljahr löschen?"
        :message="`Das Schuljahr ${year.name} wird mit allen unterrichtsfreien Zeiten gelöscht.`"
        :busy="busy"
        :error="error"
        @confirm="removeYear"
        @update:open="setOpen"
      />

      <ConfirmDialog
        :open="isOpen('deletePeriod')"
        title="Unterrichtsfreie Zeit löschen?"
        :message="`„${dialog?.period?.name}“ wird gelöscht.`"
        :busy="busy"
        :error="error"
        @confirm="removePeriod"
        @update:open="setOpen"
      />
    </template>
  </main>
</template>
