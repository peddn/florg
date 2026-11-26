<script setup>
import BusyButton from './BusyButton.vue'
import { ref } from 'vue'

import { addressLabel } from '../format'
import { useStudentsStore } from '../stores/students'

// Reports how many students were created.
const emit = defineEmits(['imported', 'cancel'])

const store = useStudentsStore()

const EXAMPLE = 'Vorname;Nachname;Klasse;Anrede\nAnna;Huber;7a;sie\nBen;Maier;7b;er'

const text = ref('')
const fileName = ref(null)
// The answer of the backend to a dry run, or null while entering.
const preview = ref(null)
const busy = ref(false)
const error = ref(null)

const statusLabels = { new: 'neu', duplicate: 'schon vorhanden', error: 'Fehler' }
const statusStyles = {
  new: 'bg-emerald-50 text-emerald-800',
  duplicate: 'bg-slate-100 text-slate-600',
  error: 'bg-red-50 text-red-800',
}

// Files from Excel are often not UTF-8. If the bytes are no valid UTF-8,
// they are read as Windows-1252, so umlauts survive.
async function readFile(file) {
  const buffer = await file.arrayBuffer()
  try {
    return new TextDecoder('utf-8', { fatal: true }).decode(buffer)
  } catch {
    return new TextDecoder('windows-1252').decode(buffer)
  }
}

async function fileChosen(event) {
  const file = event.target.files[0]
  if (file) {
    text.value = await readFile(file)
    fileName.value = file.name
  }
}

async function send(dryRun) {
  busy.value = true
  error.value = null
  const result = await store.importCsv(text.value, dryRun)
  busy.value = false
  if (!result.ok) {
    error.value = result.message
    return
  }
  if (result.data.imported) {
    emit('imported', result.data.new_count)
  } else {
    preview.value = result.data
  }
}

function back() {
  preview.value = null
  error.value = null
}
</script>

<template>
  <div v-if="!preview" class="grid gap-3">
    <p class="text-sm text-slate-600">
      Die erste Zeile nennt die Spalten: <strong>Vorname</strong> ist Pflicht,
      <strong>Nachname</strong>, <strong>Klasse</strong> und <strong>Anrede</strong> (er, sie oder
      leer) sind optional. Statt der Anrede geht auch <strong>Geschlecht</strong> (m, w, d).
      Trennzeichen ist Semikolon, Komma oder Tabulator.
    </p>
    <label class="block text-sm font-medium text-slate-700">
      CSV-Datei
      <input
        type="file"
        accept=".csv,.txt,text/csv,text/plain"
        class="mt-1 block w-full text-sm file:mr-3 file:rounded-lg file:border-0 file:bg-slate-100 file:px-3 file:py-2 file:font-medium"
        @change="fileChosen"
      />
    </label>
    <label class="block text-sm font-medium text-slate-700">
      oder Text einfügen, zum Beispiel kopierte Zellen aus einer Tabelle
      <textarea
        v-model="text"
        rows="8"
        spellcheck="false"
        :placeholder="EXAMPLE"
        class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 font-mono text-sm"
      />
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
        class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800"
        @click="send(true)"
        :busy="busy"
        :disabled="!text.trim()"
      >
        Vorschau
      </BusyButton>
    </div>
  </div>

  <div v-else class="grid gap-3">
    <p data-testid="import-summary">
      <strong>{{ preview.new_count }}</strong> neu,
      <strong>{{ preview.duplicate_count }}</strong> schon vorhanden,
      <strong>{{ preview.error_count }}</strong> fehlerhaft. Trennzeichen: {{ preview.delimiter }}.
    </p>
    <p v-if="preview.new_classes.length" class="text-sm text-slate-600">
      Diese Klassen werden angelegt: {{ preview.new_classes.join(', ') }}.
    </p>
    <p v-if="preview.ignored_columns.length" class="text-sm text-slate-600">
      Diese Spalten werden übergangen: {{ preview.ignored_columns.join(', ') }}.
    </p>
    <p v-if="preview.error_count" class="text-sm text-red-700" role="alert">
      Solange eine Zeile fehlerhaft ist, wird nichts angelegt. Korrigiere die Zeilen und versuche es
      erneut.
    </p>

    <div class="max-h-[45dvh] overflow-auto rounded-lg border border-slate-200">
      <table class="w-full text-left text-sm">
        <thead class="sticky top-0 border-b border-slate-200 bg-white text-slate-500">
          <tr>
            <th scope="col" class="px-3 py-2 font-medium">Zeile</th>
            <th scope="col" class="px-3 py-2 font-medium">Vorname</th>
            <th scope="col" class="px-3 py-2 font-medium">Nachname</th>
            <th scope="col" class="px-3 py-2 font-medium">Klasse</th>
            <th scope="col" class="px-3 py-2 font-medium">Anrede</th>
            <th scope="col" class="px-3 py-2 font-medium">Ergebnis</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-200">
          <tr v-for="row in preview.rows" :key="row.line">
            <td class="px-3 py-2 text-slate-500">{{ row.line }}</td>
            <td class="px-3 py-2">{{ row.first_name }}</td>
            <td class="px-3 py-2">{{ row.last_name }}</td>
            <td class="px-3 py-2">{{ row.school_class }}</td>
            <td class="px-3 py-2">{{ addressLabel(row.form_of_address) }}</td>
            <td class="px-3 py-2">
              <span
                class="rounded-full px-2 py-0.5 text-xs font-medium whitespace-nowrap"
                :class="statusStyles[row.status]"
              >
                {{ statusLabels[row.status] }}
              </span>
              <span v-if="row.message" class="mt-1 block text-xs text-red-700">
                {{ row.message }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <p v-if="error" class="text-sm text-red-700" role="alert">{{ error }}</p>
    <div class="mt-2 flex justify-end gap-2">
      <button
        type="button"
        class="rounded-lg px-4 py-2 font-medium text-slate-600 hover:bg-slate-100"
        @click="back"
      >
        Zurück
      </button>
      <BusyButton
        class="rounded-lg bg-teal-700 px-4 py-2 font-medium text-white hover:bg-teal-800"
        @click="send(false)"
        :busy="busy"
        :disabled="preview.error_count > 0 || preview.new_count === 0"
      >
        {{ preview.new_count }} Schüler anlegen
      </BusyButton>
    </div>
  </div>
</template>
