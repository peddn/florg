import { defineStore } from 'pinia'

import { api } from '../api'

// The learning groups of one school year, and the one shown on its page.
export const useLearningGroupsStore = defineStore('learningGroups', {
  state: () => ({
    // Id of the school year the list belongs to.
    yearId: null,
    // Summaries in display order. `null` until loaded.
    list: null,
    // The learning group shown on its own page, with its members.
    current: null,
  }),
  actions: {
    // Always starts from an empty list: member counts and names change on
    // other pages, and an old list must not show up even briefly.
    async load(yearId) {
      this.yearId = yearId
      this.list = null
      const result = await api('GET', `/school-years/${yearId}/learning-groups`)
      // Ignore an answer that arrives after another school year was chosen.
      if (result.ok && yearId === this.yearId) {
        this.list = result.data
      }
      return result
    },
    async create(input) {
      const result = await api('POST', `/school-years/${this.yearId}/learning-groups`, input)
      if (result.ok) {
        await this.load(this.yearId)
      }
      return result
    },
    async loadCurrent(id) {
      this.current = null
      const result = await api('GET', `/learning-groups/${id}`)
      if (result.ok) {
        this.current = result.data
      }
      return result
    },
    // The four actions below answer with the changed learning group.
    update(id, input) {
      return this.changeCurrent(api('PUT', `/learning-groups/${id}`, input))
    },
    addMembers(id, studentIds) {
      return this.changeCurrent(
        api('POST', `/learning-groups/${id}/members`, { student_ids: studentIds }),
      )
    },
    // Another period of a student who returns to the group.
    addPeriod(id, studentId, period) {
      return this.changeCurrent(
        api('POST', `/learning-groups/${id}/memberships`, { student_id: studentId, ...period }),
      )
    },
    // A membership is one period of a student in the group.
    setPeriod(membershipId, period) {
      return this.changeCurrent(api('PUT', `/group-memberships/${membershipId}`, period))
    },
    async removePeriod(id, membershipId) {
      const result = await api('DELETE', `/group-memberships/${membershipId}`)
      if (result.ok) {
        await this.loadCurrent(id)
      }
      return result
    },
    remove(id) {
      return api('DELETE', `/learning-groups/${id}`)
    },
    async changeCurrent(request) {
      const result = await request
      if (result.ok) {
        this.current = result.data
      }
      return result
    },
  },
})
