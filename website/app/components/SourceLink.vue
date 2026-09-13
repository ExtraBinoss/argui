<script setup lang="ts">
import { FileCode, ArrowUpRight } from '@lucide/vue'
import { sourceUrl } from '~/data/project'
defineProps<{ path: string }>()
const { t } = useI18n()
</script>
<template>
  <a
    class="source-link"
    :href="sourceUrl(path)"
    target="_blank"
    rel="noopener noreferrer"
    :aria-label="`${t('components.source')}: ${path}`"
  >
    <FileCode :size="15" />
    <code>{{ path }}</code>
    <ArrowUpRight :size="14" />
  </a>
</template>
