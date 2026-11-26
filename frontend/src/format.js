// German display of the ISO dates the backend uses ("2026-09-15").

const dateFormat = new Intl.DateTimeFormat('de-DE', {
  weekday: 'short',
  day: '2-digit',
  month: '2-digit',
  year: 'numeric',
  timeZone: 'UTC',
})

// "Di., 15.09.2026"
export function formatDate(iso) {
  return dateFormat.format(new Date(`${iso}T00:00:00Z`))
}

// A single day or "first – last".
export function formatRange(start, end) {
  return start === end ? formatDate(start) : `${formatDate(start)} – ${formatDate(end)}`
}

// Labels of `school_free_period.kind`.
export const FREE_PERIOD_KINDS = [
  { value: 'holiday', label: 'Ferien' },
  { value: 'public_holiday', label: 'Feiertag' },
  { value: 'other', label: 'Sonstiges' },
]

export function kindLabel(kind) {
  return FREE_PERIOD_KINDS.find((entry) => entry.value === kind)?.label ?? kind
}
