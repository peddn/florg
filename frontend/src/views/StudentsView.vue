<script setup>
import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import ConfirmDialog from '../components/ConfirmDialog.vue'
import ModalDialog from '../components/ModalDialog.vue'
import StudentForm from '../components/StudentForm.vue'
import StudentImport from '../components/StudentImport.vue'
import { addressLabel, fullName } from '../format'
import { useSchoolClassesStore } from '../stores/masterData'
import { useSchoolYearsStore } from '../stores/schoolYears'
import { useStudentsStore } from '../stores/students'

const store = useStudentsStore()
const classes = useSchoolClassesStore()
const schoolYears = useSchoolYearsStore()
const route = useRoute()

const loadError = ref(null)
const busy = ref(false)
const error = ref(null)
// Shown after an import.
const notice = ref(null)
// The open window, or null: { type, student? } with the types 'edit'
// (student null for a new one), 'delete' and 'import'.
const dialog = ref(null)

// Filter: 'all', 'none' (without a class) or the id of a class.
const classFilter = ref('all')
const search = ref('')

const year = computed(() => schoolYears.active)

// Classes that occur among the students, in list order.
const usedClasses = computed(() => {
  const seen = new Map()
  for (const student of store.list ?? []) {
    if (student.school_class_id !== null) {
      seen.set(student.school_class_id, student.school_class_name)
    }
  }
  return [...seen].map(([id, name]) => ({ id, name }))
})

const visible = computed(() => {
  const words = search.value.trim().toLowerCase()
  return (store.list ?? []).filter((student) => {
    if (classFilter.value === 'none' && student.school_class_id !== null) {
      return false
    }
    if (typeof classFilter.value === 'number' && student.school_class_id !== classFilter.value) {
      return false
    }
    return !words || fullName(student).toLowerCase().includes(words)
  })
})

watch(
  () => route.params.yearId,
  async (yearId) => {
    if (!yearId) {
      return
    }
    dialog.value = null
    notice.value = null
    classFilter.value = 'all'
    search.value = ''
    const [students] = await Promise.all([store.load(Number(yearId)), classes.load()])
    loadError.value = students.message
  },
  { immediate: true },
)

function show(type, student = null) {
  error.value = null
  notice.value = null
  dialog.value = { type, student }
}

function setOpen(open) {
  if (!open) {
    dialog.value = null
  }
}

// Runs a change and closes the window if it worked.
async function run(request) {
  busy.value = true
  error.value = null
  const result = await request
  busy.value = false
  if (result.ok) {
    dialog.value = null
  } else {
    error.value = result.message
  }
}

function save(input) {
  const student = dialog.value.student
  return run(student ? store.update(student.id, input) : store.create(input))
}

function remove() {
  return run(store.remove(dialog.value.student.id))
}

async function imported(count) {
  dialog.value = null
  // The import may have created classes.
  await classes.load()
  notice.value = `${count} Schüler angelegt.`
}
</script>

<template>
  <main class="mx-auto max-w-6xl p-4">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h1 class="text-2xl font-semibold">Schüler</h1>
        <p v-if="year && store.list" class="text-sm text-slate-600" data-testid="student-count">
          Schuljahr {{ year.name }} · {{ store.list.length }} Schüler
        </p>
      </div>
      <div class="flex gap-2">
        <button
          type="button"
          class="rounded-lg border border-slate-300 px-4 py-2 font-medium hover:bg-slate-100"
          @click="show('import')"
        >
          CSV importieren
        </button>
        <button
          type="button"
          class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800"
          @click="show('edit')"
        >
          Schüler anlegen
        </button>
      </div>
    </div>

    <p v-if="notice" class="mt-3 rounded-lg bg-emerald-50 px-3 py-2 text-emerald-800" role="status">
      {{ notice }}
    </p>

    <p v-if="loadError" class="mt-4 text-red-700" role="alert">{{ loadError }}</p>
    <p v-else-if="store.list === null" class="mt-4 text-slate-500">Wird geladen …</p>
    <p v-else-if="store.list.length === 0" class="mt-4 text-slate-600">
      In diesem Schuljahr gibt es noch keine Schüler.
    </p>
    <template v-else>
      <div class="mt-4 flex flex-wrap items-end gap-3">
        <label class="block text-sm font-medium text-slate-700">
          Klasse
          <select
            v-model="classFilter"
            class="mt-1 block rounded-lg border border-slate-300 bg-white px-3 py-2 text-base"
          >
            <option value="all">alle</option>
            <option v-for="entry in usedClasses" :key="entry.id" :value="entry.id">
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
            class="mt-1 block w-56 rounded-lg border border-slate-300 px-3 py-2 text-base"
          />
        </label>
        <p v-if="visible.length !== store.list.length" class="pb-2 text-sm text-slate-600">
          {{ visible.length }} von {{ store.list.length }}
        </p>
      </div>

      <div class="mt-3 relative overflow-x-auto rounded-xl border border-slate-200 bg-white shadow-sm">
        <table class="w-full text-left">
          <thead class="border-b border-slate-200 text-sm text-slate-500">
            <tr>
              <th scope="col" class="px-4 py-2 font-medium">Nachname</th>
              <th scope="col" class="px-4 py-2 font-medium">Vorname</th>
              <th scope="col" class="px-4 py-2 font-medium">Klasse</th>
              <th scope="col" class="px-4 py-2 font-medium">Anrede</th>
              <th scope="col" class="px-4 py-2"><span class="sr-only">Aktionen</span></th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-200">
            <tr v-for="student in visible" :key="student.id">
              <td class="px-4 py-2.5 font-medium">{{ student.last_name }}</td>
              <td class="px-4 py-2.5" :class="{ 'font-medium': !student.last_name }">
                {{ student.first_name }}
              </td>
              <td class="px-4 py-2.5">{{ student.school_class_name }}</td>
              <td class="px-4 py-2.5">{{ addressLabel(student.form_of_address) }}</td>
              <td class="px-4 py-2.5 text-right whitespace-nowrap">
                <button
                  type="button"
                  class="rounded-lg px-2 py-1 text-sm font-medium text-teal-800 hover:bg-teal-50"
                  :aria-label="`${fullName(student)} ändern`"
                  @click="show('edit', student)"
                >
                  Ändern
                </button>
                <button
                  type="button"
                  class="rounded-lg px-2 py-1 text-sm font-medium text-red-700 hover:bg-red-50"
                  :aria-label="`${fullName(student)} entfernen`"
                  @click="show('delete', student)"
                >
                  Entfernen
                </button>
              </td>
            </tr>
            <tr v-if="visible.length === 0">
              <td colspan="5" class="px-4 py-3 text-slate-600">Kein Schüler passt zum Filter.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </template>

    <ModalDialog
      :open="dialog?.type === 'edit'"
      :title="dialog?.student ? 'Schüler ändern' : 'Neuer Schüler'"
      @update:open="setOpen"
    >
      <StudentForm
        :student="dialog?.student"
        :classes="classes.list ?? []"
        :busy="busy"
        :error="error"
        @submit="save"
        @cancel="setOpen(false)"
      />
    </ModalDialog>

    <ModalDialog
      :open="dialog?.type === 'import'"
      title="Schüler aus CSV importieren"
      wide
      @update:open="setOpen"
    >
      <StudentImport @imported="imported" @cancel="setOpen(false)" />
    </ModalDialog>

    <ConfirmDialog
      :open="dialog?.type === 'delete'"
      title="Schüler entfernen?"
      :message="`„${dialog?.student ? fullName(dialog.student) : ''}“ wird aus dem Schuljahr ${year?.name} und aus dessen Lerngruppen entfernt. Gehört der Schüler zu keinem anderen Schuljahr, wird er ganz gelöscht.`"
      confirm-label="Entfernen"
      :busy="busy"
      :error="error"
      @confirm="remove"
      @update:open="setOpen"
    />
  </main>
</template>
