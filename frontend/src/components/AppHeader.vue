<script setup>
import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { useAuthStore } from '../stores/auth'
import { useSchoolYearsStore } from '../stores/schoolYears'
import BusyButton from './BusyButton.vue'

const auth = useAuthStore()
const schoolYears = useSchoolYearsStore()
const route = useRoute()
const router = useRouter()

// First row: pages that do not depend on a school year.
const generalLinks = [
  { name: 'schoolYears', label: 'Schuljahre' },
  { name: 'subjects', label: 'Fächer' },
  { name: 'schoolClasses', label: 'Klassen' },
]

// Second row: pages of the active school year. Their routes carry its id
// as `yearId`.
const yearLinks = [
  { name: 'students', label: 'Schüler' },
  { name: 'schoolYear', label: 'Unterrichtsfreie Zeiten' },
]

const active = computed(() => schoolYears.active)

const linkClass = 'rounded-lg px-3 py-2 text-sm font-medium whitespace-nowrap'
const activeClass = 'bg-teal-50 text-teal-800'
const idleClass = 'text-slate-600 hover:bg-slate-100'

function stateClass(name) {
  return route.name === name ? activeClass : idleClass
}

onMounted(() => {
  if (schoolYears.list === null) {
    schoolYears.loadList()
  }
})

// Opening a page of a school year makes that school year the active one.
// If the address names a school year that no longer exists, e.g. an old
// bookmark, the same page opens for the active school year instead.
watch(
  [() => route.params.yearId, () => schoolYears.list],
  ([yearId, list]) => {
    if (!yearId) {
      return
    }
    const id = Number(yearId)
    if (list === null || list.some((year) => year.id === id)) {
      schoolYears.setActive(id)
      return
    }
    schoolYears.ensureActive()
    if (schoolYears.active) {
      router.replace({ name: route.name, params: { yearId: schoolYears.active.id } })
    } else {
      router.replace({ name: 'schoolYears' })
    }
  },
  { immediate: true },
)

// Choosing another school year stays on the same kind of page.
function chooseYear(event) {
  const id = Number(event.target.value)
  schoolYears.setActive(id)
  if (route.params.yearId) {
    router.push({ name: route.name, params: { yearId: id } })
  }
}

const error = ref(null)
const loggingOut = ref(false)

// Signs out this device only (rule L9).
async function logout() {
  error.value = null
  loggingOut.value = true
  const result = await auth.logout()
  loggingOut.value = false
  if (auth.isSignedIn) {
    error.value = result.message
  } else {
    await router.replace({ name: 'login' })
  }
}
</script>

<template>
  <header class="border-b border-slate-200 bg-white">
    <nav
      class="mx-auto flex max-w-6xl items-center gap-1 overflow-x-auto px-4 py-2"
      aria-label="Hauptnavigation"
    >
      <RouterLink
        :to="{ name: 'home' }"
        class="mr-3 text-xl font-semibold tracking-tight text-teal-700"
      >
        florg
      </RouterLink>
      <RouterLink
        v-for="link in generalLinks"
        :key="link.name"
        :to="{ name: link.name }"
        :class="[linkClass, stateClass(link.name)]"
        :aria-current="route.name === link.name ? 'page' : undefined"
      >
        {{ link.label }}
      </RouterLink>

      <span class="grow" />

      <RouterLink :to="{ name: 'account' }" :class="[linkClass, stateClass('account')]">
        Konto
      </RouterLink>
      <BusyButton
        class="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium whitespace-nowrap text-slate-700 hover:bg-slate-100"
        :busy="loggingOut"
        @click="logout"
      >
        Abmelden
      </BusyButton>
    </nav>
    <p v-if="error" class="mx-auto max-w-6xl px-4 pb-2 text-right text-sm text-red-700" role="alert">
      {{ error }}
    </p>

    <!-- Everything in this row refers to the active school year. -->
    <nav
      v-if="active"
      class="border-t border-slate-200 bg-slate-50"
      aria-label="Aktives Schuljahr"
    >
      <div class="mx-auto flex max-w-6xl items-center gap-1 overflow-x-auto px-4 py-1.5">
        <label class="mr-2 flex items-center gap-2 text-sm font-medium text-slate-600">
          <span class="whitespace-nowrap">Schuljahr</span>
          <select
            :value="active.id"
            class="rounded-lg border border-slate-300 bg-white px-2 py-1.5 text-sm font-semibold text-slate-900"
            @change="chooseYear"
          >
            <option v-for="year in schoolYears.list" :key="year.id" :value="year.id">
              {{ year.name }}
            </option>
          </select>
        </label>
        <RouterLink
          v-for="link in yearLinks"
          :key="link.name"
          :to="{ name: link.name, params: { yearId: active.id } }"
          :class="[linkClass, route.name === link.name ? 'bg-white text-teal-800 shadow-sm' : idleClass]"
          :aria-current="route.name === link.name ? 'page' : undefined"
        >
          {{ link.label }}
        </RouterLink>
      </div>
    </nav>
  </header>
</template>
