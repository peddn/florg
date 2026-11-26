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

// Labels of `student.form_of_address` (concept S7).
export const FORMS_OF_ADDRESS = [
  { value: 'he', label: 'er' },
  { value: 'she', label: 'sie' },
  { value: 'name_only', label: 'nur Vorname' },
]

export function addressLabel(value) {
  return FORMS_OF_ADDRESS.find((entry) => entry.value === value)?.label ?? value
}

// "Anna Huber", or just the first name if there is no last name.
export function fullName(student) {
  return [student.first_name, student.last_name].filter(Boolean).join(' ')
}
