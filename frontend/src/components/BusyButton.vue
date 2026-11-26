<script setup>
// A button that triggers a request to the server. While the request runs
// it is locked, and after a short delay it shows a spinner (concept,
// "Bedienung"). Classes and other attributes go to the button itself.
import { useDelayed } from '../delayed'
import BusySpinner from './BusySpinner.vue'

const props = defineProps({
  // Whether the request this button triggered is still running.
  busy: { type: Boolean, default: false },
  // Locked for another reason, e.g. an incomplete form.
  disabled: { type: Boolean, default: false },
  type: { type: String, default: 'button' },
})

const showSpinner = useDelayed(() => props.busy)
</script>

<template>
  <button
    :type="type"
    :disabled="busy || disabled"
    :aria-busy="busy"
    class="inline-flex items-center justify-center gap-2 disabled:opacity-60"
  >
    <BusySpinner v-if="showSpinner" />
    <slot />
  </button>
</template>
