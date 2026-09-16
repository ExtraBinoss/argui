<script setup lang="ts">
import { ArrowUpRight, LoaderCircle, Play, RotateCw, Square, TriangleAlert } from '@lucide/vue'
import WebGpuHelp from './WebGpuHelp.vue'

const props = defineProps<{ example: string; title: string }>()
const asset = usePublicAsset()
const { t } = useI18n()
const frame = ref<HTMLIFrameElement>()
const state = ref<'idle' | 'loading' | 'ready' | 'error' | 'slow'>('loading')
const attempt = ref(0)
const issue = ref<{
  reason?: string
  origin?: string
  browser?: string
  os?: string
  mobile?: boolean
} | null>(null)
const url = computed(
  () => `${asset('examples/docs/index.html')}?example=${encodeURIComponent(props.example)}`,
)
let timer: ReturnType<typeof setTimeout> | undefined

function launch() {
  attempt.value++
  state.value = 'loading'
  issue.value = null
  clearTimeout(timer)
  timer = setTimeout(() => {
    if (state.value === 'loading') state.value = 'slow'
  }, 45_000)
}
function stop() {
  state.value = 'idle'
  clearTimeout(timer)
}
function receive(event: MessageEvent) {
  if (
    event.origin !== location.origin ||
    event.source !== frame.value?.contentWindow ||
    event.data?.type !== 'argui-preview'
  )
    return
  if (event.data.state === 'ready' || event.data.state === 'error') {
    clearTimeout(timer)
    state.value = event.data.state
    if (event.data.state === 'error') issue.value = event.data
  }
}
onMounted(() => {
  window.addEventListener('message', receive)
  launch()
})
onBeforeUnmount(() => {
  clearTimeout(timer)
  window.removeEventListener('message', receive)
})
watch(() => props.example, launch)
</script>

<template>
  <div class="gallery-shell docs-example-frame">
    <div class="gallery-toolbar">
      <span class="gallery-status">
        <span class="status-dot" :class="{ live: state === 'ready' }" />
        {{ title }}
      </span>
      <div>
        <button
          v-if="state !== 'idle'"
          class="icon-button"
          :aria-label="t('gallery.stop')"
          @click="stop"
        >
          <Square :size="14" />
        </button>
        <a :href="url" target="_blank" rel="noopener noreferrer" :aria-label="t('gallery.newTab')">
          <ArrowUpRight :size="18" />
        </a>
      </div>
    </div>
    <div class="gallery-stage" :aria-busy="state === 'loading' || state === 'slow'">
      <iframe
        v-if="attempt > 0 && state !== 'idle' && state !== 'error'"
        :key="attempt"
        ref="frame"
        :src="url"
        :title="title"
        @error="state = 'error'"
      />
      <div
        v-if="state !== 'ready'"
        class="gallery-launch"
        :class="{ 'gallery-launch-error': state === 'error' }"
        role="status"
      >
        <template v-if="state === 'idle'">
          <button class="action-link action-primary" @click="launch">
            <Play :size="15" />
            Run this exact source
          </button>
        </template>
        <template v-else-if="state === 'loading'">
          <LoaderCircle class="spin" :size="28" />
          <p>Starting this lesson’s WebAssembly module…</p>
        </template>
        <template v-else-if="state === 'slow'">
          <TriangleAlert :size="28" />
          <p>{{ t('gallery.timeout') }}</p>
          <button class="action-link action-secondary" @click="launch">
            <RotateCw :size="15" />
            {{ t('gallery.retry') }}
          </button>
        </template>
        <WebGpuHelp v-else :issue="issue" @retry="launch" />
      </div>
    </div>
  </div>
</template>
