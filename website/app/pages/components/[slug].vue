<script setup lang="ts">
import catalogue from '~/data/catalogue.json'
import { repository, sourceUrl } from '~/data/project'
definePageMeta({
  validate: (route) => catalogue.some((item) => item.slug === route.params.slug),
})
const route = useRoute()
const { t } = useI18n()
const item = computed(() => catalogue.find((item) => item.slug === route.params.slug))
const featureCode = computed(
  () => `[dependencies.argui]\ngit = "${repository}"\nfeatures = ["widget-${item.value?.feature}"]`,
)
const related = computed(() => {
  const position = catalogue.findIndex((entry) => entry.slug === item.value?.slug)
  return [catalogue[position - 1], catalogue[position + 1]].filter((entry) => entry !== undefined)
})
usePageSeo(
  () => `${item.value?.name} component`,
  () => item.value?.description ?? '',
)
</script>
<template>
  <article v-if="item">
    <SourceLink :path="item.source" />
    <div class="component-page-heading">
      <h1>{{ item.name }}</h1>
      <p>{{ item.description }}</p>
    </div>
    <GalleryFrame v-if="item.gallery" :component="item.gallery" />
    <div v-else class="no-demo">
      <p>{{ t('components.noDemo') }}</p>
      <ActionLink to="/components" variant="secondary">{{ t('components.title') }}</ActionLink>
    </div>
    <p v-if="item.slug === 'updater'" class="component-note">{{ t('components.updaterNote') }}</p>
    <section v-if="item.feature" class="component-install">
      <h2>{{ t('components.enable') }}</h2>
      <CodeBlock :code="featureCode" filename="Cargo.toml" />
      <p>{{ t('components.integration') }}</p>
    </section>
    <ActionLink :to="sourceUrl(item.source)" external variant="text">
      {{
        t(item.category === 'Examples' ? 'components.exampleSource' : 'components.implementation')
      }}
    </ActionLink>
    <nav class="related-components" :aria-label="t('components.related')">
      <NuxtLink v-for="entry in related" :key="entry.slug" :to="`/components/${entry.slug}`">
        {{ entry.name }}
        <span aria-hidden="true">↗</span>
      </NuxtLink>
    </nav>
  </article>
</template>
