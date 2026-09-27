import { defineStore } from 'pinia'

export const useGithubStore = defineStore('github', {
  state: () => ({ stars: null as number | null, loading: false, fetchedAt: 0 }),
  actions: {
    async refreshStars() {
      if (this.loading || Date.now() - this.fetchedAt < 5 * 60_000) return
      this.loading = true
      this.fetchedAt = Date.now()
      try {
        const response = await fetch('https://api.github.com/repos/ExtraBinoss/argui', {
          headers: { Accept: 'application/vnd.github+json' },
        })
        if (!response.ok) throw new Error(`GitHub returned ${response.status}`)
        const data = await response.json() as { stargazers_count?: unknown }
        this.stars = typeof data.stargazers_count === 'number' && Number.isInteger(data.stargazers_count)
          ? data.stargazers_count : null
      } catch {
        this.stars = null
      } finally {
        this.loading = false
      }
    },
  },
})
