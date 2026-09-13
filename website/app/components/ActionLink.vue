<script setup lang="ts">
import { ArrowRight, ArrowUpRight } from '@lucide/vue'
withDefaults(
  defineProps<{ to: string; variant?: 'primary' | 'secondary' | 'text'; external?: boolean }>(),
  { variant: 'primary', external: false },
)
</script>
<template>
  <NuxtLink
    :to="to"
    :external="external"
    :target="external ? '_blank' : undefined"
    :rel="external ? 'noopener noreferrer' : undefined"
    class="action-link"
    :class="`action-${variant}`"
  >
    <slot />
    <ArrowUpRight v-if="external" :size="17" />
    <ArrowRight v-else :size="17" />
  </NuxtLink>
</template>
