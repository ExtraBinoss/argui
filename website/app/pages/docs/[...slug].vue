<script setup lang="ts">
type DocPage = { slug: string; title: string; section: string; source: string; html: string }
const pages = import.meta.glob<{ default: DocPage }>('../../data/doc-pages/**/*.json')
const route = useRoute()
const slug = (Array.isArray(route.params.slug) ? route.params.slug.join('/') : String(route.params.slug)).replace(/^\/+|\/+$/g, '')
const load = pages[`../../data/doc-pages/${slug}.json`]
if (!load) throw createError({ statusCode: 404, statusMessage: 'Guide not found' })
const page = (await load()).default
useSeoMeta({ title: page.title, description: `${page.title} in the Argui documentation.` })
</script>

<template><DocArticle :page="page" /></template>
