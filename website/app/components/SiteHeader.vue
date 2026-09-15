<script setup lang="ts">
import { ArrowUpRight, Menu, Moon, Star, Sun, X } from '@lucide/vue'
import { discord, repository } from '~/data/project'
const { t } = useI18n()
const ui = useInterfaceStore()
const route = useRoute()
const open = ref(false)
const stars = ref<number | null>(null)
const formattedStars = computed(() =>
  stars.value === null
    ? '—'
    : Intl.NumberFormat('en', { notation: 'compact', maximumFractionDigits: 1 }).format(
        stars.value,
      ),
)
onMounted(async () => {
  const key = 'argui-github-stars'
  try {
    const cached = JSON.parse(localStorage.getItem(key) ?? 'null') as {
      count: number
      savedAt: number
    } | null
    if (cached && Date.now() - cached.savedAt < 3_600_000) stars.value = cached.count
    else {
      const response = await fetch('https://api.github.com/repos/ExtraBinoss/argui')
      if (!response.ok) return
      const data = (await response.json()) as { stargazers_count?: number }
      if (typeof data.stargazers_count !== 'number') return
      stars.value = data.stargazers_count
      localStorage.setItem(key, JSON.stringify({ count: stars.value, savedAt: Date.now() }))
    }
  } catch {
    // The repository link remains useful when the public API or storage is unavailable.
  }
})
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
        <OnboardingTooltip :message="t('nav.newHere')" storage-key="argui-docs-onboarding">
          <NuxtLink to="/docs">{{ t('nav.docs') }}</NuxtLink>
        </OnboardingTooltip>
        <NuxtLink to="/components">{{ t('nav.components') }}</NuxtLink>
        <NuxtLink to="/examples">{{ t('nav.examples') }}</NuxtLink>
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
        <a
          class="icon-button discord-link"
          :href="discord"
          target="_blank"
          rel="noopener noreferrer"
          :aria-label="t('nav.discord')"
          :title="t('nav.discord')"
        >
          <img src="/discord.svg" alt="" width="20" height="15" />
        </a>
        <a class="github-link" :href="repository" target="_blank" rel="noopener noreferrer">
          <span class="github-name">{{ t('nav.github') }}</span>
          <span class="github-stars" :aria-label="t('nav.stars', { count: stars ?? 0 })">
            <Star :size="14" />
            {{ formattedStars }}
          </span>
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
      <OnboardingTooltip :message="t('nav.newHere')" storage-key="argui-docs-onboarding">
        <NuxtLink to="/docs">{{ t('nav.docs') }}</NuxtLink>
      </OnboardingTooltip>
      <NuxtLink to="/components">{{ t('nav.components') }}</NuxtLink>
      <NuxtLink to="/examples">{{ t('nav.examples') }}</NuxtLink>
      <a :href="discord" target="_blank" rel="noopener noreferrer">{{ t('nav.discord') }}</a>
    </nav>
  </header>
</template>
