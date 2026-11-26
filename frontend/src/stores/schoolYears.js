import { defineStore } from 'pinia'

import { api } from '../api'

// School years and their school-free periods.
export const useSchoolYearsStore = defineStore('schoolYears', {
  state: () => ({
    // Summaries of all school years, latest first. `null` until loaded.
    list: null,
    // School years of the holiday data file, offered when creating one.
    templates: [],
    // The school year shown on its own page, with its free periods.
    current: null,
  }),
  actions: {
    async loadList() {
      const [years, templates] = await Promise.all([
        api('GET', '/school-years'),
        api('GET', '/school-years/templates'),
      ])
      if (years.ok) {
        this.list = years.data
      }
      if (templates.ok) {
        this.templates = templates.data
      }
      return years
    },
    async loadCurrent(id) {
      this.current = null
      return this.reloadCurrent(id)
    },
    async reloadCurrent(id) {
      const result = await api('GET', `/school-years/${id}`)
      if (result.ok) {
        this.current = result.data
      }
      return result
    },
    create(input) {
      return api('POST', '/school-years', input)
    },
    async update(id, input) {
      const result = await api('PUT', `/school-years/${id}`, input)
      if (result.ok) {
        this.current = result.data
      }
      return result
    },
    remove(id) {
      return api('DELETE', `/school-years/${id}`)
    },
    // The three actions below change the number of school days, so the
    // school year is loaded again afterwards.
    async addFreePeriod(schoolYearId, input) {
      return this.changeAndReload(
        schoolYearId,
        api('POST', `/school-years/${schoolYearId}/free-periods`, input),
      )
    },
    async updateFreePeriod(schoolYearId, id, input) {
      return this.changeAndReload(schoolYearId, api('PUT', `/free-periods/${id}`, input))
    },
    async removeFreePeriod(schoolYearId, id) {
      return this.changeAndReload(schoolYearId, api('DELETE', `/free-periods/${id}`))
    },
    async changeAndReload(schoolYearId, request) {
      const result = await request
      if (result.ok) {
        await this.reloadCurrent(schoolYearId)
      }
      return result
    },
  },
})
