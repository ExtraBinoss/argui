<script setup lang="ts">
import { useGithubStore } from '~/stores/github'

const github = useGithubStore()
</script>

<template>
  <span class="github-stars"><SiteIcon name="star" /><span>{{ github.stars === null ? '—' : github.stars.toLocaleString('en-US') }}</span></span>
</template>
