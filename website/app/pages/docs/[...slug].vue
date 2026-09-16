<script setup lang="ts">
import { ArrowLeft, ArrowRight, CheckCircle2, Clock3, ExternalLink, PlayCircle } from '@lucide/vue'
import { docs, findDoc } from '~/data/docs'
import { sourceUrl } from '~/data/project'

definePageMeta({
  validate: (route) => {
    const value = Array.isArray(route.params.slug)
      ? route.params.slug.join('/')
      : String(route.params.slug ?? '')
    return Boolean(findDoc(value))
  },
})
const route = useRoute()
const slug = computed(() =>
  Array.isArray(route.params.slug) ? route.params.slug.join('/') : String(route.params.slug),
)
const guide = computed(() => findDoc(slug.value))
const position = computed(() => docs.findIndex((item) => item.slug === slug.value))
const previous = computed(() => docs[position.value - 1])
const next = computed(() => docs[position.value + 1])
usePageSeo(
  () => guide.value?.title ?? 'Documentation',
  () => guide.value?.description ?? '',
)
</script>

<template>
  <article v-if="guide" id="main-content" class="docs-content docs-guide">
    <nav class="docs-breadcrumb" aria-label="Breadcrumb">
      <NuxtLink to="/docs">Docs</NuxtLink>
      <span>/</span>
      <span>{{ guide.category }}</span>
    </nav>
    <header class="docs-guide-heading">
      <div class="docs-guide-meta">
        <span>{{ guide.level }}</span>
        <span>
          <Clock3 :size="13" />
          {{ guide.minutes }} min
        </span>
      </div>
      <h1>{{ guide.title }}</h1>
      <p>{{ guide.description }}</p>
    </header>

    <nav class="docs-toc" aria-label="On this page">
      <strong>On this page</strong>
      <a v-for="section in guide.sections" :key="section.id" :href="`#${section.id}`">
        {{ section.title }}
      </a>
      <a href="#live-example">Exact-source WebAssembly example</a>
    </nav>

    <section
      v-for="section in guide.sections"
      :id="section.id"
      :key="section.id"
      class="docs-section"
    >
      <h2>{{ section.title }}</h2>
      <p v-for="paragraph in section.paragraphs" :key="paragraph">{{ paragraph }}</p>
      <ul v-if="section.bullets">
        <li v-for="item in section.bullets" :key="item">
          <CheckCircle2 :size="17" />
          {{ item }}
        </li>
      </ul>
      <CodeBlock v-if="section.code" :filename="section.code.filename" :code="section.code.code" />
      <div v-if="section.table" class="docs-table-wrap">
        <table>
          <thead>
            <tr>
              <th v-for="header in section.table.headers" :key="header" scope="col">
                {{ header }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(row, rowIndex) in section.table.rows" :key="rowIndex">
              <td v-for="(cell, cellIndex) in row" :key="cellIndex">{{ cell }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <aside v-if="section.note" class="docs-note">
        <strong>Good to know</strong>
        <p>{{ section.note }}</p>
      </aside>
    </section>

    <section class="docs-sources" aria-labelledby="source-heading">
      <h2 id="source-heading">Reference files</h2>
      <p>These are the implementation and guide files used for this chapter.</p>
      <a
        v-for="source in guide.sources"
        :key="source"
        :href="sourceUrl(source)"
        target="_blank"
        rel="noopener noreferrer"
      >
        <code>{{ source }}</code>
        <ExternalLink :size="15" />
      </a>
    </section>

    <section id="live-example" class="docs-live-example" aria-labelledby="live-heading">
      <div class="docs-live-heading">
        <div>
          <span>
            <PlayCircle :size="15" />
            Compiled example
          </span>
          <h2 id="live-heading">{{ guide.demoTitle }}</h2>
          <p>
            The Rust file below is imported verbatim by this page and compiled into the WebAssembly
            application running underneath it.
          </p>
        </div>
        <a :href="sourceUrl(guide.example.path)" target="_blank" rel="noopener noreferrer">
          Open exact source
          <ExternalLink :size="15" />
        </a>
      </div>
      <CodeBlock :filename="guide.example.path" :code="guide.example.source" />
      <DocsExampleFrame :example="guide.example.id" :title="guide.demoTitle" />
    </section>

    <nav class="docs-pagination" aria-label="Previous and next guides">
      <NuxtLink v-if="previous" :to="`/docs/${previous.slug}`">
        <ArrowLeft :size="17" />
        <span>
          <small>Previous</small>
          {{ previous.title }}
        </span>
      </NuxtLink>
      <span v-else />
      <NuxtLink v-if="next" :to="`/docs/${next.slug}`">
        <span>
          <small>Next</small>
          {{ next.title }}
        </span>
        <ArrowRight :size="17" />
      </NuxtLink>
    </nav>
  </article>
</template>
