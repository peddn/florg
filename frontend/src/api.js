// Thin wrapper around fetch for the JSON API of the backend.

import { ref } from 'vue'

const UNREACHABLE = 'Das Backend ist nicht erreichbar.'
const FAILED = 'Die Anfrage ist fehlgeschlagen.'

// Fired when a request other than the login itself is answered with 401,
// i.e. the session has ended. `main.js` reacts by showing the login page.
export const UNAUTHENTICATED_EVENT = 'florg:unauthenticated'

// Number of changes (everything but GET) that are on their way to the
// server. The status bar shows "Speichert …" while it is above zero.
export const pendingChanges = ref(0)

// Sends a request and returns { ok, status, data, message }.
// `message` is a German text for the user if the request failed.
export async function api(method, path, body) {
  if (method === 'GET') {
    return send(method, path, body)
  }
  pendingChanges.value += 1
  try {
    return await send(method, path, body)
  } finally {
    pendingChanges.value -= 1
  }
}

async function send(method, path, body) {
  let response
  try {
    response = await fetch(`/api${path}`, {
      method,
      headers: body ? { 'Content-Type': 'application/json' } : undefined,
      body: body ? JSON.stringify(body) : undefined,
    })
  } catch {
    return { ok: false, status: 0, data: null, message: UNREACHABLE }
  }

  let data = null
  try {
    data = await response.json()
  } catch {
    // Responses without content are fine.
  }

  if (response.status === 401 && !path.startsWith('/auth/')) {
    window.dispatchEvent(new Event(UNAUTHENTICATED_EVENT))
  }

  const message = response.ok ? null : (data?.message ?? FAILED)
  return { ok: response.ok, status: response.status, data, message }
}
