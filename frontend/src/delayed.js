import { onBeforeUnmount, ref, watch } from 'vue'

// How long something must take before the interface mentions it. Below
// that, a spinner would only flicker.
export const BUSY_DELAY_MS = 150

// Returns a ref that turns true once `source()` has been true for `delay`
// milliseconds, and false as soon as it is false again.
export function useDelayed(source, delay = BUSY_DELAY_MS) {
  const delayed = ref(false)
  let timer = null

  watch(
    source,
    (active) => {
      window.clearTimeout(timer)
      if (active) {
        timer = window.setTimeout(() => {
          delayed.value = true
        }, delay)
      } else {
        delayed.value = false
      }
    },
    { immediate: true },
  )
  onBeforeUnmount(() => window.clearTimeout(timer))

  return delayed
}
