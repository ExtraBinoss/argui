<script setup lang="ts">
import { Check, Copy } from '@lucide/vue'
const props = defineProps<{ code: string; filename: string }>()
const { t } = useI18n()
const copied = ref(false)
const failed = ref(false)
let timeout: ReturnType<typeof setTimeout> | undefined
async function copy() {
  try {
    await navigator.clipboard.writeText(props.code)
    copied.value = true
    failed.value = false
    clearTimeout(timeout)
    timeout = setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch {
    failed.value = true
  }
}
onBeforeUnmount(() => clearTimeout(timeout))
</script>
<template>
  <div class="code-block">
    <div class="code-header">
      <span>{{ filename }}</span>
      <button
        class="copy-button"
        :aria-label="t(copied ? 'code.copied' : 'code.copy')"
        @click="copy"
      >
        <Check v-if="copied" :size="14" />
        <Copy v-else :size="14" />
        <span>{{ t(copied ? 'code.copied' : 'code.copy') }}</span>
      </button>
    </div>
    <pre tabindex="0"><code>{{ code }}</code></pre>
    <p v-if="failed" class="copy-error" role="status">{{ t('code.failed') }}</p>
    <span class="sr-only" role="status">{{ copied ? t('code.copied') : '' }}</span>
  </div>
</template>
