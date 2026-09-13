<script setup lang="ts">
import { Grid2X2, PanelLeft, Search, X } from '@lucide/vue'
import catalogue from '~/data/catalogue.json'
const ui = useInterfaceStore()
const route = useRoute()
const currentPath = computed(() => route.path.replace(/\/+$/, ''))
const { t } = useI18n()
const filtered = computed(() => {
  const query = ui.query.trim().toLowerCase()
  return catalogue.filter((item) =>
    `${item.name} ${item.source} ${item.feature ?? ''} ${item.category}`
      .toLowerCase()
      .includes(query),
  )
})
const groups = computed(() =>
  ['Components', 'Examples'].map((name) => ({
    name,
    items: filtered.value.filter((item) => item.category === name),
  })),
)
watch(
  () => route.path,
  () => {
    ui.navigationOpen = false
  },
)
</script>
<template>
  <div class="component-nav-wrapper">
    <button
      class="component-nav-toggle"
      :aria-expanded="ui.navigationOpen"
      aria-controls="component-navigation"
      @click="ui.navigationOpen = !ui.navigationOpen"
    >
      <PanelLeft :size="18" />
      {{ t('components.browse') }}
      <span>{{ catalogue.length }}</span>
    </button>
    <aside
      id="component-navigation"
      class="component-sidebar"
      :class="{ 'is-open': ui.navigationOpen }"
      :aria-label="t('components.list')"
      @keydown.esc="ui.navigationOpen = false"
    >
      <label class="component-search">
        <Search :size="16" />
        <span class="sr-only">{{ t('components.search') }}</span>
        <input
          v-model="ui.query"
          type="search"
          :placeholder="t('components.search')"
          autocomplete="off"
        />
        <button
          v-if="ui.query"
          class="icon-button"
          :aria-label="t('components.clear')"
          @click="ui.query = ''"
        >
          <X :size="14" />
        </button>
      </label>
      <nav :aria-label="t('components.list')">
        <NuxtLink
          to="/components"
          class="all-gallery-link"
          :class="{ selected: currentPath === '/components' }"
          :aria-current="currentPath === '/components' ? 'page' : undefined"
        >
          <Grid2X2 :size="16" />
          {{ t('components.title') }}
        </NuxtLink>
        <div v-if="!filtered.length" class="search-empty" role="status">
          <p>{{ t('components.empty') }}</p>
          <button @click="ui.query = ''">{{ t('components.reset') }}</button>
        </div>
        <section
          v-for="group in groups.filter((group) => group.items.length)"
          :key="group.name"
          class="component-nav-group"
        >
          <h2>
            {{ t(group.name === 'Components' ? 'components.widgets' : 'components.examples') }}
            <span>{{ group.items.length }}</span>
          </h2>
          <NuxtLink
            v-for="item in group.items"
            :key="item.slug"
            :to="`/components/${item.slug}`"
            :class="{ selected: currentPath === `/components/${item.slug}` }"
            :aria-current="currentPath === `/components/${item.slug}` ? 'page' : undefined"
          >
            {{ item.name }}
            <span v-if="item.slug === 'updater'" class="nav-dot" />
          </NuxtLink>
        </section>
      </nav>
    </aside>
  </div>
</template>
