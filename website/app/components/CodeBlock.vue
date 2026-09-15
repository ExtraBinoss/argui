<script setup lang="ts">
import { Check, Copy } from '@lucide/vue'
import { highlightCode } from '~/utils/highlight'
const props = defineProps<{ code: string; filename: string }>()
const { t } = useI18n()
const copied = ref(false)
const failed = ref(false)
let timeout: ReturnType<typeof setTimeout> | undefined
const highlighted = ref('')
let highlightVersion = 0
function language(filename: string) {
  if (filename.endsWith('.rs')) return 'rust'
  if (filename.endsWith('.toml')) return 'toml'
  if (filename === 'Terminal') return 'shellscript'
  return 'shellscript'
}
async function highlight() {
  const version = ++highlightVersion
  const html = await highlightCode(props.code, language(props.filename))
  if (version === highlightVersion) highlighted.value = html
}
await highlight()
watch(() => [props.code, props.filename], highlight)
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
    <div class="highlighted-code" tabindex="0" v-html="highlighted" />
    <p v-if="failed" class="copy-error" role="status">{{ t('code.failed') }}</p>
    <span class="sr-only" role="status">{{ copied ? t('code.copied') : '' }}</span>
  </div>
</template>
