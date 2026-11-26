import { defineStore } from 'pinia'

import { api } from '../api'

// Where the browser remembers the school year chosen last.
const ACTIVE_KEY = 'florg.activeSchoolYear'

function storedActiveId() {
  const id = Number(window.localStorage.getItem(ACTIVE_KEY))
  return Number.isInteger(id) && id > 0 ? id : null
}

// Today as an ISO date in local time, comparable with the dates of the API.
function today() {
  const now = new Date()
  const pad = (value) => String(value).padStart(2, '0')
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`
}

// School years and their school-free periods.
export const useSchoolYearsStore = defineStore('schoolYears', {
  state: () => ({
    // Summaries of all school years, latest first. `null` until loaded.
    list: null,
    // School years of the holiday data file, offered when creating one.
    templates: [],
    // The school year shown on its own page, with its free periods.
    current: null,
    // Id of the school year the second menu row refers to.
    activeId: storedActiveId(),
  }),
  getters: {
    // The active school year, or null if there is none yet.
    active: (state) => (state.list ?? []).find((year) => year.id === state.activeId) ?? null,
  },
  actions: {
    async loadList() {
      const [years, templates] = await Promise.all([
        api('GET', '/school-years'),
        api('GET', '/school-years/templates'),
      ])
      if (years.ok) {
        this.list = years.data
        this.ensureActive()
      }
      if (templates.ok) {
        this.templates = templates.data
      }
      return years
    },
    setActive(id) {
      this.activeId = id
      if (id === null) {
        window.localStorage.removeItem(ACTIVE_KEY)
      } else {
        window.localStorage.setItem(ACTIVE_KEY, String(id))
      }
    },
    // Keeps the chosen school year if it still exists. Otherwise picks the
    // one that contains today, or else the latest.
    ensureActive() {
      if (this.active) {
        return
      }
      const day = today()
      const fallback =
        this.list.find((year) => year.start_date <= day && day <= year.end_date) ?? this.list[0]
      this.setActive(fallback?.id ?? null)
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
    // Creating, renaming and deleting change the list in the menu, so it
    // is loaded again afterwards.
    async create(input) {
      const result = await api('POST', '/school-years', input)
      if (result.ok) {
        await this.loadList()
      }
      return result
    },
    async update(id, input) {
      const result = await api('PUT', `/school-years/${id}`, input)
      if (result.ok) {
        this.current = result.data
        await this.loadList()
      }
      return result
    },
    async remove(id) {
      const result = await api('DELETE', `/school-years/${id}`)
      if (result.ok) {
        await this.loadList()
      }
      return result
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
