<script setup lang="ts">
import { ArrowUpRight, Check, Copy, RotateCw, TriangleAlert } from '@lucide/vue'

const props = defineProps<{
  issue: {
    reason?: string
    origin?: string
    browser?: string
    os?: string
  } | null
}>()
defineEmits<{ retry: [] }>()

const { t } = useI18n()
const browser = computed(() => props.issue?.browser ?? t('gallery.browser'))
const os = computed(() => props.issue?.os ?? t('gallery.device'))
const insecure = computed(() => props.issue?.reason === 'insecure')
const scheme = computed(() =>
  browser.value === 'Brave' ? 'brave' : browser.value === 'Edge' ? 'edge' : 'chrome',
)
const setting = computed(() => {
  if (browser.value === 'Safari') return 'Use an HTTPS address or Google Chrome'
  if (browser.value === 'Firefox') return 'about:config'
  return insecure.value
    ? `${scheme.value}://flags/#unsafely-treat-insecure-origin-as-secure`
    : `${scheme.value}://gpu`
})
const browserUrl = computed(() =>
  os.value === 'Android'
    ? 'https://play.google.com/store/apps/details?id=com.android.chrome'
    : 'https://www.google.com/chrome/',
)
const copied = ref<'setting' | 'origin' | null>(null)

async function copyValue(value: string, key: 'setting' | 'origin') {
  let success = false
  if (navigator.clipboard && isSecureContext) {
    try {
      await navigator.clipboard.writeText(value)
      success = true
    } catch {
      success = false
    }
  }
  if (!success) {
    const input = document.createElement('textarea')
    input.value = value
    input.readOnly = true
    input.style.cssText = 'position:fixed;inset:-9999px auto auto -9999px;opacity:0'
    document.body.append(input)
    input.focus()
    input.select()
    input.setSelectionRange(0, value.length)
    success = document.execCommand('copy')
    input.remove()
  }
  copied.value = success ? key : null
  if (success) setTimeout(() => (copied.value = null), 1_500)
}
</script>

<template>
  <section class="webgpu-help-card" aria-labelledby="webgpu-help-title">
    <TriangleAlert :size="27" />
    <div class="webgpu-help-heading">
      <h2 id="webgpu-help-title">
        {{
          t(insecure ? 'gallery.blockedOn' : 'gallery.unavailableTitle', {
            browser,
            os,
          })
        }}
      </h2>
      <p>
        {{
          insecure
            ? t('gallery.insecure', {
                origin: issue?.origin ?? t('gallery.thisOrigin'),
                browser,
              })
            : t('gallery.unavailableOn', { browser, os })
        }}
      </p>
    </div>

    <ol class="webgpu-help-steps">
      <li>
        <span>1</span>
        <div>
          <strong>{{ t(insecure ? 'gallery.openSetting' : 'gallery.inspectGpu') }}</strong>
          <p>{{ t('gallery.internalAddress') }}</p>
          <div class="browser-setting">
            <code>{{ setting }}</code>
            <button type="button" @click="copyValue(setting, 'setting')">
              <Check v-if="copied === 'setting'" :size="14" />
              <Copy v-else :size="14" />
              {{ copied === 'setting' ? t('gallery.copied') : t('gallery.copySetting') }}
            </button>
          </div>
        </div>
      </li>
      <li v-if="insecure">
        <span>2</span>
        <div>
          <strong>{{ t('gallery.addOrigin') }}</strong>
          <div class="browser-setting">
            <code>{{ issue?.origin ?? t('gallery.thisOrigin') }}</code>
            <button type="button" @click="copyValue(issue?.origin ?? '', 'origin')">
              <Check v-if="copied === 'origin'" :size="14" />
              <Copy v-else :size="14" />
              {{ copied === 'origin' ? t('gallery.copied') : t('gallery.copySetting') }}
            </button>
          </div>
        </div>
      </li>
      <li>
        <span>{{ insecure ? 3 : 2 }}</span>
        <div>
          <strong>{{ t('gallery.restart', { browser }) }}</strong>
        </div>
      </li>
    </ol>

    <div class="webgpu-help-actions">
      <a :href="browserUrl" target="_blank" rel="noopener noreferrer">
        {{ t('gallery.chrome') }}
        <ArrowUpRight :size="14" />
      </a>
      <button class="action-link action-secondary" type="button" @click="$emit('retry')">
        <RotateCw :size="15" />
        {{ t('gallery.retry') }}
      </button>
    </div>
  </section>
</template>
