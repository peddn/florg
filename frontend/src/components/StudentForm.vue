<script setup>
import { reactive } from 'vue'

import { FORMS_OF_ADDRESS } from '../format'

const props = defineProps({
  // The student to change, or null for a new one.
  student: { type: Object, default: null },
  // All classes, for the selection.
  classes: { type: Array, required: true },
  busy: { type: Boolean, default: false },
  error: { type: String, default: null },
})
const emit = defineEmits(['submit', 'cancel'])

const form = reactive({
  first_name: props.student?.first_name ?? '',
  last_name: props.student?.last_name ?? '',
  form_of_address: props.student?.form_of_address ?? 'name_only',
  school_class_id: props.student?.school_class_id ?? null,
})

function submit() {
  emit('submit', { ...form, last_name: form.last_name || null })
}
</script>

<template>
  <form class="grid gap-3" @submit.prevent="submit">
    <div class="grid gap-3 sm:grid-cols-2">
      <label class="block text-sm font-medium text-slate-700">
        Vorname
        <input
          v-model.trim="form.first_name"
          type="text"
          required
          maxlength="100"
          class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
        />
      </label>
      <label class="block text-sm font-medium text-slate-700">
        Nachname <span class="font-normal text-slate-500">(optional)</span>
        <input
          v-model.trim="form.last_name"
          type="text"
          maxlength="100"
          class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
        />
      </label>
      <label class="block text-sm font-medium text-slate-700">
        Klasse
        <select
          v-model="form.school_class_id"
          class="mt-1 w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-base"
        >
          <option :value="null">ohne Klasse</option>
          <option v-for="schoolClass in classes" :key="schoolClass.id" :value="schoolClass.id">
            {{ schoolClass.name }}
          </option>
        </select>
      </label>
      <label class="block text-sm font-medium text-slate-700">
        Anrede
        <select
          v-model="form.form_of_address"
          required
          class="mt-1 w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-base"
        >
          <option v-for="entry in FORMS_OF_ADDRESS" :key="entry.value" :value="entry.value">
            {{ entry.label }}
          </option>
        </select>
      </label>
    </div>
    <p class="text-xs text-slate-500">
      Die Anrede bestimmt, ob Zeugnisbemerkungen „er“, „sie“ oder immer den Vornamen verwenden.
    </p>
    <p v-if="error" class="text-sm text-red-700" role="alert">{{ error }}</p>
    <div class="mt-2 flex justify-end gap-2">
      <button
        type="button"
        class="rounded-lg px-4 py-2 font-medium text-slate-600 hover:bg-slate-100"
        @click="emit('cancel')"
      >
        Abbrechen
      </button>
      <button
        type="submit"
        class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800 disabled:opacity-60"
        :disabled="busy"
      >
        {{ student ? 'Speichern' : 'Anlegen' }}
      </button>
    </div>
  </form>
</template>
