<script setup lang="ts">
import { components, isComponentSlug } from '~/data/components'
import { useShowcaseStore } from '~/stores/showcase'

const showcase = useShowcaseStore()
const route = useRoute()
const router = useRouter()
const searchInput = ref<HTMLInputElement | null>(null)
const docsSlug: Record<string, string> = {
  button: 'ui/controls', 'button-group': 'ui/controls', checkbox: 'ui/controls',
  'input-field': 'ui/editing', popover: 'ui/controls', progress: 'ui/controls',
  select: 'ui/controls', slider: 'ui/controls', switch: 'ui/controls',
  tabs: 'ui/controls', tooltip: 'ui/controls', 'virtual-list': 'ui/scroll',
}

watch(() => route.query.component, value => {
  if (isComponentSlug(value)) showcase.choose(value)
}, { immediate: true })
watch(() => showcase.selected, component => {
  if (route.query.component !== component) void router.replace({ query: { ...route.query, component } })
})
function searchShortcut(event: KeyboardEvent) {
  if (event.key !== '/' || event.metaKey || event.ctrlKey || event.altKey) return
  if (event.target instanceof HTMLElement && event.target.closest('input, textarea, [contenteditable="true"]')) return
  event.preventDefault()
  searchInput.value?.focus()
}
onMounted(() => window.addEventListener('keydown', searchShortcut))
onUnmounted(() => window.removeEventListener('keydown', searchShortcut))
useSeoMeta({
  title: 'Components',
  description: 'Explore 12 live Argui widget pages rendered by the Rust WebAssembly host. Switch between Solid and React TSX examples.',
})
</script>

<template>
  <main id="main-content" class="components-page shell">
    <div class="page-intro"><div class="section-kicker">THE LIVE GALLERY</div><h1>Components</h1><p>Carefully composed controls on a native Rust foundation. Pick a widget, then try the real renderer below.</p><div class="intro-meta"><span class="meta-pill"><span class="live-dot" /> {{ components.length }} widget pages</span><span>Solid + React</span><span>WebAssembly + WebGPU</span></div></div>
    <div class="components-toolbar"><div class="toolbar-label"><strong>Explore the gallery</strong><span>Choose an adapter and interact with the canvas.</span></div><div class="segmented" aria-label="Choose TSX adapter"><button type="button" :aria-pressed="showcase.adapter === 'solid'" @click="showcase.useAdapter('solid')">Solid</button><button type="button" :aria-pressed="showcase.adapter === 'react'" @click="showcase.useAdapter('react')">React</button></div></div>
    <div class="components-layout">
      <aside class="component-sidebar" aria-label="Component list">
        <label class="search-box"><span class="sr-only">Search components</span><span aria-hidden="true">⌕</span><input ref="searchInput" v-model="showcase.query" type="search" placeholder="Search components…"><kbd>/</kbd></label>
        <div class="sidebar-section-label">WIDGETS <span>{{ showcase.filtered.length }}</span></div>
        <nav class="component-list" aria-label="Widgets"><button v-for="component in showcase.filtered" :key="component.slug" type="button" :class="{ active: component.slug === showcase.selected }" :aria-current="component.slug === showcase.selected ? 'page' : undefined" @click="showcase.choose(component.slug)"><span>{{ component.name }}</span><span aria-hidden="true">↗</span></button><p v-if="!showcase.filtered.length" class="empty-search">No matching components.</p></nav>
      </aside>
      <div class="component-main"><GalleryFrame :adapter="showcase.adapter" :component="showcase.selected" /><div class="component-detail"><div><span class="detail-label">CURRENTLY VIEWING</span><h2>{{ showcase.active.name }}</h2><p>{{ showcase.active.description }}</p></div><NuxtLink class="button button-small button-outline" :to="`/docs/${docsSlug[showcase.selected]}`">Read the guide <span aria-hidden="true">↗</span></NuxtLink></div></div>
    </div>
    <div class="browser-note"><span class="note-icon">ⓘ</span><p>The browser gallery runs on WebGPU. Desktop-only services, such as native outside-window popups and screen overlays, are available in the native gallery.</p><NuxtLink to="/docs/platform/desktop">Desktop guide ↗</NuxtLink></div>
  </main>
</template>
