<script setup lang="ts">
import { ChevronDown, Search, X } from '@lucide/vue'
import { docsByCategory } from '~/data/docs'

const open = ref(false)
const query = ref('')
const route = useRoute()
const current = computed(() => route.path.replace(/^\/docs\/?/, '') || undefined)
const visible = computed(() => {
  const needle = query.value.trim().toLowerCase()
  if (!needle) return docsByCategory
  return docsByCategory
    .map((group) => ({
      ...group,
      guides: group.guides.filter((guide) =>
        `${guide.title} ${guide.description}`.toLowerCase().includes(needle),
      ),
    }))
    .filter((group) => group.guides.length > 0)
})
watch(
  () => route.path,
  () => {
    open.value = false
  },
)
</script>

<template>
  <div class="docs-nav-wrapper">
    <button
      class="docs-nav-toggle"
      type="button"
      :aria-expanded="open"
      aria-controls="docs-navigation"
      @click="open = !open"
    >
      Documentation
      <span>{{ current ? 'On this guide' : 'Browse guides' }}</span>
      <ChevronDown :size="16" :class="{ rotated: open }" />
    </button>
    <aside id="docs-navigation" class="docs-sidebar" :class="{ 'is-open': open }">
      <label class="docs-search">
        <Search :size="15" />
        <span class="sr-only">Search documentation</span>
        <input v-model="query" type="search" placeholder="Search documentation" />
        <button v-if="query" type="button" aria-label="Clear search" @click="query = ''">
          <X :size="14" />
        </button>
      </label>
      <NuxtLink class="docs-overview-link" to="/docs" :aria-current="!current ? 'page' : undefined">
        Documentation home
      </NuxtLink>
      <nav aria-label="Documentation guides">
        <section v-for="group in visible" :key="group.category" class="docs-nav-group">
          <h2>{{ group.category }}</h2>
          <NuxtLink
            v-for="guide in group.guides"
            :key="guide.slug"
            :to="`/docs/${guide.slug}`"
            :class="{ selected: guide.slug === current }"
            :aria-current="guide.slug === current ? 'page' : undefined"
          >
            {{ guide.title }}
          </NuxtLink>
        </section>
      </nav>
      <p v-if="visible.length === 0" class="docs-search-empty">No matching guides.</p>
    </aside>
  </div>
</template>
