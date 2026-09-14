<script setup lang="ts">
import { ArrowUpRight } from '@lucide/vue'
import { composeExample, repository, sourceUrl } from '~/data/project'
const { t } = useI18n()
usePageSeo(
  () => t('meta.start'),
  () => t('meta.startDescription'),
)
const run = `git clone ${repository}.git\ncd argui\ncargo run -p argui-widget-gallery --all-features`
const dependency = `[dependencies.argui]\ngit = "${repository}"\nfeatures = ["widget-button"]`
const links = [
  { key: 'examples', url: '/examples' },
  { key: 'docs', url: sourceUrl('docs/README.md') },
  { key: 'linux', url: sourceUrl('docs/platform/webview.md') },
  { key: 'web', url: '/components' },
]
</script>
<template>
  <main id="main-content" class="container start-page">
    <div class="page-heading">
      <h1>{{ t('start.title') }}</h1>
      <p>{{ t('start.intro') }}</p>
    </div>
    <div class="start-steps">
      <section>
        <div class="step-number" aria-hidden="true">01</div>
        <div>
          <h2>{{ t('start.run') }}</h2>
          <p>{{ t('start.runBody') }}</p>
          <CodeBlock :code="run" filename="Terminal" />
          <p class="step-note">{{ t('start.prerequisites') }}</p>
        </div>
      </section>
      <section>
        <div class="step-number" aria-hidden="true">02</div>
        <div>
          <h2>{{ t('start.add') }}</h2>
          <p>{{ t('start.addBody') }}</p>
          <CodeBlock :code="dependency" filename="Cargo.toml" />
        </div>
      </section>
      <section>
        <div class="step-number" aria-hidden="true">03</div>
        <div>
          <h2>{{ t('start.compose') }}</h2>
          <p>{{ t('start.composeBody') }}</p>
          <CodeBlock :code="composeExample" filename="view.rs" />
        </div>
      </section>
    </div>
    <section class="next-steps">
      <h2>{{ t('start.next') }}</h2>
      <div>
        <NuxtLink
          v-for="link in links"
          :key="link.key"
          :to="link.url"
          :external="link.url.startsWith('https:')"
          :target="link.url.startsWith('https:') ? '_blank' : undefined"
          rel="noopener noreferrer"
        >
          {{ t(`start.${link.key}`) }}
          <ArrowUpRight :size="18" />
        </NuxtLink>
      </div>
    </section>
  </main>
</template>
