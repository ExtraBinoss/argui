<script setup lang="ts">
import { ArrowUpRight } from '@lucide/vue'
import { discord, repository } from '~/data/project'
const { t } = useI18n()
</script>
<template>
  <footer class="site-footer container">
    <div>
      <NuxtLink to="/" :aria-label="t('nav.home')"><BrandLogo /></NuxtLink>
      <p>{{ t('footer.description') }}</p>
    </div>
    <p class="footer-note">{{ t('footer.note') }}</p>
    <div class="footer-links">
      <NuxtLink to="/docs">
        {{ t('footer.docs') }}
        <ArrowUpRight :size="13" />
      </NuxtLink>
      <a :href="repository" target="_blank" rel="noopener noreferrer">
        {{ t('footer.source') }}
        <ArrowUpRight :size="13" />
      </a>
      <a :href="discord" target="_blank" rel="noopener noreferrer">
        {{ t('footer.discord') }}
        <ArrowUpRight :size="13" />
      </a>
      <span>{{ t('footer.license') }}</span>
    </div>
  </footer>
</template>
