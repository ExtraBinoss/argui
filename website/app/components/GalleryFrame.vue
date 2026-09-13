<script setup lang="ts">
import { ArrowUpRight, Box, LoaderCircle, Play, RotateCw, Square, TriangleAlert } from '@lucide/vue'
const props = defineProps<{ component?: string | null }>()
const asset = usePublicAsset()
const { t } = useI18n()
const frame = ref<HTMLIFrameElement>()
const state = ref<'idle' | 'loading' | 'ready' | 'error' | 'slow'>('loading')
const attempt = ref(0)
const phase = ref<'downloading' | 'loading'>('downloading')
const url = computed(
  () =>
    `${asset('gallery/index.html')}${props.component ? `?component=${encodeURIComponent(props.component)}` : ''}`,
)
let timer: ReturnType<typeof setTimeout> | undefined
function launch() {
  attempt.value++
  state.value = 'loading'
  phase.value = 'downloading'
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
    event.data?.type !== 'argui-gallery'
  )
    return
  if (event.data.state === 'loading') phase.value = 'loading'
  if (event.data.state === 'ready' || event.data.state === 'error') {
    clearTimeout(timer)
    state.value = event.data.state
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
watch(() => props.component, launch)
</script>
<template>
  <div class="gallery-shell">
    <div class="gallery-toolbar">
      <span class="gallery-status">
        <span class="status-dot" :class="{ live: state === 'ready' }" />
        {{ state === 'ready' ? t('gallery.ready') : 'Argui Widget Gallery' }}
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
        :title="t('gallery.label')"
        :style="{ visibility: state === 'ready' ? 'visible' : 'hidden' }"
        @error="state = 'error'"
      />
      <div v-if="state !== 'ready'" class="gallery-launch" role="status">
        <template v-if="state === 'idle'">
          <Box :size="38" :stroke-width="1.2" />
          <button class="action-link action-primary" @click="launch">
            <Play :size="15" />
            {{
              component ? t('gallery.launchComponent', { name: component }) : t('gallery.launch')
            }}
          </button>
          <p>{{ t('gallery.hint') }}</p>
        </template>
        <template v-else-if="state === 'loading'">
          <LoaderCircle class="spin" :size="28" />
          <p>{{ t(`gallery.${phase}`) }}</p>
        </template>
        <template v-else>
          <TriangleAlert :size="28" />
          <p>{{ t(state === 'slow' ? 'gallery.timeout' : 'gallery.unsupported') }}</p>
          <p class="gallery-fallback">{{ t('gallery.fallback') }}</p>
          <button class="action-link action-secondary" @click="launch">
            <RotateCw :size="15" />
            {{ t('gallery.retry') }}
          </button>
        </template>
      </div>
    </div>
  </div>
  <p v-if="state === 'ready'" class="gallery-scroll-hint">{{ t('gallery.scroll') }}</p>
</template>
