import './browser-shortcuts.js'

const status = document.querySelector('#status')
const component = new URLSearchParams(location.search).get('component')
const notify = (state) => parent.postMessage({ type: 'argui-gallery', state }, location.origin)
let observer
let timer
let rendererReady = false
let finished = false
let failed = false

function fail() {
  finished = true
  failed = true
  clearTimeout(timer)
  observer?.disconnect()
  status.hidden = false
  status.replaceChildren()
  const message = document.createElement('p')
  message.textContent = 'The WebGPU gallery could not start in this browser.'
  const link = document.createElement('a')
  link.href = '/components'
  link.target = '_top'
  link.textContent = 'Explore the components and Rust source'
  status.append(message, link)
  notify('error')
}

function ready() {
  if (!rendererReady || finished) return
  const target = component
    ? [...document.querySelectorAll('button[aria-label]')].find(
        (button) => button.getAttribute('aria-label') === component,
      )
    : document.querySelector('input[aria-label="Search components"]')
  if (!target) return
  if (component) target.click()
  finished = true
  observer.disconnect()
  clearTimeout(timer)
  requestAnimationFrame(() =>
    requestAnimationFrame(() => {
      if (failed) return
      status.hidden = true
      notify('ready')
    }),
  )
}

// RendererReady comes from Argui's runtime, after asynchronous GPU initialization.
addEventListener('argui:renderer-state', (event) => {
  if (event.detail?.state === 'error') return fail()
  if (event.detail?.state === 'ready') {
    rendererReady = true
    ready()
  }
})
addEventListener('error', fail, { once: true })
addEventListener('unhandledrejection', fail, { once: true })

try {
  if (!navigator.gpu || !(await navigator.gpu.requestAdapter()))
    throw new Error('WebGPU unavailable')
  observer = new MutationObserver(ready)
  observer.observe(document.body, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['aria-label'],
  })
  timer = setTimeout(fail, 90_000)
  const { default: init } = await import('./pkg/argui_widget_gallery.js')
  await init()
  if (!rendererReady && !finished) {
    status.textContent = 'Starting the Argui renderer…'
    notify('loading')
  }
  ready()
} catch (error) {
  console.error(error)
  fail()
}
