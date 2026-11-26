import { defineStore } from 'pinia'

// Connection state of the backend, shown on the start page.
export const useHealthStore = defineStore('health', {
  state: () => ({
    // 'unknown' | 'ok' | 'degraded' | 'unreachable'
    status: 'unknown',
    version: null,
  }),
  actions: {
    async check() {
      try {
        const response = await fetch('/api/health')
        const body = await response.json()
        this.status = body.status === 'ok' ? 'ok' : 'degraded'
        this.version = body.version ?? null
      } catch {
        this.status = 'unreachable'
        this.version = null
      }
    },
  },
})
