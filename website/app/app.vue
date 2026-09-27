<script setup lang="ts">
import { useGithubStore } from '~/stores/github'

const github = useGithubStore()
let starsTimer: ReturnType<typeof setInterval> | undefined
function refreshStars() { void github.refreshStars() }
onMounted(() => {
  refreshStars()
  window.addEventListener('focus', refreshStars)
  starsTimer = setInterval(refreshStars, 5 * 60_000)
})
onUnmounted(() => {
  window.removeEventListener('focus', refreshStars)
  clearInterval(starsTimer)
})
useHead({ titleTemplate: title => title ? `${title} · Argui` : 'Argui — Native interfaces, one Rust engine' })
</script>

<template>
  <NuxtRouteAnnouncer />
  <SiteHeader />
  <SearchDialog />
  <NuxtPage />
  <SiteFooter />
</template>
