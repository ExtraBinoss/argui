<script setup lang="ts">
import { components } from '~/data/components'
import { useSearchStore } from '~/stores/search'
import { searchEntries, type SearchEntry, type SearchHit } from '~/utils/search'

const search = useSearchStore()
const router = useRouter()
const route = useRoute()
const baseURL = useRuntimeConfig().app.baseURL
const dialog = ref<HTMLDialogElement | null>(null)
const input = ref<HTMLInputElement | null>(null)
const query = ref('')
const activeIndex = ref(0)
const guides = shallowRef<SearchEntry[]>([])
const loading = ref(false)
const failed = ref(false)
let loaded = false

const featured: SearchEntry[] = [
  { title: 'Getting started', section: 'Guide', kind: 'guide', href: '/docs/getting-started', text: 'Install Argui and render your first interface.' },
  { title: 'Argui CLI', section: 'Guide', kind: 'guide', href: '/docs/cli', text: 'Install Argui 0.4 and learn its application commands.' },
  { title: 'Custom components and state', section: 'Guide', kind: 'guide', href: '/docs/ui/custom-components', text: 'Rectangle, row, column, text, WebAssembly examples, and a stateful counter.' },
  { title: 'Components', section: 'Gallery', kind: 'page', href: '/components', text: 'Explore the live Solid and React component gallery.' },
]
const sitePages: SearchEntry[] = [
  { title: 'Home', section: 'Argui', kind: 'page', href: '/', text: 'Native Rust interfaces with Solid and React TSX, WebAssembly previews, and component documentation.' },
  { title: 'Components', section: 'Gallery', kind: 'page', href: '/components', text: 'Live widget gallery with Button, Checkbox, Input field, Popover, Progress, Select, Slider, Switch, Tabs, Tooltip, and Virtual list.' },
]
const widgetEntries: SearchEntry[] = components.map(component => ({
  title: component.name,
  section: 'Components',
  kind: 'component',
  href: `/components?component=${component.slug}`,
  text: component.description,
}))
const entries = computed(() => [...sitePages, ...widgetEntries, ...guides.value])
const results = computed<SearchHit[]>(() => query.value.trim()
  ? searchEntries(entries.value, query.value)
  : featured.map(entry => ({ ...entry, score: 0, excerpt: entry.text })))

async function loadGuides() {
  if (loaded || loading.value) return
  loading.value = true
  failed.value = false
  try {
    const response = await fetch(`${baseURL}search-index.json`)
    if (!response.ok) throw new Error(`Search index returned ${response.status}`)
    guides.value = await response.json() as SearchEntry[]
    loaded = true
  } catch {
    failed.value = true
  } finally {
    loading.value = false
  }
}

function shortcut(event: KeyboardEvent) {
  if (event.altKey || !(event.ctrlKey || event.metaKey)) return
  if (event.key.toLowerCase() !== 'f' && event.key.toLowerCase() !== 'k') return
  event.preventDefault()
  search.show()
  input.value?.focus()
  input.value?.select()
}

function onInputKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    if (!results.value.length) return
    activeIndex.value = (activeIndex.value + (event.key === 'ArrowDown' ? 1 : -1) + results.value.length) % results.value.length
  } else if (event.key === 'Enter') {
    event.preventDefault()
    const result = results.value[activeIndex.value]
    if (result) void visit(result.href)
  }
}

async function visit(href: string) {
  search.hide()
  await router.push(href)
}

function dismissBackdrop(event: MouseEvent) {
  if (event.target !== dialog.value) return
  const bounds = dialog.value.getBoundingClientRect()
  if (event.clientX < bounds.left || event.clientX > bounds.right || event.clientY < bounds.top || event.clientY > bounds.bottom) search.hide()
}

watch(query, () => { activeIndex.value = 0 })
watch(() => route.fullPath, () => search.hide())
watch(() => search.open, async open => {
  if (open) {
    query.value = ''
    dialog.value?.showModal()
    await nextTick()
    input.value?.focus()
    void loadGuides()
  } else if (dialog.value?.open) {
    dialog.value.close()
  }
})
onMounted(() => window.addEventListener('keydown', shortcut))
onUnmounted(() => window.removeEventListener('keydown', shortcut))
</script>

<template>
  <dialog ref="dialog" class="site-search-dialog" aria-label="Search Argui" @close="search.hide()" @click="dismissBackdrop">
    <div class="site-search-panel">
      <div class="site-search-field">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><circle cx="10.8" cy="10.8" r="6.8" /><path d="m16 16 5 5" /></svg>
        <input ref="input" v-model="query" type="search" placeholder="Search docs and components…" aria-label="Search docs and components" aria-controls="site-search-results" @keydown="onInputKeydown">
        <button type="button" class="site-search-close" aria-label="Close search" @click="search.hide()">Esc</button>
      </div>
      <div id="site-search-results" class="site-search-results">
        <div class="site-search-caption"><span>{{ query.trim() ? 'RESULTS' : 'QUICK LINKS' }}</span><span v-if="loading">Loading guides…</span></div>
        <span class="sr-only" aria-live="polite">{{ query.trim() ? `${results.length} results. ${results[activeIndex]?.title ?? ''} selected.` : 'Quick links.' }}</span>
        <nav v-if="results.length" aria-label="Search results">
          <NuxtLink v-for="(result, index) in results" :key="result.href" :to="result.href" class="site-search-result" :class="{ selected: index === activeIndex }" @mouseenter="activeIndex = index" @click="search.hide()">
            <span class="site-search-result-icon" aria-hidden="true"><svg viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.5"><template v-if="result.kind === 'component'"><rect x="2.5" y="2.5" width="15" height="15" rx="2" /><path d="M2.5 7.5h15M7.5 7.5v10" /></template><template v-else-if="result.kind === 'page'"><path d="m2.5 9 7.5-6 7.5 6v7.5a1 1 0 0 1-1 1h-13a1 1 0 0 1-1-1zM7.5 17.5v-6h5v6" /></template><template v-else><path d="M4 2.5h8l4 4v11H4zM12 2.5v4h4M6.5 10h7M6.5 13h7" /></template></svg></span>
            <span class="site-search-result-copy"><strong>{{ result.title }}</strong><small>{{ result.section === result.title ? 'Guide' : result.section }} <span v-if="result.excerpt">· {{ result.excerpt }}</span></small></span>
            <span class="site-search-result-arrow" aria-hidden="true">↗</span>
          </NuxtLink>
        </nav>
        <div v-else class="site-search-empty">{{ loading ? 'Searching guides…' : 'No results. Try another word or component name.' }}</div>
        <div v-if="failed" class="site-search-error">Guide search could not load. <button type="button" @click="loadGuides">Retry</button></div>
      </div>
      <div class="site-search-footer"><span><kbd>↑</kbd><kbd>↓</kbd> navigate</span><span><kbd>↵</kbd> open</span><span><kbd>Esc</kbd> close</span></div>
    </div>
  </dialog>
</template>
