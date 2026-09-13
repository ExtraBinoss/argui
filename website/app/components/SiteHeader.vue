<script setup lang="ts">
import { ArrowUpRight, Menu, Moon, Sun, X } from '@lucide/vue'
import { repository } from '~/data/project'
const { t } = useI18n()
const ui = useInterfaceStore()
const route = useRoute()
const open = ref(false)
watch(
  () => route.path,
  () => {
    open.value = false
  },
)
</script>

<template>
  <header class="site-header">
    <div class="header-inner container">
      <NuxtLink to="/" :aria-label="t('nav.home')"><BrandLogo /></NuxtLink>
      <nav class="desktop-nav" :aria-label="t('nav.menu')">
        <NuxtLink to="/features">{{ t('nav.features') }}</NuxtLink>
        <NuxtLink to="/components">{{ t('nav.components') }}</NuxtLink>
        <NuxtLink to="/get-started">{{ t('nav.start') }}</NuxtLink>
      </nav>
      <div class="header-actions">
        <button
          class="icon-button theme-button"
          :aria-label="t(ui.theme === 'light' ? 'nav.dark' : 'nav.light')"
          @click="ui.toggleTheme"
        >
          <Sun v-if="ui.theme === 'dark'" :size="18" />
          <Moon v-else :size="18" />
        </button>
        <a class="github-link" :href="repository" target="_blank" rel="noopener noreferrer">
          {{ t('nav.github') }}
          <ArrowUpRight :size="15" />
        </a>
        <button
          class="icon-button mobile-menu-toggle"
          :aria-label="t('nav.menu')"
          :aria-expanded="open"
          aria-controls="mobile-navigation"
          @click="open = !open"
        >
          <X v-if="open" :size="20" />
          <Menu v-else :size="20" />
        </button>
      </div>
    </div>
    <nav
      v-if="open"
      id="mobile-navigation"
      class="mobile-navigation"
      :aria-label="t('nav.menu')"
      @keydown.esc="open = false"
    >
      <NuxtLink to="/features">{{ t('nav.features') }}</NuxtLink>
      <NuxtLink to="/components">{{ t('nav.components') }}</NuxtLink>
      <NuxtLink to="/get-started">{{ t('nav.start') }}</NuxtLink>
    </nav>
  </header>
</template>
