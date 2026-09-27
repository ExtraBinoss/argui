<script setup lang="ts">
import { useSearchStore } from '~/stores/search'

const menuOpen = ref(false)
const search = useSearchStore()
const route = useRoute()
const baseURL = useRuntimeConfig().app.baseURL
watch(() => route.fullPath, () => { menuOpen.value = false })
</script>

<template>
  <a class="skip-link" href="#main-content">Skip to content</a>
  <header class="site-header">
    <div class="shell header-inner">
      <NuxtLink class="brand" to="/" aria-label="Argui home">
        <img class="brand-mark" :src="`${baseURL}argui-icon.png`" width="29" height="29" alt="" aria-hidden="true" />
        <span>argui</span>
      </NuxtLink>
      <nav class="desktop-nav" aria-label="Main navigation">
        <NuxtLink to="/components">Components</NuxtLink>
        <NuxtLink to="/docs">Docs</NuxtLink>
        <NuxtLink to="/docs/cli">CLI</NuxtLink>
        <a class="social-nav-link" href="https://discord.gg/xY9CWSc65" target="_blank" rel="noopener noreferrer"><SiteIcon name="discord" />Discord</a>
        <a class="social-nav-link" href="https://github.com/ExtraBinoss/argui" target="_blank" rel="noopener noreferrer"><SiteIcon name="github" />GitHub<GithubStars /></a>
      </nav>
      <div class="header-actions">
        <button class="header-search" type="button" aria-label="Search the site" @click="search.show()"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><circle cx="10.8" cy="10.8" r="6.8" /><path d="m16 16 5 5" /></svg><span>Search</span><kbd>Ctrl F</kbd></button>
        <NuxtLink class="button button-small button-outline header-cta" to="/docs/getting-started">Get started <span aria-hidden="true">↗</span></NuxtLink>
        <button class="menu-toggle" type="button" :aria-expanded="menuOpen" aria-controls="mobile-navigation" @click="menuOpen = !menuOpen">
          <span class="sr-only">Toggle navigation</span><span aria-hidden="true">☰</span>
        </button>
      </div>
    </div>
    <nav v-if="menuOpen" id="mobile-navigation" class="mobile-nav shell" aria-label="Mobile navigation">
      <NuxtLink to="/components">Components</NuxtLink>
      <NuxtLink to="/docs">Docs</NuxtLink>
      <NuxtLink to="/docs/cli">CLI</NuxtLink>
      <NuxtLink to="/docs/getting-started">Get started</NuxtLink>
      <a class="social-nav-link" href="https://discord.gg/xY9CWSc65" target="_blank" rel="noopener noreferrer"><SiteIcon name="discord" />Discord</a>
      <a class="social-nav-link" href="https://github.com/ExtraBinoss/argui" target="_blank" rel="noopener noreferrer"><SiteIcon name="github" />GitHub<GithubStars /></a>
    </nav>
  </header>
</template>
