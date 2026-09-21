<script setup lang="ts">
import { ArrowUpRight } from '@lucide/vue'
import { sourceUrl } from '~/data/project'

const { t } = useI18n()
const selectedApp = ref<'ai-harness' | 'widget-gallery' | 'gpu-canvas' | 'astra-editor'>(
  'ai-harness',
)
usePageSeo(
  () => t('meta.examples'),
  () => t('meta.examplesDescription'),
)
const moreExamples = ['motion', 'i18n', 'liquid-glass'] as const
const isHarness = computed(() => selectedApp.value === 'ai-harness')
const isGallery = computed(() => selectedApp.value === 'widget-gallery')
const isGpuCanvas = computed(() => selectedApp.value === 'gpu-canvas')
const isAstraEditor = computed(() => selectedApp.value === 'astra-editor')
const selectedTitle = computed(() => {
  if (isHarness.value) return t('appExamples.harnessTitle')
  if (isGpuCanvas.value) return t('appExamples.gpuTitle')
  if (isAstraEditor.value) return t('appExamples.astraTitle')
  return t('appExamples.galleryTitle')
})
const selectedBody = computed(() => {
  if (isHarness.value) return t('appExamples.harnessBody')
  if (isGpuCanvas.value) return t('appExamples.gpuBody')
  if (isAstraEditor.value) return t('appExamples.astraBody')
  return t('appExamples.galleryBody')
})
const selectedSource = computed(() => {
  if (isHarness.value) return sourceUrl('app_examples/fake-ai-harness/src/main.rs')
  if (isGpuCanvas.value) return sourceUrl('app_examples/gpu-canvas/src/main.rs')
  if (isAstraEditor.value) return sourceUrl('app_examples/astra-editor/src/main.rs')
  return sourceUrl('crates/argui-widget-gallery/src/main.rs')
})
const selectedPreviewApp = computed(() =>
  isGallery.value
    ? undefined
    : isGpuCanvas.value
      ? 'gpu-canvas'
      : isAstraEditor.value
        ? 'astra-editor'
        : 'ai-harness',
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
        :aria-selected="isGallery"
        :class="{ selected: isGallery }"
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
      <button
        type="button"
        role="tab"
        :aria-selected="isGpuCanvas"
        :class="{ selected: isGpuCanvas }"
        @click="selectedApp = 'gpu-canvas'"
      >
        <span>03</span>
        <strong>{{ t('appExamples.gpuTitle') }}</strong>
        <small>{{ t('appExamples.gpuCategory') }}</small>
        <b>{{ t('appExamples.gpuProof') }}</b>
      </button>
      <button
        type="button"
        role="tab"
        :aria-selected="isAstraEditor"
        :class="{ selected: isAstraEditor }"
        @click="selectedApp = 'astra-editor'"
      >
        <span>04</span>
        <strong>{{ t('appExamples.astraTitle') }}</strong>
        <small>{{ t('appExamples.astraCategory') }}</small>
        <b>{{ t('appExamples.astraProof') }}</b>
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
      <GalleryFrame :app="selectedPreviewApp" />
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
