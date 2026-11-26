import { defineStore } from 'pinia'

import { api } from '../api'

// Builds a store for one of the small lists the teacher maintains.
// `path` is the collection in the API, e.g. '/subjects'.
function defineListStore(name, path) {
  return defineStore(name, {
    state: () => ({
      // All entries in display order. `null` until loaded.
      list: null,
    }),
    actions: {
      async load() {
        const result = await api('GET', path)
        if (result.ok) {
          this.list = result.data
        }
        return result
      },
      create(input) {
        return this.changeAndReload(api('POST', path, input))
      },
      update(id, input) {
        return this.changeAndReload(api('PUT', `${path}/${id}`, input))
      },
      remove(id) {
        return this.changeAndReload(api('DELETE', `${path}/${id}`))
      },
      // The backend decides the order, so the list is loaded again.
      async changeAndReload(request) {
        const result = await request
        if (result.ok) {
          await this.load()
        }
        return result
      },
    },
  })
}

export const useSubjectsStore = defineListStore('subjects', '/subjects')
export const useSchoolClassesStore = defineListStore('schoolClasses', '/classes')
