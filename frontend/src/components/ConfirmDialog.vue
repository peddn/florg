<script setup>
// Asks before something is deleted. Behaviour comes from Reka UI: unlike a
// form window, it does not close on a click next to it.
import {
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogOverlay,
  AlertDialogPortal,
  AlertDialogRoot,
  AlertDialogTitle,
} from 'reka-ui'

defineProps({
  title: { type: String, required: true },
  message: { type: String, required: true },
  confirmLabel: { type: String, default: 'Löschen' },
  busy: { type: Boolean, default: false },
  error: { type: String, default: null },
})
// The parent closes the window once the action has succeeded.
const emit = defineEmits(['confirm'])

const open = defineModel('open', { type: Boolean, required: true })
</script>

<template>
  <AlertDialogRoot v-model:open="open">
    <AlertDialogPortal>
      <AlertDialogOverlay class="fixed inset-0 z-20 bg-slate-900/40" />
      <AlertDialogContent
        class="fixed top-1/2 left-1/2 z-30 w-[calc(100vw-2rem)] max-w-md -translate-x-1/2 -translate-y-1/2 rounded-xl bg-white p-5 text-slate-900 shadow-xl"
      >
        <AlertDialogTitle class="text-lg font-semibold">{{ title }}</AlertDialogTitle>
        <AlertDialogDescription class="mt-2 text-slate-600">{{ message }}</AlertDialogDescription>
        <p v-if="error" class="mt-3 text-sm text-red-700" role="alert">{{ error }}</p>
        <div class="mt-5 flex justify-end gap-2">
          <AlertDialogCancel
            class="rounded-lg px-4 py-2 font-medium text-slate-600 hover:bg-slate-100"
          >
            Abbrechen
          </AlertDialogCancel>
          <button
            type="button"
            class="rounded-lg bg-red-700 px-4 py-2 font-medium text-white hover:bg-red-800 disabled:opacity-60"
            :disabled="busy"
            @click="emit('confirm')"
          >
            {{ confirmLabel }}
          </button>
        </div>
      </AlertDialogContent>
    </AlertDialogPortal>
  </AlertDialogRoot>
</template>
