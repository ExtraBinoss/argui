import assert from 'node:assert/strict'
import { spawn, spawnSync } from 'node:child_process'
import { access, mkdir, mkdtemp, readFile, rm, stat, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Run through scripts/linux-hidden-display.sh.')
assert.equal(process.env.DISPLAY, undefined, 'The user desktop display must stay unused.')
const website = fileURLToPath(new URL('../', import.meta.url))
const output = resolve(website, 'test-results')
const origin = process.env.WEBSITE_URL ?? 'http://127.0.0.1:3100'
const basePath = new URL(origin).pathname.replace(/\/$/, '')
const pathFor = path => `${basePath}${path}`
await mkdir(output, { recursive: true })
const profile = await mkdtemp(join(tmpdir(), 'argui-site-chrome-'))
const chrome = spawn(process.env.CHROME_PATH ?? 'google-chrome', [
  '--no-sandbox', '--disable-dev-shm-usage', '--disable-gpu-sandbox',
  '--no-first-run', '--password-store=basic', '--disable-background-networking',
  '--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist',
  '--enable-features=Vulkan', '--use-angle=vulkan', '--remote-debugging-port=0',
  '--remote-allow-origins=*', `--user-data-dir=${profile}`, 'about:blank',
], { stdio: ['ignore', 'ignore', 'pipe'] })
let chromeErrors = ''
chrome.stderr.on('data', chunk => { chromeErrors = (chromeErrors + chunk.toString()).slice(-8000) })

async function eventually(task, timeout = 60000) {
  const end = Date.now() + timeout
  let lastError
  while (Date.now() < end) {
    try {
      const result = await task()
      if (result) return result
    } catch (error) { lastError = error }
    await new Promise(done => setTimeout(done, 200))
  }
  throw new Error(`Timed out waiting for browser state: ${lastError ?? 'condition unmet'}\n${chromeErrors}`)
}

class DevTools {
  constructor(socket) {
    this.socket = socket
    this.nextId = 0
    this.pending = new Map()
    socket.addEventListener('message', event => {
      const data = JSON.parse(event.data)
      if (!data.id) return
      const call = this.pending.get(data.id)
      if (!call) return
      this.pending.delete(data.id)
      if (data.error) call.reject(new Error(data.error.message))
      else call.resolve(data.result)
    })
  }
  send(method, params = {}) {
    const id = ++this.nextId
    return new Promise((resolveCall, rejectCall) => {
      this.pending.set(id, { resolve: resolveCall, reject: rejectCall })
      this.socket.send(JSON.stringify({ id, method, params }))
    })
  }
  async evaluate(expression) {
    const result = await this.send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true })
    if (result.exceptionDetails) throw new Error(result.exceptionDetails.text)
    return result.result.value
  }
}

let devtools
try {
  const portFile = join(profile, 'DevToolsActivePort')
  const port = await eventually(async () => {
    await access(portFile)
    return Number((await readFile(portFile, 'utf8')).split('\n')[0])
  })
  const target = await eventually(async () => {
    const pages = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json()
    return pages.find(page => page.type === 'page')
  })
  const socket = new WebSocket(target.webSocketDebuggerUrl)
  await new Promise((done, fail) => { socket.addEventListener('open', done, { once: true }); socket.addEventListener('error', fail, { once: true }) })
  devtools = new DevTools(socket)
  await devtools.send('Page.enable')
  await devtools.send('Runtime.enable')
  await devtools.send('Page.addScriptToEvaluateOnNewDocument', { source: `
    const siteFetch = window.fetch.bind(window)
    window.fetch = (input, options) => String(input) === 'https://api.github.com/repos/ExtraBinoss/argui'
      ? Promise.resolve(new Response(JSON.stringify({ stargazers_count: 1234 }), { status: 200, headers: { 'Content-Type': 'application/json' } }))
      : siteFetch(input, options)
  ` })

  async function viewport(width, height) {
    await devtools.send('Emulation.setDeviceMetricsOverride', { width, height, deviceScaleFactor: 1, mobile: false })
  }
  async function navigate(path, expected) {
    await devtools.send('Page.navigate', { url: `${origin}${path}` })
    await eventually(() => devtools.evaluate(`document.readyState === 'complete' && document.body?.textContent?.includes(${JSON.stringify(expected)})`))
  }
  async function capture(name, resetScroll = true, minimumBytes = 15000) {
    if (resetScroll) await devtools.evaluate('scrollTo(0, 0)')
    await new Promise(done => setTimeout(done, 700))
    const image = await devtools.send('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false })
    const path = resolve(output, `${name}.png`)
    await writeFile(path, Buffer.from(image.data, 'base64'))
    assert.ok((await stat(path)).size > minimumBytes, `Blank capture: ${name}`)
    assert.equal(await devtools.evaluate('document.documentElement.scrollWidth > innerWidth'), false, `Horizontal overflow: ${name}`)
    console.log(`[browser] ${name}: ${path}`)
  }

  await viewport(1440, 1000)
  await navigate('/', 'Build interfaces')
  assert.match(await devtools.evaluate('document.title'), /Argui/)
  await eventually(() => devtools.evaluate("document.querySelector('.desktop-nav .github-stars')?.textContent?.includes('1,234')"))
  assert.equal(await devtools.evaluate("document.querySelector('.footer-links .github-stars')?.textContent?.includes('1,234')"), true)
  assert.equal(await devtools.evaluate("document.querySelectorAll('a[href=\"https://discord.gg/xY9CWSc65\"]').length >= 2"), true)
  assert.equal(await devtools.evaluate("!!document.querySelector('.desktop-nav .social-nav-link svg')"), true)
  await capture('home-desktop')
  await devtools.evaluate("window.dispatchEvent(new KeyboardEvent('keydown', { key: 'f', ctrlKey: true, bubbles: true, cancelable: true }))")
  await eventually(() => devtools.evaluate("document.querySelector('.site-search-dialog')?.open && document.activeElement?.matches('.site-search-field input')"))
  await devtools.evaluate("(() => { const input = document.querySelector('.site-search-field input'); input.value = 'custom components'; input.dispatchEvent(new Event('input', { bubbles: true })) })()")
  await eventually(() => devtools.evaluate("document.querySelector('.site-search-result')?.textContent?.includes('Custom components and state') && !document.querySelector('.site-search-caption')?.textContent?.includes('Loading')"))
  assert.ok((await devtools.evaluate("document.querySelector('.site-search-result')?.getAttribute('href')")).startsWith(pathFor('/docs/ui/custom-components#')))
  await devtools.evaluate("document.querySelector('.site-search-field input')?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, cancelable: true }))")
  assert.equal(await devtools.evaluate("document.querySelectorAll('.site-search-result')[1]?.classList.contains('selected')"), true)
  await devtools.evaluate("document.querySelector('.site-search-field input')?.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true, cancelable: true }))")
  assert.equal(await devtools.evaluate("document.querySelector('.site-search-result')?.classList.contains('selected')"), true)
  await capture('site-search-desktop')
  await devtools.evaluate("document.querySelector('.site-search-field input')?.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }))")
  await eventually(() => devtools.evaluate(`location.pathname === ${JSON.stringify(pathFor('/docs/ui/custom-components'))} && !document.querySelector('.site-search-dialog')?.open`))
  await navigate('/docs/cli', 'Argui CLI')
  assert.equal(await devtools.evaluate("document.querySelector('.markdown-body')?.textContent?.includes('install-cli.sh')"), true)
  await capture('docs-cli')
  await navigate('/docs/crates', 'Crates, dependencies, and features')
  assert.equal(await devtools.evaluate("document.querySelector('.markdown-body')?.textContent?.includes('argui-runtime')"), true)
  await capture('docs-crates')
  await navigate('/', 'Build interfaces')
  await devtools.evaluate("document.querySelector('.preview-section')?.scrollIntoView()")
  await eventually(() => devtools.evaluate("!document.querySelector('.preview-section .gallery-loading')"), 90000)
  const homeFailure = await devtools.evaluate("document.querySelector('.preview-section .gallery-failure')?.textContent ?? ''")
  assert.equal(homeFailure, '', `Homepage WASM preview failed: ${homeFailure}`)
  await capture('home-preview', false)

  await navigate('/components', 'Explore the gallery')
  await eventually(() => devtools.evaluate("!document.querySelector('.gallery-loading')"), 90000)
  const failure = await devtools.evaluate("document.querySelector('.gallery-failure')?.textContent ?? ''")
  assert.equal(failure, '', `WASM gallery failed: ${failure}`)
  assert.equal(await devtools.evaluate("!!document.querySelector('iframe')?.contentDocument?.querySelector('canvas')"), true, 'WASM canvas is missing')
  await capture('components-solid')
  await devtools.evaluate("document.querySelector('iframe')?.contentDocument?.dispatchEvent(new KeyboardEvent('keydown', { key: 'f', ctrlKey: true, bubbles: true, cancelable: true }))")
  await eventually(() => devtools.evaluate("document.querySelector('.site-search-dialog')?.open && document.activeElement?.matches('.site-search-field input')"))
  await devtools.evaluate("document.querySelector('.site-search-close')?.click()")
  await eventually(() => devtools.evaluate("!document.querySelector('.site-search-dialog')?.open"))
  const rect = await devtools.evaluate("(() => { const box = document.querySelector('iframe').getBoundingClientRect(); return { x: box.x, y: box.y } })()")
  const x = Math.round(rect.x + 60)
  const y = Math.round(rect.y + 166)
  await devtools.send('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y })
  await devtools.send('Input.dispatchMouseEvent', { type: 'mousePressed', x, y, button: 'left', clickCount: 1 })
  await devtools.send('Input.dispatchMouseEvent', { type: 'mouseReleased', x, y, button: 'left', clickCount: 1 })
  await capture('components-button-clicked')
  const crop = [rect.x + 20, rect.y + 410, rect.x + 210, rect.y + 455].map(Math.round)
  const difference = spawnSync('python3', ['-c', `
from PIL import Image, ImageChops
import sys
box = tuple(map(int, sys.argv[3:]))
before = Image.open(sys.argv[1]).convert('RGB').crop(box)
after = Image.open(sys.argv[2]).convert('RGB').crop(box)
print(ImageChops.difference(before, after).getbbox() is not None)
`, resolve(output, 'components-solid.png'), resolve(output, 'components-button-clicked.png'), ...crop.map(String)], { encoding: 'utf8' })
  assert.equal(difference.status, 0, difference.stderr)
  assert.equal(difference.stdout.trim(), 'True', 'Clicking the native Button did not change its visible counter')
  await devtools.evaluate("[...document.querySelectorAll('.component-list button')].find(button => button.textContent.includes('Checkbox'))?.click()")
  await eventually(() => devtools.evaluate("document.querySelector('.component-detail h2')?.textContent === 'Checkbox'"))
  await capture('components-checkbox')
  for (const adapter of ['solid', 'react']) {
    const manifest = await (await fetch(`${origin}/gallery/${adapter}/assets.generated.json`)).json()
    assert.equal(manifest.assets.some(asset => asset.key === 'tabler/check.svg'), true, `${adapter} gallery omitted the Tabler check icon`)
  }
  await devtools.evaluate("window.dispatchEvent(new KeyboardEvent('keydown', { key: '/', bubbles: true }))")
  assert.equal(await devtools.evaluate("document.activeElement?.matches('.search-box input')"), true, 'Search shortcut did not focus the field')

  const initialFrame = await devtools.evaluate("document.querySelector('iframe')?.src")
  await devtools.evaluate("[...document.querySelectorAll('.component-list button')].find(button => button.textContent.includes('Switch'))?.click()")
  await eventually(() => devtools.evaluate("document.querySelector('.component-detail h2')?.textContent === 'Switch'"))
  assert.equal(await devtools.evaluate("document.querySelector('iframe')?.src"), initialFrame, 'Component change reloaded the WASM host')
  await capture('components-switch')

  await devtools.evaluate("document.querySelector('.components-toolbar .segmented button:last-child')?.click()")
  await eventually(() => devtools.evaluate("document.querySelector('iframe')?.src.includes('/gallery/react/')"))
  await eventually(() => devtools.evaluate("!document.querySelector('.gallery-loading')"), 90000)
  const reactFailure = await devtools.evaluate("document.querySelector('.gallery-failure')?.textContent ?? ''")
  assert.equal(reactFailure, '', `React WASM gallery failed: ${reactFailure}`)
  await capture('components-react')

  await viewport(390, 844)
  await capture('components-mobile')
  assert.ok(await devtools.evaluate("document.querySelector('.gallery-viewport')?.getBoundingClientRect().width >= 300"))
  await navigate('/docs/getting-started', 'Getting started')
  await eventually(() => devtools.evaluate("document.querySelector('.markdown-body h1')?.textContent === 'Getting started'"), 15000)
  await capture('docs-mobile')
  await devtools.evaluate("document.querySelector('.docs-search-button')?.click()")
  await eventually(() => devtools.evaluate("document.querySelector('.site-search-dialog')?.open"))
  await devtools.evaluate("(() => { const input = document.querySelector('.site-search-field input'); input.value = 'virtual list'; input.dispatchEvent(new Event('input', { bubbles: true })) })()")
  await eventually(() => devtools.evaluate(`document.querySelector('.site-search-result')?.getAttribute('href') === ${JSON.stringify(pathFor('/components?component=virtual-list'))}`))
  await capture('site-search-mobile')
  await devtools.evaluate("document.querySelector('.site-search-result')?.click()")
  await eventually(() => devtools.evaluate(`location.pathname === ${JSON.stringify(pathFor('/components'))} && new URLSearchParams(location.search).get('component') === 'virtual-list' && document.querySelector('.component-detail h2')?.textContent === 'Virtual list'`))
  await navigate('/docs/ui/custom-components', 'Custom components and state')
  assert.equal(await devtools.evaluate("document.querySelectorAll('.doc-live-example').length"), 2)
  assert.ok(await devtools.evaluate("document.querySelectorAll('.code-block .shiki').length >= 4"))
  for (const [index, name] of ['custom-elements', 'counter'].entries()) {
    await devtools.evaluate(`document.querySelectorAll('.doc-live-example')[${index}]?.scrollIntoView()`)
    await eventually(() => devtools.evaluate(`!document.querySelectorAll('.doc-live-example')[${index}]?.querySelector('.gallery-loading')`), 90000)
    const failure = await devtools.evaluate(`document.querySelectorAll('.doc-live-example')[${index}]?.querySelector('.gallery-failure')?.textContent ?? ''`)
    assert.equal(failure, '', `${name} WebAssembly example failed: ${failure}`)
    assert.equal(await devtools.evaluate(`!!document.querySelectorAll('.doc-live-example')[${index}]?.querySelector('iframe')?.contentDocument?.querySelector('canvas')`), true)
    await capture(`docs-${name}`, false)
  }
  const counterFrame = await devtools.evaluate("(() => { const box = document.querySelectorAll('.doc-live-example iframe')[1].getBoundingClientRect(); return { x: box.x, y: box.y } })()")
  const counterX = Math.round(counterFrame.x + 75)
  const counterY = Math.round(counterFrame.y + 145)
  await devtools.send('Input.dispatchMouseEvent', { type: 'mouseMoved', x: counterX, y: counterY })
  await devtools.send('Input.dispatchMouseEvent', { type: 'mousePressed', x: counterX, y: counterY, button: 'left', clickCount: 1 })
  await devtools.send('Input.dispatchMouseEvent', { type: 'mouseReleased', x: counterX, y: counterY, button: 'left', clickCount: 1 })
  await capture('docs-counter-clicked', false)
  const counterCrop = [counterFrame.x + 130, counterFrame.y + 95, counterFrame.x + 300, counterFrame.y + 155].map(Math.round)
  const counterDifference = spawnSync('python3', ['-c', `
from PIL import Image, ImageChops
import sys
box = tuple(map(int, sys.argv[3:]))
before = Image.open(sys.argv[1]).convert('RGB').crop(box)
after = Image.open(sys.argv[2]).convert('RGB').crop(box)
print(ImageChops.difference(before, after).getbbox() is not None)
`, resolve(output, 'docs-counter.png'), resolve(output, 'docs-counter-clicked.png'), ...counterCrop.map(String)], { encoding: 'utf8' })
  assert.equal(counterDifference.status, 0, counterDifference.stderr)
  assert.equal(counterDifference.stdout.trim(), 'True', 'The stateful counter did not visibly update after a native click')
  await devtools.send('Page.navigate', { url: `${origin}/gallery/react/index.html?example=counter` })
  await eventually(() => devtools.evaluate("!!document.querySelector('#argui-root canvas')"), 90000)
  await capture('docs-counter-react', true, 5000)
  await devtools.send('Page.addScriptToEvaluateOnNewDocument', {
    source: "Object.defineProperty(navigator, 'gpu', { configurable: true, value: undefined })",
  })
  await navigate('/components', 'Explore the gallery')
  await eventually(() => devtools.evaluate("!!document.querySelector('.gallery-failure')"))
  assert.match(await devtools.evaluate("document.querySelector('.gallery-failure')?.textContent"), /does not expose WebGPU/)
  assert.equal(await devtools.evaluate("document.querySelector('iframe')"), null)
  await capture('components-no-webgpu')
  console.log('[browser] SSR pages, WASM adapters and docs examples, fallback, and mobile layout passed.')
} finally {
  devtools?.socket.close()
  if (chrome.exitCode === null && chrome.signalCode === null) {
    chrome.kill('SIGTERM')
    await new Promise(done => chrome.once('exit', done))
  }
  await rm(profile, { recursive: true, force: true })
}
