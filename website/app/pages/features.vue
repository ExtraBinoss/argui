<script setup lang="ts">
import {
  Accessibility,
  Cpu,
  Globe,
  Languages,
  PanelTop,
  RefreshCw,
  RotateCw,
  Sparkles,
} from '@lucide/vue'
import { sourceUrl } from '~/data/project'
const { t } = useI18n()
usePageSeo(
  () => t('meta.features'),
  () => t('meta.featuresDescription'),
)
const features = [
  { key: 'performance', icon: Cpu, path: 'docs/performance/optimizations.md' },
  { key: 'accessibility', icon: Accessibility, path: 'docs/ui/interaction.md' },
  { key: 'shaders', icon: Sparkles, path: 'docs/rendering/effects.md' },
  { key: 'popovers', icon: PanelTop, path: 'docs/widgets/overlays.md' },
  { key: 'updater', icon: RefreshCw, path: 'docs/platform/updater.md' },
  { key: 'platforms', icon: Globe, path: 'docs/architecture.md' },
  { key: 'hotReload', icon: RotateCw, path: 'docs/hot-reload.md' },
  { key: 'i18n', icon: Languages, path: 'docs/i18n.md' },
]
</script>
<template>
  <main id="main-content" class="container features-page">
    <div class="page-heading">
      <h1>{{ t('features.title') }}</h1>
    </div>
    <div class="feature-grid full-feature-grid">
      <FeatureCard
        v-for="item in features"
        :key="item.key"
        :icon="item.icon"
        :title="t(`features.${item.key}.title`)"
        :body="t(`features.${item.key}.body`)"
        :href="sourceUrl(item.path)"
      />
    </div>
    <div class="feature-evidence">
      <Cpu :size="20" />
      <p>{{ t('stats.evidence') }}</p>
      <ActionLink :to="sourceUrl('docs/performance/optimizations.md')" external variant="text">
        {{ t('features.performanceLink') }}
      </ActionLink>
    </div>
    <p class="experimental-note">{{ t('features.experimental') }}</p>
    <ActionLink to="/get-started">{{ t('nav.start') }}</ActionLink>
  </main>
</template>
