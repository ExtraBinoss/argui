<script setup lang="ts">
import { ArrowUpRight } from '@lucide/vue'
import catalogue from '~/data/catalogue.json'
const { t } = useI18n()
usePageSeo(
  () => t('meta.components'),
  () => t('meta.componentsDescription'),
)
</script>
<template>
  <div>
    <SourceLink path="crates/argui-widget-gallery/src/lib.rs" />
    <div class="component-page-heading">
      <h1>{{ t('components.title') }}</h1>
      <p>{{ t('components.galleryBody') }}</p>
    </div>
    <GalleryFrame />
    <div class="section-heading component-index-heading">
      <h2>{{ t('components.all') }}</h2>
      <span>{{ t('components.count', { count: catalogue.length }) }}</span>
    </div>
    <div class="component-index">
      <NuxtLink v-for="item in catalogue" :key="item.slug" :to="`/components/${item.slug}`">
        <span>{{ item.name }}</span>
        <ArrowUpRight :size="15" />
      </NuxtLink>
    </div>
  </div>
</template>
