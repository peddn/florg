<script setup>
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import LearningGroupForm from '../components/LearningGroupForm.vue'
import ModalDialog from '../components/ModalDialog.vue'
import { useLearningGroupsStore } from '../stores/learningGroups'
import { useSubjectsStore } from '../stores/masterData'
import { useSchoolYearsStore } from '../stores/schoolYears'

const store = useLearningGroupsStore()
const subjects = useSubjectsStore()
const schoolYears = useSchoolYearsStore()
const route = useRoute()
const router = useRouter()

const loadError = ref(null)
const creating = ref(false)
const busy = ref(false)
const error = ref(null)

const year = computed(() => schoolYears.active)

watch(
  () => route.params.yearId,
  async (yearId) => {
    if (!yearId) {
      return
    }
    creating.value = false
    const [groups] = await Promise.all([store.load(Number(yearId)), subjects.load()])
    loadError.value = groups.message
  },
  { immediate: true },
)

function startCreating() {
  error.value = null
  creating.value = true
}

// A new learning group opens right away, so students can be added.
async function create(input) {
  busy.value = true
  error.value = null
  const result = await store.create(input)
  busy.value = false
  if (result.ok) {
    await router.push({
      name: 'learningGroup',
      params: { yearId: route.params.yearId, groupId: result.data.id },
    })
  } else {
    error.value = result.message
  }
}
</script>

<template>
  <main class="mx-auto max-w-6xl p-4">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div>
        <h1 class="text-2xl font-semibold">Lerngruppen</h1>
        <p v-if="year && store.list" class="text-sm text-slate-600" data-testid="group-count">
          Schuljahr {{ year.name }} · {{ store.list.length }} Lerngruppen
        </p>
      </div>
      <button
        type="button"
        class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800"
        @click="startCreating"
      >
        Lerngruppe anlegen
      </button>
    </div>

    <p v-if="loadError" class="mt-4 text-red-700" role="alert">{{ loadError }}</p>
    <p v-else-if="store.list === null" class="mt-4 text-slate-500">Wird geladen …</p>
    <p v-else-if="store.list.length === 0" class="mt-4 text-slate-600">
      In diesem Schuljahr gibt es noch keine Lerngruppen.
    </p>
    <div v-else class="mt-4 relative overflow-x-auto rounded-xl border border-slate-200 bg-white shadow-sm">
      <table class="w-full text-left">
        <thead class="border-b border-slate-200 text-sm text-slate-500">
          <tr>
            <th scope="col" class="px-4 py-2 font-medium">Name</th>
            <th scope="col" class="px-4 py-2 font-medium">Fach</th>
            <th scope="col" class="px-4 py-2 text-right font-medium">Schüler</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-200">
          <tr v-for="group in store.list" :key="group.id" class="hover:bg-slate-50">
            <td class="px-4 py-3">
              <RouterLink
                :to="{
                  name: 'learningGroup',
                  params: { yearId: route.params.yearId, groupId: group.id },
                }"
                class="font-medium text-teal-800 underline-offset-2 hover:underline"
              >
                {{ group.name }}
              </RouterLink>
            </td>
            <td class="px-4 py-3">{{ group.subject_name }}</td>
            <td class="px-4 py-3 text-right tabular-nums">{{ group.member_count }}</td>
          </tr>
        </tbody>
      </table>
    </div>

    <ModalDialog v-model:open="creating" title="Neue Lerngruppe">
      <LearningGroupForm
        :subjects="subjects.list ?? []"
        :busy="busy"
        :error="error"
        @submit="create"
        @cancel="creating = false"
      />
    </ModalDialog>
  </main>
</template>
