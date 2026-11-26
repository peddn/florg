<script setup>
import { reactive } from 'vue'

const props = defineProps({
  // { name, start_date, end_date } to start from.
  initial: { type: Object, required: true },
  submitLabel: { type: String, required: true },
  busy: { type: Boolean, default: false },
  error: { type: String, default: null },
})
const emit = defineEmits(['submit', 'cancel'])

const form = reactive({
  name: props.initial.name,
  start_date: props.initial.start_date,
  end_date: props.initial.end_date,
})

// Lets the parent fill in the values of a template.
defineExpose({
  fill({ name, start_date, end_date }) {
    Object.assign(form, { name, start_date, end_date })
  },
})
</script>

<template>
  <form class="grid gap-3" @submit.prevent="emit('submit', { ...form })">
    <label class="block text-sm font-medium text-slate-700">
      Name
      <input
        v-model.trim="form.name"
        type="text"
        required
        maxlength="100"
        placeholder="z. B. 2026/27"
        class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
      />
    </label>
    <div class="grid gap-3 sm:grid-cols-2">
      <label class="block text-sm font-medium text-slate-700">
        Erster Schultag
        <input
          v-model="form.start_date"
          type="date"
          required
          class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
        />
      </label>
      <label class="block text-sm font-medium text-slate-700">
        Letzter Schultag
        <input
          v-model="form.end_date"
          type="date"
          required
          :min="form.start_date"
          class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
        />
      </label>
    </div>
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
        {{ submitLabel }}
      </button>
    </div>
  </form>
</template>
