<script setup lang="ts">
import { ArrowUpRight } from '@lucide/vue'
import { sourceUrl } from '~/data/project'

const { t } = useI18n()
const selectedApp = ref<'ai-harness' | 'widget-gallery'>('ai-harness')
usePageSeo(
  () => t('meta.examples'),
  () => t('meta.examplesDescription'),
)
const moreExamples = ['motion', 'i18n', 'hot-reload', 'liquid-glass'] as const
const isHarness = computed(() => selectedApp.value === 'ai-harness')
const selectedTitle = computed(() =>
  t(isHarness.value ? 'appExamples.harnessTitle' : 'appExamples.galleryTitle'),
)
const selectedBody = computed(() =>
  t(isHarness.value ? 'appExamples.harnessBody' : 'appExamples.galleryBody'),
)
const selectedSource = computed(() =>
  sourceUrl(
    isHarness.value
      ? 'app_examples/fake-ai-harness/src/main.rs'
      : 'crates/argui-widget-gallery/src/main.rs',
  ),
)
</script>

<template>
  <main id="main-content" class="container examples-page">
    <div class="page-heading examples-heading">
      <h1>{{ t('appExamples.title') }}</h1>
      <p>{{ t('appExamples.intro') }}</p>
    </div>
    <nav class="app-example-list" :aria-label="t('appExamples.available')" role="tablist">
      <button
        type="button"
        role="tab"
        :aria-selected="isHarness"
        :class="{ selected: isHarness }"
        @click="selectedApp = 'ai-harness'"
      >
        <span>01</span>
        <strong>{{ t('appExamples.harnessTitle') }}</strong>
        <small>{{ t('appExamples.category') }}</small>
        <b>
          <time>0.30 s</time>
          {{ t('appExamples.startupLabel') }}
        </b>
      </button>
      <button
        type="button"
        role="tab"
        :aria-selected="!isHarness"
        :class="{ selected: !isHarness }"
        @click="selectedApp = 'widget-gallery'"
      >
        <span>02</span>
        <strong>{{ t('appExamples.galleryTitle') }}</strong>
        <small>{{ t('appExamples.galleryCategory') }}</small>
        <b>
          <time>0.79 s</time>
          {{ t('appExamples.startupLabel') }}
        </b>
      </button>
    </nav>
    <p class="app-startup-evidence">
      {{ t('appExamples.startupEvidence') }}
      <a
        :href="sourceUrl('docs/performance/data/wasm-startup.json')"
        target="_blank"
        rel="noopener noreferrer"
      >
        {{ t('appExamples.startupDetails') }} ↗
      </a>
    </p>
    <section class="app-example">
      <div class="app-example-copy">
        <div>
          <h2>{{ selectedTitle }}</h2>
          <p>{{ selectedBody }}</p>
        </div>
        <a :href="selectedSource" target="_blank" rel="noopener noreferrer">
          {{ t('appExamples.source') }}
          <ArrowUpRight :size="16" />
        </a>
      </div>
      <GalleryFrame :app="isHarness ? 'ai-harness' : undefined" />
    </section>
    <section class="more-examples">
      <div class="section-heading">
        <div>
          <h2>{{ t('appExamples.moreTitle') }}</h2>
          <p>{{ t('appExamples.moreBody') }}</p>
        </div>
      </div>
      <div class="more-example-grid">
        <NuxtLink v-for="slug in moreExamples" :key="slug" :to="`/components/${slug}`">
          <span>{{ t(`appExamples.more.${slug}.category`) }}</span>
          <strong>{{ t(`appExamples.more.${slug}.title`) }}</strong>
          <ArrowUpRight :size="16" />
        </NuxtLink>
      </div>
    </section>
  </main>
</template>
