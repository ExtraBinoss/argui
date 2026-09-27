<script setup lang="ts">
import type { Adapter, ComponentSlug } from '~/data/components'
import { useSearchStore } from '~/stores/search'

const props = defineProps<{ adapter: Adapter; component?: ComponentSlug; example?: 'custom-elements' | 'counter'; lazy?: boolean }>()
const config = useRuntimeConfig()
const search = useSearchStore()
const frame = ref<HTMLIFrameElement | null>(null)
const frameComponent = ref(props.component ?? 'button')
const canRender = ref(false)
const status = ref<'loading' | 'ready' | 'error'>('loading')
const message = ref('')
let startupTimer: ReturnType<typeof setTimeout> | undefined
let canvasCheck: ReturnType<typeof setInterval> | undefined
let frameDocument: Document | null = null
const source = computed(() => `${config.app.baseURL}gallery/${props.adapter}/index.html?${props.example ? `example=${props.example}` : `component=${frameComponent.value}`}`)

watch(() => props.adapter, () => { clearTimeout(startupTimer); clearInterval(canvasCheck); frameComponent.value = props.component ?? 'button'; status.value = canRender.value ? 'loading' : 'error' })
watch(() => props.component, component => {
  if (component && !props.example && import.meta.client) frame.value?.contentWindow?.postMessage({ type: 'argui:navigate', component }, window.location.origin)
})

function receive(event: MessageEvent) {
  if (event.origin !== window.location.origin || event.source !== frame.value?.contentWindow) return
  if (event.data?.type === 'argui:mounted') { clearTimeout(startupTimer); clearInterval(canvasCheck); status.value = 'ready' }
  if (event.data?.type === 'argui:error') {
    clearTimeout(startupTimer)
    clearInterval(canvasCheck)
    status.value = 'error'
    message.value = String(event.data.message ?? 'The renderer could not start.')
  }
}

function frameShortcut(event: KeyboardEvent) {
  if (event.altKey || !(event.ctrlKey || event.metaKey)) return
  if (event.key.toLowerCase() !== 'f' && event.key.toLowerCase() !== 'k') return
  event.preventDefault()
  search.show()
}

function frameLoaded() {
  frameDocument?.removeEventListener('keydown', frameShortcut)
  frameDocument = frame.value?.contentDocument ?? null
  frameDocument?.addEventListener('keydown', frameShortcut)
  if (status.value !== 'loading') return
  clearTimeout(startupTimer)
  clearInterval(canvasCheck)
  canvasCheck = setInterval(() => {
    const root = frame.value?.contentDocument?.getElementById('argui-root')
    if (root?.querySelector('canvas')) {
      clearTimeout(startupTimer)
      clearInterval(canvasCheck)
      status.value = 'ready'
    } else if (root?.textContent?.startsWith('Argui could not')) {
      clearTimeout(startupTimer)
      clearInterval(canvasCheck)
      status.value = 'error'
      message.value = root.textContent
    }
  }, 250)
  startupTimer = setTimeout(() => {
    clearInterval(canvasCheck)
    if (status.value === 'loading') {
      status.value = 'error'
      message.value = 'The WebAssembly gallery did not finish starting.'
    }
  }, 30000)
}

onMounted(() => {
  window.addEventListener('message', receive)
  if (!('gpu' in navigator) || !navigator.gpu) {
    status.value = 'error'
    message.value = 'This browser does not expose WebGPU, which the Argui Rust renderer needs.'
    return
  }
  canRender.value = true
})
onUnmounted(() => { clearTimeout(startupTimer); clearInterval(canvasCheck); frameDocument?.removeEventListener('keydown', frameShortcut); window.removeEventListener('message', receive) })
</script>

<template>
  <div class="gallery-window">
    <div class="gallery-window-bar">
      <div class="window-dots" aria-hidden="true"><i /><i /><i /></div>
      <span>Argui gallery <span class="window-separator">/</span> {{ adapter === 'solid' ? 'Solid' : 'React' }}</span>
      <span class="window-live"><span class="live-dot" /> Live WASM</span>
    </div>
    <div class="gallery-viewport">
      <iframe v-if="canRender" :key="adapter" ref="frame" :src="source" :loading="lazy ? 'lazy' : 'eager'" :title="`Argui ${adapter} ${example ?? 'component gallery'}`" @load="frameLoaded" />
      <div v-if="status === 'loading'" class="gallery-loading" role="status"><span class="spinner" /> Starting the Rust renderer…</div>
      <div v-else-if="status === 'error'" class="gallery-failure" role="alert">
        <strong>Gallery unavailable in this browser</strong>
        <p>{{ message }}</p>
        <p>Open this page in a browser with WebGPU enabled. You can also read the <NuxtLink to="/docs/ui/custom-components">custom component guide</NuxtLink>.</p>
        <a :href="source" target="_blank" rel="noopener noreferrer">Open the gallery in a new tab ↗</a>
      </div>
    </div>
  </div>
</template>
