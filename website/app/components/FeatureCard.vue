<script setup lang="ts">
import type { Component } from 'vue'
import { ArrowUpRight } from '@lucide/vue'
defineProps<{ icon: Component; title: string; body: string; href: string }>()
</script>
<template>
  <a class="feature-card" :href="href" target="_blank" rel="noopener noreferrer">
    <div class="feature-card-icon">
      <component :is="icon" :size="24" :stroke-width="1.5" />
      <ArrowUpRight :size="17" />
    </div>
    <h3>{{ title }}</h3>
    <p>{{ body }}</p>
  </a>
</template>
