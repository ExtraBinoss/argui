import './browser-shortcuts.js'

const status = document.querySelector('#status')
const modulePath = document.body.dataset.module
const readyLabel = document.body.dataset.readyLabel
const component = new URLSearchParams(location.search).get('component')
const browser = navigator.brave
  ? 'Brave'
  : /Edg\//.test(navigator.userAgent)
    ? 'Edge'
    : /Chrome|CriOS/.test(navigator.userAgent)
      ? 'Chrome'
      : /Firefox|FxiOS/.test(navigator.userAgent)
        ? 'Firefox'
        : /Safari/.test(navigator.userAgent)
          ? 'Safari'
          : 'this browser'
const os = /Android/.test(navigator.userAgent)
  ? 'Android'
  : /iPhone|iPad|iPod/.test(navigator.userAgent)
    ? 'iOS'
    : /Windows/.test(navigator.userAgent)
      ? 'Windows'
      : /Macintosh|Mac OS/.test(navigator.userAgent)
        ? 'macOS'
        : /Linux/.test(navigator.userAgent)
          ? 'Linux'
          : 'your device'
const profile = { browser, os, mobile: /Android|iPhone|iPad|iPod/.test(navigator.userAgent) }
const browserScheme = browser === 'Brave' ? 'brave' : browser === 'Edge' ? 'edge' : 'chrome'
const flagsUrl =
  browser === 'Safari'
    ? 'Settings → Safari → Advanced → Feature Flags'
    : browser === 'Firefox'
      ? 'about:config'
      : `${browserScheme}://flags/#unsafely-treat-insecure-origin-as-secure`
const notify = (state, reason) =>
  parent.postMessage(
    { type: 'argui-preview', state, reason, origin: location.origin, ...profile },
    location.origin,
  )
const notifySelection = (component) =>
  parent.postMessage({ type: 'argui-preview', state: 'selection', component }, location.origin)
let observer
let timer
let rendererReady = false
let finished = false
let failed = false

addEventListener(
  'click',
  (event) => {
    const button =
      event.target instanceof Element ? event.target.closest('button[aria-label]') : null
    const label = button?.getAttribute('aria-label')
    if (label) notifySelection(label)
  },
  true,
)

function previewError(reason, message) {
  const error = new Error(message)
  error.previewReason = reason
  return error
}

function fail(error, reason = 'runtime') {
  if (failed) return
  failed = true
  finished = true
  clearTimeout(timer)
  observer?.disconnect()
  console.error(error)
  status.hidden = false
  status.replaceChildren()

  const heading = document.createElement('h1')
  heading.textContent =
    reason === 'insecure'
      ? `WebGPU is blocked by ${browser} on ${os}`
      : `WebGPU is unavailable in ${browser} on ${os}`
  const explanation = document.createElement('p')
  explanation.textContent =
    reason === 'insecure'
      ? `${location.origin} uses plain HTTP. GitHub Pages works because HTTPS is a secure context, while ${browser} blocks WebGPU on this local network address.`
      : `${browser} on ${os} did not expose a usable WebGPU adapter to Argui.`
  const steps = document.createElement('ol')
  const diagnostics =
    browser === 'Safari'
      ? 'Settings → Safari → Advanced → Feature Flags'
      : browser === 'Firefox'
        ? 'about:support'
        : `${browserScheme}://gpu`
  const instructions =
    reason === 'insecure'
      ? [
          `Open ${flagsUrl}.`,
          `Add this exact origin: ${location.origin}.`,
          `Restart ${browser}, then reload the page.`,
        ]
      : [
          `Update ${browser}, enable hardware acceleration and restart it.`,
          `Open ${diagnostics} and confirm that WebGPU is enabled.`,
          ...(os === 'Linux'
            ? [`Enable ${browserScheme}://flags/#enable-unsafe-webgpu if WebGPU remains blocked.`]
            : []),
        ]
  for (const value of instructions) {
    const item = document.createElement('li')
    item.textContent = value
    steps.append(item)
  }
  const chrome = document.createElement('a')
  chrome.href = 'https://www.google.com/chrome/'
  chrome.target = '_blank'
  chrome.rel = 'noopener noreferrer'
  chrome.textContent = 'Download Google Chrome'
  status.append(heading, explanation, steps, chrome)
  notify('error', reason)
}

function ready() {
  if (!rendererReady || finished) return
  const targetLabel = component || readyLabel
  const target = targetLabel
    ? [...document.querySelectorAll('[aria-label]')].find(
        (element) => element.getAttribute('aria-label') === targetLabel,
      )
    : document.body
  if (!target) return
  if (component && target instanceof HTMLElement) target.click()
  finished = true
  observer?.disconnect()
  clearTimeout(timer)
  requestAnimationFrame(() =>
    requestAnimationFrame(() => {
      status.hidden = true
      notify('ready')
    }),
  )
}

addEventListener('argui:renderer-state', (event) => {
  if (event.detail?.state === 'error') return fail(new Error('Argui renderer failed'), 'runtime')
  if (event.detail?.state === 'ready') {
    rendererReady = true
    ready()
  }
})
addEventListener('error', (event) => fail(event.error ?? event, 'runtime'), { once: true })
addEventListener('unhandledrejection', (event) => fail(event.reason, 'runtime'), { once: true })

try {
  if (!isSecureContext) throw previewError('insecure', 'WebGPU requires HTTPS or localhost')
  if (!navigator.gpu) throw previewError('missing-api', 'navigator.gpu is unavailable')
  if (!(await navigator.gpu.requestAdapter()))
    throw previewError('no-adapter', 'No WebGPU adapter is available')
  if (!modulePath) throw previewError('config', 'The preview module is not configured')
  observer = new MutationObserver(ready)
  observer.observe(document.body, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['aria-label'],
  })
  timer = setTimeout(() => fail(new Error('Argui renderer startup timed out'), 'timeout'), 90_000)
  const { default: init } = await import(modulePath)
  await init()
  if (!rendererReady && !finished) {
    status.textContent = 'Starting the Argui renderer…'
    notify('loading')
  }
  ready()
} catch (error) {
  fail(error, error.previewReason)
}
