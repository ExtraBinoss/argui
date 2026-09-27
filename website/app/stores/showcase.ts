import { defineStore } from 'pinia'
import { components, type Adapter, type ComponentSlug } from '~/data/components'

export const useShowcaseStore = defineStore('showcase', {
  state: () => ({
    adapter: 'solid' as Adapter,
    selected: 'button' as ComponentSlug,
    query: '',
  }),
  getters: {
    filtered(state) {
      const query = state.query.trim().toLowerCase()
      return query ? components.filter(component => `${component.name} ${component.description}`.toLowerCase().includes(query)) : components
    },
    active(state) {
      return components.find(component => component.slug === state.selected) ?? components[0]
    },
  },
  actions: {
    choose(slug: ComponentSlug) { this.selected = slug },
    useAdapter(adapter: Adapter) { this.adapter = adapter },
  },
})
