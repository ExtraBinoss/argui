<script setup lang="ts">
const ui = useInterfaceStore()
const { t } = useI18n()
const preference = useCookie('argui-theme')
onPrehydrate(() => {
  const dark = document.cookie.split('; ').includes('argui-theme=dark')
  document.documentElement.dataset.theme = dark ? 'dark' : 'light'
})
onMounted(() => {
  ui.theme = preference.value === 'dark' ? 'dark' : 'light'
})
watch(
  () => ui.theme,
  (theme) => {
    if (import.meta.client) document.documentElement.dataset.theme = theme
  },
)
useHead({
  titleTemplate: (title) =>
    title?.includes('Argui') ? title : `${title ?? 'Native interfaces, pure Rust'} — Argui`,
})
</script>

<template>
  <NuxtRouteAnnouncer />
  <a class="skip-link" href="#main-content">{{ t('nav.skip') }}</a>
  <SiteHeader />
  <NuxtPage />
  <SiteFooter />
</template>
