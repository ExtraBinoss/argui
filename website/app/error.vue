<script setup lang="ts">
defineProps<{ error: { statusCode?: number } }>()
const { t } = useI18n()
useHead({ title: 'Page not found — Argui', meta: [{ name: 'robots', content: 'noindex' }] })
</script>
<template>
  <main class="error-page container">
    <BrandLogo />
    <span class="error-code">{{ error.statusCode ?? 500 }}</span>
    <h1>{{ t('error.title') }}</h1>
    <p>{{ t('error.body') }}</p>
    <button class="action-link action-primary" @click="clearError({ redirect: '/' })">
      {{ t('error.home') }}
      <span aria-hidden="true">→</span>
    </button>
  </main>
</template>
