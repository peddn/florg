// Thin wrapper around fetch for the JSON API of the backend.

const UNREACHABLE = 'Das Backend ist nicht erreichbar.'

// Sends a request and returns { ok, status, data, message }.
// `message` is a German text for the user if the request failed.
export async function api(method, path, body) {
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

  const message = response.ok ? null : (data?.message ?? UNREACHABLE)
  return { ok: response.ok, status: response.status, data, message }
}
