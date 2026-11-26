<script setup>
import { reactive } from 'vue'

import BusyButton from './BusyButton.vue'

const props = defineProps({
  // The learning group to change, or null for a new one.
  group: { type: Object, default: null },
  // All subjects, for the selection.
  subjects: { type: Array, required: true },
  busy: { type: Boolean, default: false },
  error: { type: String, default: null },
})
const emit = defineEmits(['submit', 'cancel'])

const form = reactive({
  name: props.group?.name ?? '',
  // With a single subject there is nothing to choose.
  subject_id: props.group?.subject_id ?? (props.subjects.length === 1 ? props.subjects[0].id : null),
})
</script>

<template>
  <p v-if="subjects.length === 0" class="text-slate-700">
    Eine Lerngruppe braucht ein Fach. Lege zuerst unter
    <RouterLink :to="{ name: 'subjects' }" class="text-teal-800 underline">Fächer</RouterLink>
    ein Fach an.
  </p>
  <form v-else class="grid gap-3" @submit.prevent="emit('submit', { ...form })">
    <label class="block text-sm font-medium text-slate-700">
      Name
      <input
        v-model.trim="form.name"
        type="text"
        required
        maxlength="100"
        placeholder="z. B. 7a/b Technik Gruppe 1"
        class="mt-1 block w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
      />
    </label>
    <label class="block text-sm font-medium text-slate-700">
      Fach
      <select
        v-model="form.subject_id"
        required
        class="mt-1 block w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-base"
      >
        <option :value="null" disabled>Bitte wählen</option>
        <option v-for="subject in subjects" :key="subject.id" :value="subject.id">
          {{ subject.name }}
        </option>
      </select>
    </label>
    <p v-if="error" class="text-sm text-red-700" role="alert">{{ error }}</p>
    <div class="mt-2 flex justify-end gap-2">
      <button
        type="button"
        class="rounded-lg px-4 py-2 font-medium text-slate-600 hover:bg-slate-100"
        @click="emit('cancel')"
      >
        Abbrechen
      </button>
      <BusyButton
        type="submit"
        class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800"
        :busy="busy"
      >
        {{ group ? 'Speichern' : 'Anlegen' }}
      </BusyButton>
    </div>
  </form>
</template>
