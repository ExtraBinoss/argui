<script setup lang="ts">
import page from '~/data/doc-pages/overview.json'
useSeoMeta({ title: 'Documentation', description: 'Current Argui guides for getting started, widgets, layout, desktop services, the Rust runtime, and rendering.' })
</script>

<template><DocArticle :page="page" /></template>
