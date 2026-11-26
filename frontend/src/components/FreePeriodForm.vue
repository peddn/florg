<script setup>
import BusyButton from './BusyButton.vue'
import { reactive } from 'vue'

import { FREE_PERIOD_KINDS } from '../format'

const props = defineProps({
  // { name, kind, start_date, end_date } to start from.
  initial: { type: Object, required: true },
  // First and last school day: the period must lie in between.
  min: { type: String, required: true },
  max: { type: String, required: true },
  submitLabel: { type: String, required: true },
  busy: { type: Boolean, default: false },
  error: { type: String, default: null },
})
const emit = defineEmits(['submit', 'cancel'])

const form = reactive({
  name: props.initial.name,
  kind: props.initial.kind,
  start_date: props.initial.start_date,
  end_date: props.initial.end_date,
})

// A single day is the common case: the end follows the start until it is
// set by hand.
function startChanged() {
  if (!form.end_date || form.end_date < form.start_date) {
    form.end_date = form.start_date
  }
}
</script>

<template>
  <form class="grid gap-3" @submit.prevent="emit('submit', { ...form })">
    <div class="grid gap-3 sm:grid-cols-2">
      <label class="block text-sm font-medium text-slate-700">
        Name
        <input
          v-model.trim="form.name"
          type="text"
          required
          maxlength="100"
          placeholder="z. B. Pädagogischer Tag"
          class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
        />
      </label>
      <label class="block text-sm font-medium text-slate-700">
        Art
        <select
          v-model="form.kind"
          required
          class="mt-1 w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-base"
        >
          <option v-for="kind in FREE_PERIOD_KINDS" :key="kind.value" :value="kind.value">
            {{ kind.label }}
          </option>
        </select>
      </label>
      <label class="block text-sm font-medium text-slate-700">
        Erster freier Tag
        <input
          v-model="form.start_date"
          type="date"
          required
          :min="min"
          :max="max"
          class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 text-base"
          @change="startChanged"
        />
      </label>
      <label class="block text-sm font-medium text-slate-700">
        Letzter freier Tag
        <input
          v-model="form.end_date"
          type="date"
          required
          :min="form.start_date || min"
          :max="max"
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
      <BusyButton
        type="submit"
        class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800"
        :busy="busy"
      >
        {{ submitLabel }}
      </BusyButton>
    </div>
  </form>
</template>
