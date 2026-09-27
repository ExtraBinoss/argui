<script setup lang="ts">
import docs from '~/data/docs-index.json'
import { useSearchStore } from '~/stores/search'

type DocPage = typeof docs[number] & { source: string; html: string }
const props = defineProps<{ page: DocPage }>()
const search = useSearchStore()
type Example = 'custom-elements' | 'counter'
const parts = computed(() => {
  const result: { html: string; example: Example | null }[] = []
  const marker = /<!-- argui-example:(custom-elements|counter) -->/g
  let start = 0
  for (const match of props.page.html.matchAll(marker)) {
    result.push({ html: props.page.html.slice(start, match.index), example: null })
    result.push({ html: '', example: match[1] as Example })
    start = match.index + match[0].length
  }
  result.push({ html: props.page.html.slice(start), example: null })
  return result
})
const sections = ['Start', 'Interface', 'Desktop & mobile', 'Runtime', 'Rendering', 'Performance', 'Contributing', 'Repository']
const grouped = sections.map(name => ({ name, pages: docs.filter(page => page.section === name) })).filter(group => group.pages.length)
</script>

<template>
  <main id="main-content" class="docs-page shell">
    <aside class="docs-sidebar" aria-label="Documentation navigation"><div class="docs-sidebar-title">Documentation</div><button class="docs-search-button" type="button" @click="search.show()"><span aria-hidden="true">⌕</span><span>Search the site…</span><kbd>Ctrl F</kbd></button><nav v-for="group in grouped" :key="group.name"><div class="docs-nav-heading">{{ group.name }}</div><NuxtLink v-for="item in group.pages" :key="item.slug" :to="item.slug ? `/docs/${item.slug}` : '/docs'" :class="{ current: item.slug === page.slug }">{{ item.slug ? item.title : 'Overview' }}</NuxtLink></nav></aside>
    <div class="docs-main"><div class="docs-breadcrumb"><NuxtLink to="/docs">Docs</NuxtLink><span>/</span><span>{{ page.section }}</span></div><article class="markdown-body"><template v-for="(part, index) in parts" :key="index"><div v-if="part.example === null" v-html="part.html" /><div v-else class="doc-live-example"><GalleryFrame adapter="solid" :example="part.example" lazy /><p>Live Argui renderer · Solid TSX · WebAssembly</p></div></template></article><div class="docs-source"><a :href="`https://github.com/ExtraBinoss/argui/blob/main/${page.source}`" target="_blank" rel="noopener noreferrer">Edit this page on GitHub ↗</a></div></div>
  </main>
</template>
