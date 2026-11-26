<script setup>
import { computed, reactive, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import BusyButton from '../components/BusyButton.vue'
import ConfirmDialog from '../components/ConfirmDialog.vue'
import LearningGroupForm from '../components/LearningGroupForm.vue'
import MemberPicker from '../components/MemberPicker.vue'
import ModalDialog from '../components/ModalDialog.vue'
import { formatDate, fullName } from '../format'
import { useLearningGroupsStore } from '../stores/learningGroups'
import { useSubjectsStore } from '../stores/masterData'
import { useSchoolYearsStore } from '../stores/schoolYears'
import { useStudentsStore } from '../stores/students'

const store = useLearningGroupsStore()
const students = useStudentsStore()
const subjects = useSubjectsStore()
const schoolYears = useSchoolYearsStore()
const route = useRoute()
const router = useRouter()

const group = computed(() => store.current)
const year = computed(() => schoolYears.active)
const loadError = ref(null)
const busy = ref(false)
const error = ref(null)

// The open window, or null: { type, member? } with the types 'edit',
// 'delete', 'add', 'period', 'rejoin' and 'remove'. `member` is one row,
// i.e. one period of a student.
const dialog = ref(null)
const period = reactive({ joined_on: '', left_on: '' })

// Students of the school year who have never been in the group.
const candidates = computed(() => {
  const members = new Set((group.value?.members ?? []).map((member) => member.student_id))
  return (students.list ?? []).filter((student) => !members.has(student.id))
})

// A student with several periods counts once.
const studentCount = computed(
  () => new Set((group.value?.members ?? []).map((member) => member.student_id)).size,
)

// Number of periods per student, to word the questions.
const periodsOf = computed(() => {
  const counts = new Map()
  for (const member of group.value?.members ?? []) {
    counts.set(member.student_id, (counts.get(member.student_id) ?? 0) + 1)
  }
  return counts
})

// A student can return if their last period has ended. Rows of a student
// are sorted by date, so the last row is the latest period.
function canRejoin(member) {
  const rows = group.value.members.filter((row) => row.student_id === member.student_id)
  return member.left_on !== null && rows[rows.length - 1].id === member.id
}

// The day after a date, as an ISO date.
function dayAfter(iso) {
  const date = new Date(`${iso}T00:00:00Z`)
  date.setUTCDate(date.getUTCDate() + 1)
  return date.toISOString().slice(0, 10)
}

const listRoute = computed(() => ({
  name: 'learningGroups',
  params: { yearId: route.params.yearId },
}))

watch(
  () => route.params.groupId,
  async (groupId) => {
    if (!groupId) {
      return
    }
    dialog.value = null
    const result = await store.loadCurrent(groupId)
    loadError.value = result.message
    // An address that names the wrong school year is corrected.
    if (result.ok && String(result.data.school_year_id) !== String(route.params.yearId)) {
      await router.replace({
        name: 'learningGroup',
        params: { yearId: result.data.school_year_id, groupId },
      })
    }
  },
  { immediate: true },
)

function show(type, member = null) {
  error.value = null
  dialog.value = { type, member }
  if (type === 'period') {
    period.joined_on = member.joined_on ?? ''
    period.left_on = member.left_on ?? ''
  }
  if (type === 'rejoin') {
    period.joined_on = ''
    period.left_on = ''
  }
  if (type === 'edit') {
    subjects.load()
  }
  if (type === 'add') {
    students.load(group.value.school_year_id)
  }
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
  return result
}

function save(input) {
  return run(store.update(group.value.id, input))
}

async function remove() {
  const result = await run(store.remove(group.value.id))
  if (result.ok) {
    await router.replace(listRoute.value)
  }
}

function addMembers(studentIds) {
  return run(store.addMembers(group.value.id, studentIds))
}

function currentPeriod() {
  return { joined_on: period.joined_on || null, left_on: period.left_on || null }
}

function savePeriod() {
  return run(store.setPeriod(dialog.value.member.id, currentPeriod()))
}

function rejoin() {
  return run(store.addPeriod(group.value.id, dialog.value.member.student_id, currentPeriod()))
}

function removeMember() {
  return run(store.removePeriod(group.value.id, dialog.value.member.id))
}

// Wording of the question before removing a row.
function removeMessage(member) {
  if (!member) {
    return ''
  }
  const name = fullName(member)
  if (periodsOf.value.get(member.student_id) > 1) {
    const text = periodText(member) || 'ganzes Schuljahr'
    return `Dieser Zeitraum von „${name}“ (${text}) wird entfernt. Die anderen Zeiträume bleiben.`
  }
  return `„${name}“ wird aus der Lerngruppe entfernt und bleibt im Schuljahr erhalten.`
}

// "seit …", "bis …" or both. Empty for the whole school year.
function periodText(member) {
  const parts = []
  if (member.joined_on) {
    parts.push(`seit ${formatDate(member.joined_on)}`)
  }
  if (member.left_on) {
    parts.push(`bis ${formatDate(member.left_on)}`)
  }
  return parts.join(', ')
}
</script>

<template>
  <main class="mx-auto max-w-6xl p-4">
    <RouterLink :to="listRoute" class="text-sm text-teal-800 underline">
      Alle Lerngruppen
    </RouterLink>

    <p v-if="loadError" class="mt-4 text-red-700" role="alert">{{ loadError }}</p>
    <p v-else-if="!group" class="mt-4 text-slate-500">Wird geladen …</p>

    <template v-else>
      <section class="mt-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm">
        <h1 class="text-2xl font-semibold">{{ group.name }}</h1>
        <p class="mt-1 text-slate-600">
          {{ group.subject_name }}<template v-if="year"> · Schuljahr {{ year.name }}</template>
        </p>
        <div class="mt-3 flex gap-2">
          <button
            type="button"
            class="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium hover:bg-slate-100"
            @click="show('edit')"
          >
            Ändern
          </button>
          <button
            type="button"
            class="rounded-lg px-3 py-2 text-sm font-medium text-red-700 hover:bg-red-50"
            @click="show('delete')"
          >
            Löschen
          </button>
        </div>
      </section>

      <div class="mt-6 flex items-center justify-between gap-3">
        <h2 class="text-xl font-semibold" data-testid="member-count">
          {{ studentCount }} Schüler
        </h2>
        <button
          type="button"
          class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800"
          @click="show('add')"
        >
          Schüler hinzufügen
        </button>
      </div>

      <p v-if="group.members.length === 0" class="mt-3 text-slate-600">
        Die Lerngruppe hat noch keine Schüler.
      </p>
      <div v-else class="mt-3 relative overflow-x-auto rounded-xl border border-slate-200 bg-white shadow-sm">
        <table class="w-full text-left">
          <thead class="border-b border-slate-200 text-sm text-slate-500">
            <tr>
              <th scope="col" class="px-4 py-2 font-medium">Nachname</th>
              <th scope="col" class="px-4 py-2 font-medium">Vorname</th>
              <th scope="col" class="px-4 py-2 font-medium">Klasse</th>
              <th scope="col" class="px-4 py-2 font-medium">Zeitraum</th>
              <th scope="col" class="px-4 py-2"><span class="sr-only">Aktionen</span></th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-200">
            <tr v-for="member in group.members" :key="member.id">
              <td class="px-4 py-2.5 font-medium">{{ member.last_name }}</td>
              <td class="px-4 py-2.5" :class="{ 'font-medium': !member.last_name }">
                {{ member.first_name }}
              </td>
              <td class="px-4 py-2.5">{{ member.school_class_name }}</td>
              <td class="px-4 py-2.5 text-sm text-slate-600">{{ periodText(member) }}</td>
              <td class="px-4 py-2.5 text-right whitespace-nowrap">
                <button
                  v-if="canRejoin(member)"
                  type="button"
                  class="rounded-lg px-2 py-1 text-sm font-medium text-teal-800 hover:bg-teal-50"
                  :aria-label="`${fullName(member)} wieder aufnehmen`"
                  @click="show('rejoin', member)"
                >
                  Wieder aufnehmen
                </button>
                <button
                  type="button"
                  class="rounded-lg px-2 py-1 text-sm font-medium text-teal-800 hover:bg-teal-50"
                  :aria-label="`Zeitraum von ${fullName(member)} ändern`"
                  @click="show('period', member)"
                >
                  Zeitraum
                </button>
                <button
                  type="button"
                  class="rounded-lg px-2 py-1 text-sm font-medium text-red-700 hover:bg-red-50"
                  :aria-label="`${fullName(member)} entfernen`"
                  @click="show('remove', member)"
                >
                  Entfernen
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <ModalDialog
        :open="dialog?.type === 'edit'"
        title="Lerngruppe ändern"
        @update:open="setOpen"
      >
        <p v-if="subjects.list === null" class="text-slate-500">Wird geladen …</p>
        <LearningGroupForm
          v-else
          :group="group"
          :subjects="subjects.list"
          :busy="busy"
          :error="error"
          @submit="save"
          @cancel="setOpen(false)"
        />
      </ModalDialog>

      <ModalDialog
        :open="dialog?.type === 'add'"
        title="Schüler hinzufügen"
        :description="`Zur Auswahl stehen die Schüler des Schuljahres, die noch nicht in „${group.name}“ sind.`"
        wide
        @update:open="setOpen"
      >
        <p v-if="students.list === null" class="text-slate-500">Wird geladen …</p>
        <MemberPicker
          v-else
          :candidates="candidates"
          :busy="busy"
          :error="error"
          @submit="addMembers"
          @cancel="setOpen(false)"
        />
      </ModalDialog>

      <ModalDialog
        :open="dialog?.type === 'period'"
        :title="dialog?.member ? `Zeitraum von ${fullName(dialog.member)}` : ''"
        description="Für einen Wechsel während des Schuljahres. Leere Felder bedeuten: seit Beginn und bis zum Ende des Schuljahres."
        @update:open="setOpen"
      >
        <form class="grid gap-3" @submit.prevent="savePeriod">
          <div class="grid gap-3 sm:grid-cols-2">
            <label class="block text-sm font-medium text-slate-700">
              In der Gruppe seit
              <input
                v-model="period.joined_on"
                type="date"
                :min="year?.start_date"
                :max="period.left_on || year?.end_date"
                class="mt-1 block w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
              />
            </label>
            <label class="block text-sm font-medium text-slate-700">
              In der Gruppe bis
              <input
                v-model="period.left_on"
                type="date"
                :min="period.joined_on || year?.start_date"
                :max="year?.end_date"
                class="mt-1 block w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
              />
            </label>
          </div>
          <p v-if="error" class="text-sm text-red-700" role="alert">{{ error }}</p>
          <div class="mt-2 flex justify-end gap-2">
            <button
              type="button"
              class="rounded-lg px-4 py-2 font-medium text-slate-600 hover:bg-slate-100"
              @click="setOpen(false)"
            >
              Abbrechen
            </button>
            <BusyButton
              type="submit"
              class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800"
              :busy="busy"
            >
              Speichern
            </BusyButton>
          </div>
        </form>
      </ModalDialog>

      <ModalDialog
        :open="dialog?.type === 'rejoin'"
        :title="dialog?.type === 'rejoin' ? `${fullName(dialog.member)} wieder aufnehmen` : ''"
        :description="
          dialog?.type === 'rejoin'
            ? `Der Schüler hat die Gruppe am ${formatDate(dialog.member.left_on)} verlassen. Er bekommt einen neuen Zeitraum, der frühere bleibt erhalten.`
            : ''
        "
        @update:open="setOpen"
      >
        <form v-if="dialog?.type === 'rejoin'" class="grid gap-3" @submit.prevent="rejoin">
          <div class="grid gap-3 sm:grid-cols-2">
            <label class="block text-sm font-medium text-slate-700">
              Wieder in der Gruppe ab
              <input
                v-model="period.joined_on"
                type="date"
                required
                :min="dayAfter(dialog.member.left_on)"
                :max="period.left_on || year?.end_date"
                class="mt-1 block w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
              />
            </label>
            <label class="block text-sm font-medium text-slate-700">
              bis <span class="font-normal text-slate-500">(optional)</span>
              <input
                v-model="period.left_on"
                type="date"
                :min="period.joined_on || dayAfter(dialog.member.left_on)"
                :max="year?.end_date"
                class="mt-1 block w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
              />
            </label>
          </div>
          <p v-if="error" class="text-sm text-red-700" role="alert">{{ error }}</p>
          <div class="mt-2 flex justify-end gap-2">
            <button
              type="button"
              class="rounded-lg px-4 py-2 font-medium text-slate-600 hover:bg-slate-100"
              @click="setOpen(false)"
            >
              Abbrechen
            </button>
            <BusyButton
              type="submit"
              class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800"
              :busy="busy"
            >
              Aufnehmen
            </BusyButton>
          </div>
        </form>
      </ModalDialog>

      <ConfirmDialog
        :open="dialog?.type === 'delete'"
        title="Lerngruppe löschen?"
        :message="`„${group.name}“ wird gelöscht. Die Schüler bleiben im Schuljahr erhalten.`"
        :busy="busy"
        :error="error"
        @confirm="remove"
        @update:open="setOpen"
      />

      <ConfirmDialog
        :open="dialog?.type === 'remove'"
        :title="periodsOf.get(dialog?.member?.student_id) > 1 ? 'Zeitraum entfernen?' : 'Schüler entfernen?'"
        :message="removeMessage(dialog?.member)"
        confirm-label="Entfernen"
        :busy="busy"
        :error="error"
        @confirm="removeMember"
        @update:open="setOpen"
      />
    </template>
  </main>
</template>
