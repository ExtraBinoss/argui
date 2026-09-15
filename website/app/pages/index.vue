<script setup lang="ts">
import {
  Accessibility,
  ArrowUpRight,
  Cpu,
  Layers,
  PanelTop,
  RefreshCw,
  Sparkles,
} from '@lucide/vue'
import catalogue from '~/data/catalogue.json'
import { composeExample, sourceUrl } from '~/data/project'
const { t } = useI18n()
const asset = usePublicAsset()
usePageSeo(
  () => t('meta.home'),
  () => t('meta.description'),
)
const cards = [
  { key: 'performance', icon: Cpu, path: 'docs/performance/optimizations.md' },
  { key: 'accessibility', icon: Accessibility, path: 'docs/ui/interaction.md' },
  { key: 'shaders', icon: Sparkles, path: 'docs/rendering/effects.md' },
]
const components = [
  { slug: 'button', name: 'Button', icon: PanelTop },
  { slug: 'dialog', name: 'Dialog', icon: Layers },
  { slug: 'updater', name: 'Updater', icon: RefreshCw },
]
const componentCount = catalogue.filter((item) => item.category === 'Components').length
</script>
<template>
  <main id="main-content">
    <section class="hero container">
      <div class="hero-heading">
        <h1>
          {{ t('home.title') }}
          <span>{{ t('home.titleAccent') }}</span>
        </h1>
        <div class="hero-intro">
          <p>{{ t('home.intro') }}</p>
        </div>
      </div>
      <div class="hero-actions">
        <ActionLink to="/docs/start/installation">{{ t('home.code') }}</ActionLink>
        <ActionLink to="/components" variant="secondary">{{ t('home.gallery') }}</ActionLink>
      </div>
      <div class="hero-preview">
        <div class="preview-grid" aria-hidden="true" />
        <NuxtLink to="/components" class="preview-window">
          <div class="window-chrome" aria-hidden="true">
            <span />
            <span />
            <span />
            <code>argui-widget-gallery</code>
          </div>
          <img
            :src="asset('gallery-preview.webp')"
            :alt="t('home.previewAlt')"
            width="1120"
            height="700"
            fetchpriority="high"
          />
        </NuxtLink>
        <div class="preview-caption">
          <span>{{ t('home.preview') }}</span>
          <NuxtLink to="/components">
            {{ t('home.openGallery') }}
            <ArrowUpRight :size="16" />
          </NuxtLink>
        </div>
      </div>
    </section>
    <section class="metrics-section container" aria-label="Performance">
      <div class="metric">
        <strong>
          {{ t('stats.memoryValue') }}
          <span>~</span>
        </strong>
        <span>{{ t('stats.memory') }}</span>
      </div>
      <div class="metric">
        <strong class="metric-words">{{ t('stats.idleValue') }}</strong>
        <span>{{ t('stats.idle') }}</span>
      </div>
      <div class="metric">
        <strong class="metric-words">{{ t('stats.rendererValue') }}</strong>
        <span>{{ t('stats.renderer') }}</span>
      </div>
      <p class="metrics-evidence">
        {{ t('stats.evidence') }}
        <a
          :href="sourceUrl('docs/performance/optimizations.md')"
          target="_blank"
          rel="noopener noreferrer"
        >
          {{ t('stats.link') }} ↗
        </a>
      </p>
    </section>
    <section class="foundation-section container">
      <div class="split-heading">
        <h2>{{ t('home.foundation') }}</h2>
        <div>
          <p>{{ t('home.foundationBody') }}</p>
          <ActionLink to="/features" variant="text">{{ t('home.allFeatures') }}</ActionLink>
        </div>
      </div>
      <div class="feature-grid">
        <FeatureCard
          v-for="card in cards"
          :key="card.key"
          :icon="card.icon"
          :title="t(`features.${card.key}.title`)"
          :body="t(`features.${card.key}.body`)"
          :href="sourceUrl(card.path)"
        />
      </div>
    </section>
    <section class="component-teaser container">
      <div class="section-heading">
        <h2>{{ t('home.componentsTitle', { count: componentCount }) }}</h2>
        <ActionLink to="/components" variant="text">{{ t('home.browseAll') }}</ActionLink>
      </div>
      <div class="teaser-grid">
        <NuxtLink
          v-for="item in components"
          :key="item.slug"
          :to="`/components/${item.slug}`"
          class="teaser-card"
        >
          <div class="teaser-art" :class="`teaser-${item.slug}`" aria-hidden="true">
            <template v-if="item.slug === 'button'">
              <span class="demo-button">
                Save changes
                <span>↗</span>
              </span>
              <span class="demo-button outline">Cancel</span>
            </template>
            <template v-else-if="item.slug === 'dialog'">
              <div class="demo-dialog">
                <span class="demo-line short" />
                <span class="demo-line" />
                <span class="demo-line medium" />
                <div>
                  <span />
                  <span />
                </div>
              </div>
            </template>
            <template v-else>
              <div class="demo-update">
                <RefreshCw :size="21" />
                <div>
                  <span class="demo-line short" />
                  <span class="demo-progress" />
                </div>
                <span class="demo-percent">78%</span>
              </div>
            </template>
          </div>
          <div class="teaser-label">
            <span>{{ item.name }}</span>
            <ArrowUpRight :size="17" />
          </div>
        </NuxtLink>
      </div>
    </section>
    <section class="code-section container">
      <div>
        <h2>{{ t('home.codeTitle') }}</h2>
        <p>{{ t('home.codeBody') }}</p>
        <ActionLink to="/docs/start/installation" variant="text">
          {{ t('home.code') }}
        </ActionLink>
      </div>
      <CodeBlock :code="composeExample" filename="view.rs" />
    </section>
    <section class="closing-section container">
      <h2>{{ t('home.closing') }}</h2>
      <ActionLink to="/docs/start/installation">{{ t('home.closingLink') }}</ActionLink>
    </section>
  </main>
</template>
