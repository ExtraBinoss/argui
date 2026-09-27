import { defineStore } from 'pinia'

export const useSearchStore = defineStore('search', {
  state: () => ({ open: false }),
  actions: {
    show() { this.open = true },
    hide() { this.open = false },
  },
})
