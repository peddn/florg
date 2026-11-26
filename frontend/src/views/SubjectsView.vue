<script setup>
import { onMounted, reactive, ref } from 'vue'

import ConfirmDialog from '../components/ConfirmDialog.vue'
import ModalDialog from '../components/ModalDialog.vue'
import { useSubjectsStore } from '../stores/masterData'

const store = useSubjectsStore()

const loadError = ref(null)
const busy = ref(false)
const error = ref(null)
// The open window, or null: { type: 'edit' | 'delete', subject }.
// `subject` is null while a new one is created.
const dialog = ref(null)
const form = reactive({ name: '', short_name: '' })

onMounted(async () => {
  const result = await store.load()
  loadError.value = result.message
})

function showForm(subject = null) {
  error.value = null
  form.name = subject?.name ?? ''
  form.short_name = subject?.short_name ?? ''
  dialog.value = { type: 'edit', subject }
}

function showDelete(subject) {
  error.value = null
  dialog.value = { type: 'delete', subject }
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

function save() {
  const subject = dialog.value.subject
  return run(subject ? store.update(subject.id, { ...form }) : store.create({ ...form }))
}

function remove() {
  return run(store.remove(dialog.value.subject.id))
}
</script>

<template>
  <main class="mx-auto max-w-6xl p-4">
    <div class="flex items-center justify-between gap-3">
      <h1 class="text-2xl font-semibold">Fächer</h1>
      <button
        type="button"
        class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800"
        @click="showForm()"
      >
        Fach anlegen
      </button>
    </div>

    <p v-if="loadError" class="mt-4 text-red-700" role="alert">{{ loadError }}</p>
    <p v-else-if="store.list === null" class="mt-4 text-slate-500">Wird geladen …</p>
    <p v-else-if="store.list.length === 0" class="mt-4 text-slate-600">Es gibt noch kein Fach.</p>
    <div v-else class="mt-4 overflow-x-auto rounded-xl border border-slate-200 bg-white shadow-sm">
      <table class="w-full text-left">
        <thead class="border-b border-slate-200 text-sm text-slate-500">
          <tr>
            <th scope="col" class="px-4 py-2 font-medium">Name</th>
            <th scope="col" class="px-4 py-2 font-medium">Kürzel</th>
            <th scope="col" class="px-4 py-2"><span class="sr-only">Aktionen</span></th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-200">
          <tr v-for="subject in store.list" :key="subject.id">
            <td class="px-4 py-3 font-medium">{{ subject.name }}</td>
            <td class="px-4 py-3">{{ subject.short_name }}</td>
            <td class="px-4 py-3 text-right whitespace-nowrap">
              <button
                type="button"
                class="rounded-lg px-2 py-1 text-sm font-medium text-teal-800 hover:bg-teal-50"
                :aria-label="`${subject.name} ändern`"
                @click="showForm(subject)"
              >
                Ändern
              </button>
              <button
                type="button"
                class="rounded-lg px-2 py-1 text-sm font-medium text-red-700 hover:bg-red-50"
                :aria-label="`${subject.name} löschen`"
                @click="showDelete(subject)"
              >
                Löschen
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <ModalDialog
      :open="dialog?.type === 'edit'"
      :title="dialog?.subject ? 'Fach ändern' : 'Neues Fach'"
      @update:open="setOpen"
    >
      <form class="grid gap-3" @submit.prevent="save">
        <label class="block text-sm font-medium text-slate-700">
          Name
          <input
            v-model.trim="form.name"
            type="text"
            required
            maxlength="100"
            placeholder="z. B. Technik"
            class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
          />
        </label>
        <label class="block text-sm font-medium text-slate-700">
          Kürzel
          <input
            v-model.trim="form.short_name"
            type="text"
            required
            maxlength="10"
            placeholder="z. B. Te"
            class="mt-1 w-40 rounded-lg border border-slate-300 px-3 py-2 text-base"
          />
          <span class="mt-1 block text-xs font-normal text-slate-500">
            Höchstens 10 Zeichen, für enge Darstellungen wie den Stundenplan.
          </span>
        </label>
        <p v-if="error" class="text-sm text-red-700" role="alert">{{ error }}</p>
        <div class="mt-2 flex justify-end gap-2">
          <button
            type="button"
            class="rounded-lg px-4 py-2 font-medium text-slate-600 hover:bg-slate-100"
            @click="setOpen(false)"
          >
            Abbrechen
          </button>
          <button
            type="submit"
            class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800 disabled:opacity-60"
            :disabled="busy"
          >
            {{ dialog?.subject ? 'Speichern' : 'Anlegen' }}
          </button>
        </div>
      </form>
    </ModalDialog>

    <ConfirmDialog
      :open="dialog?.type === 'delete'"
      title="Fach löschen?"
      :message="`„${dialog?.subject?.name}“ wird gelöscht.`"
      :busy="busy"
      :error="error"
      @confirm="remove"
      @update:open="setOpen"
    />
  </main>
</template>
