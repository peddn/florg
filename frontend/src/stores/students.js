import { defineStore } from 'pinia'

import { api } from '../api'

// The students of one school year.
export const useStudentsStore = defineStore('students', {
  state: () => ({
    // Id of the school year the list belongs to.
    yearId: null,
    // Students in display order. `null` until loaded.
    list: null,
  }),
  actions: {
    async load(yearId) {
      if (yearId !== this.yearId) {
        this.yearId = yearId
        this.list = null
      }
      const result = await api('GET', `/school-years/${yearId}/students`)
      // Ignore an answer that arrives after another school year was chosen.
      if (result.ok && yearId === this.yearId) {
        this.list = result.data
      }
      return result
    },
    create(input) {
      return this.changeAndReload(api('POST', `/school-years/${this.yearId}/students`, input))
    },
    update(id, input) {
      return this.changeAndReload(
        api('PUT', `/school-years/${this.yearId}/students/${id}`, input),
      )
    },
    remove(id) {
      return this.changeAndReload(api('DELETE', `/school-years/${this.yearId}/students/${id}`))
    },
    // Reads a CSV text. With `dryRun` the backend only reports what it
    // would create.
    async importCsv(csv, dryRun) {
      const result = await api('POST', `/school-years/${this.yearId}/students/import`, {
        csv,
        dry_run: dryRun,
      })
      if (result.ok && result.data.imported) {
        await this.load(this.yearId)
      }
      return result
    },
    // The backend decides the order, so the list is loaded again.
    async changeAndReload(request) {
      const result = await request
      if (result.ok) {
        await this.load(this.yearId)
      }
      return result
    },
  },
})
