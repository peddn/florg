<script setup>
// A modal window for forms. Behaviour (focus, Esc, screen readers) comes
// from Reka UI. Pages use this component instead of Reka UI directly.
import {
  DialogContent,
  DialogDescription,
  DialogOverlay,
  DialogPortal,
  DialogRoot,
  DialogTitle,
} from 'reka-ui'

defineProps({
  title: { type: String, required: true },
  // Optional sentence below the title.
  description: { type: String, default: null },
})

// Whether the window is open. The content only exists while it is, so a
// form inside starts fresh every time.
const open = defineModel('open', { type: Boolean, required: true })
</script>

<template>
  <DialogRoot v-model:open="open">
    <DialogPortal>
      <DialogOverlay class="fixed inset-0 z-20 bg-slate-900/40" />
      <DialogContent
        class="fixed top-1/2 left-1/2 z-30 max-h-[calc(100dvh-2rem)] w-[calc(100vw-2rem)] max-w-xl -translate-x-1/2 -translate-y-1/2 overflow-y-auto rounded-xl bg-white p-5 text-slate-900 shadow-xl"
        :aria-describedby="description ? undefined : ''"
      >
        <DialogTitle class="text-lg font-semibold">{{ title }}</DialogTitle>
        <DialogDescription v-if="description" class="mt-1 text-sm text-slate-600">
          {{ description }}
        </DialogDescription>
        <div class="mt-4">
          <slot />
        </div>
      </DialogContent>
    </DialogPortal>
  </DialogRoot>
</template>
