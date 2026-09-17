<script setup lang="ts">
import { ArrowRight, BookOpen, Boxes, Code2, Cpu, MonitorSmartphone, Search } from '@lucide/vue'
import { docs, docsByCategory } from '~/data/docs'

const query = ref('')
const searchInput = ref<HTMLInputElement>()
const matches = computed(() => {
  const needle = query.value.trim().toLowerCase()
  if (!needle) return []
  return docs.filter((guide) =>
    `${guide.title} ${guide.description} ${guide.category}`.toLowerCase().includes(needle),
  )
})
const startGuides = docsByCategory[0]?.guides ?? []
const icons = {
  'Start here': BookOpen,
  Essentials: Boxes,
  Advanced: Code2,
  Technicalities: Cpu,
  Platforms: MonitorSmartphone,
}
const focusSearch = (event: KeyboardEvent) => {
  if (event.key !== '/' || event.metaKey || event.ctrlKey || event.altKey) return
  if (event.target instanceof HTMLInputElement || event.target instanceof HTMLTextAreaElement)
    return
  event.preventDefault()
  searchInput.value?.focus()
}
onMounted(() => window.addEventListener('keydown', focusSearch))
onBeforeUnmount(() => window.removeEventListener('keydown', focusSearch))
usePageSeo(
  () => 'Documentation',
  () =>
    'Learn Argui from your first Rust window to retained architecture, platform support, responsive layout, async tasks and custom elements, with live WebAssembly examples.',
)
</script>

<template>
  <main id="main-content" class="docs-content docs-home">
    <header class="docs-hero">
      <span class="docs-eyebrow">Learn Argui</span>
      <h1>
        Build the first window.
        <br />
        Understand the whole engine.
      </h1>
      <p>
        A progressive guide to native Rust interfaces. Every chapter is tied to real source files
        and finishes with the actual Argui application running in WebAssembly.
      </p>
      <label class="docs-home-search">
        <Search :size="18" />
        <span class="sr-only">Search all documentation</span>
        <input
          ref="searchInput"
          v-model="query"
          type="search"
          placeholder="Search guides, state, layout…"
        />
        <kbd>/</kbd>
      </label>
      <div v-if="query" class="docs-search-results" aria-live="polite">
        <NuxtLink v-for="guide in matches" :key="guide.slug" :to="`/docs/${guide.slug}`">
          <span>{{ guide.category }}</span>
          <strong>{{ guide.title }}</strong>
          <ArrowRight :size="16" />
        </NuxtLink>
        <p v-if="matches.length === 0">No guide matches “{{ query }}”.</p>
      </div>
    </header>

    <section class="docs-path" aria-labelledby="recommended-path">
      <div>
        <span>Recommended path</span>
        <h2 id="recommended-path">New to Argui?</h2>
        <p>Go from an empty Cargo project to a responsive stateful interface in four steps.</p>
      </div>
      <ol>
        <li v-for="guide in startGuides" :key="guide.slug">
          <NuxtLink :to="`/docs/${guide.slug}`">
            <span>{{ String(startGuides.indexOf(guide) + 1).padStart(2, '0') }}</span>
            <strong>{{ guide.title }}</strong>
            <small>{{ guide.minutes }} min</small>
            <ArrowRight :size="17" />
          </NuxtLink>
        </li>
      </ol>
    </section>

    <section
      v-for="group in docsByCategory.slice(1)"
      :key="group.category"
      class="docs-category"
      :aria-labelledby="`category-${group.category.toLowerCase()}`"
    >
      <div class="docs-category-heading">
        <component :is="icons[group.category]" :size="20" />
        <h2 :id="`category-${group.category.toLowerCase()}`">{{ group.category }}</h2>
        <span>{{ group.guides.length }} guides</span>
      </div>
      <div class="docs-card-grid">
        <NuxtLink v-for="guide in group.guides" :key="guide.slug" :to="`/docs/${guide.slug}`">
          <div>
            <span>{{ guide.level }} · {{ guide.minutes }} min</span>
            <h3>{{ guide.title }}</h3>
            <p>{{ guide.description }}</p>
          </div>
          <ArrowRight :size="18" />
        </NuxtLink>
      </div>
    </section>
  </main>
</template>
