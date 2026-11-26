<script setup>
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import SchoolYearForm from '../components/SchoolYearForm.vue'
import { formatRange } from '../format'
import { useSchoolYearsStore } from '../stores/schoolYears'

const store = useSchoolYearsStore()
const router = useRouter()

const loadError = ref(null)
const creating = ref(false)
const busy = ref(false)
const formError = ref(null)
const form = ref(null)

// Templates whose name is not taken yet.
const openTemplates = computed(() => {
  const taken = new Set((store.list ?? []).map((year) => year.name))
  return store.templates.filter((template) => !taken.has(template.name))
})

// The form starts with the earliest open template, if there is one.
const initial = computed(
  () => openTemplates.value[0] ?? { name: '', start_date: '', end_date: '' },
)

onMounted(async () => {
  const result = await store.loadList()
  loadError.value = result.message
})

function startCreating() {
  formError.value = null
  creating.value = true
}

async function create(input) {
  busy.value = true
  formError.value = null
  const result = await store.create(input)
  busy.value = false
  if (result.ok) {
    await router.push({ name: 'schoolYear', params: { id: result.data.id } })
  } else {
    formError.value = result.message
  }
}
</script>

<template>
  <main class="mx-auto max-w-6xl p-4">
    <div class="flex items-center justify-between gap-3">
      <h1 class="text-2xl font-semibold">Schuljahre</h1>
      <button
        v-if="!creating"
        type="button"
        class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white active:bg-teal-800"
        @click="startCreating"
      >
        Schuljahr anlegen
      </button>
    </div>

    <section
      v-if="creating"
      class="mt-4 max-w-2xl rounded-xl border border-slate-200 bg-white p-4 shadow-sm"
    >
      <h2 class="text-lg font-medium">Neues Schuljahr</h2>
      <p class="mt-1 text-sm text-slate-600">
        Ferien und Feiertage werden beim Anlegen eingetragen. Du kannst sie danach ändern.
      </p>
      <div v-if="openTemplates.length" class="mt-3">
        <p class="text-sm font-medium text-slate-700">Vorlage aus der amtlichen Ferienordnung</p>
        <div class="mt-1 flex flex-wrap gap-2">
          <button
            v-for="template in openTemplates"
            :key="template.name"
            type="button"
            class="rounded-full border border-teal-700 px-3 py-1 text-sm font-medium text-teal-800"
            @click="form.fill(template)"
          >
            {{ template.name }}
          </button>
        </div>
      </div>
      <SchoolYearForm
        ref="form"
        class="mt-4"
        :initial="initial"
        submit-label="Anlegen"
        :busy="busy"
        :error="formError"
        @submit="create"
        @cancel="creating = false"
      />
    </section>

    <p v-if="loadError" class="mt-4 text-red-700" role="alert">{{ loadError }}</p>
    <p v-else-if="store.list === null" class="mt-4 text-slate-500">Wird geladen …</p>
    <p v-else-if="store.list.length === 0 && !creating" class="mt-4 text-slate-600">
      Es gibt noch kein Schuljahr.
    </p>
    <ul v-else class="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
      <li v-for="year in store.list" :key="year.id">
        <RouterLink
          :to="{ name: 'schoolYear', params: { id: year.id } }"
          class="block rounded-xl border border-slate-200 bg-white p-4 shadow-sm hover:border-teal-600 active:bg-slate-50"
        >
          <span class="text-lg font-medium text-teal-800">{{ year.name }}</span>
          <span class="mt-1 block text-sm text-slate-600">
            {{ formatRange(year.start_date, year.end_date) }}
          </span>
        </RouterLink>
      </li>
    </ul>
  </main>
</template>
