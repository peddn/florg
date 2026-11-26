import { defineStore } from 'pinia'

import { api } from '../api'

// Whether this browser is signed in, and the actions of the login.
export const useAuthStore = defineStore('auth', {
  state: () => ({
    // 'unknown' | 'signedIn' | 'signedOut'
    status: 'unknown',
  }),
  getters: {
    isSignedIn: (state) => state.status === 'signedIn',
  },
  actions: {
    async load() {
      const result = await api('GET', '/auth/status')
      this.status = result.ok && result.data.authenticated ? 'signedIn' : 'signedOut'
    },
    // Asks the backend for a login link. It only sends one if `email` is the
    // address of the account, and its answer never tells whether it was.
    requestLink(email) {
      return api('POST', '/auth/request-link', { email })
    },
    // Redeems the token from a login link.
    async confirm(token) {
      const result = await api('POST', '/auth/confirm', { token })
      if (result.ok) {
        this.status = 'signedIn'
      }
      return result
    },
    // Signs out this device only.
    logout() {
      return this.endSession('/auth/logout')
    },
    // Signs out every device.
    logoutAll() {
      return this.endSession('/auth/logout-all')
    },
    async endSession(path) {
      const result = await api('POST', path)
      // 401 means the session was already gone.
      if (result.ok || result.status === 401) {
        this.status = 'signedOut'
      }
      return result
    },
  },
})
