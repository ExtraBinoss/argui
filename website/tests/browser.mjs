import assert from 'node:assert/strict'
import { mkdir } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import { resolve } from 'node:path'

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Run through scripts/linux-hidden-display.sh.')
assert.equal(process.env.DISPLAY, undefined)
const website = fileURLToPath(new URL('../', import.meta.url))
const output = resolve(website, 'test-results')
await mkdir(output, { recursive: true })
const origin = process.env.WEBSITE_URL ?? 'http://127.0.0.1:3100'
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer')
const browser = await (imported.puppeteer ?? imported.default).launch({
  executablePath: process.env.CHROME_PATH,
  headless: false,
  args: [
    '--ozone-platform=wayland',
    '--enable-unsafe-webgpu',
    '--ignore-gpu-blocklist',
    '--enable-features=Vulkan',
    '--use-angle=vulkan',
  ],
})
const pause = (ms = 250) => new Promise((resolve) => setTimeout(resolve, ms))
try {
  const page = await browser.newPage()
  const errors = []
  const hydration = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (/hydration/i.test(message.text())) hydration.push(message.text())
  })
  await page.emulateMediaFeatures([
    { name: 'prefers-color-scheme', value: 'light' },
    { name: 'prefers-reduced-motion', value: 'reduce' },
  ])
  const screenshot = async (name) => {
    await pause(500)
    const image = await page.screenshot({ path: resolve(output, `${name}.png`), fullPage: true })
    assert.ok(image.length > 15000, `Blank capture: ${name}`)
    assert.equal(
      await page.evaluate(() => document.documentElement.scrollWidth > innerWidth),
      false,
      `Horizontal overflow: ${name}`,
    )
  }

  if (process.env.UPDATE_ASSETS === '1') {
    await page.setViewport({ width: 1120, height: 700 })
    await page.goto(`${origin}/gallery/index.html`, { waitUntil: 'networkidle0' })
    await page.waitForFunction(() => document.querySelector('#status')?.hidden, { timeout: 90_000 })
    await page.waitForSelector('button[aria-label="Primary"]')
    await pause(1500)
    const preview = await page.screenshot({
      path: resolve(website, 'public/gallery-preview.webp'),
      type: 'webp',
      quality: 90,
    })
    assert.ok(preview.length > 15000, 'Blank gallery preview')
    console.log('Captured the actual Argui renderer for the homepage.')
  }

  await page.setViewport({ width: 1440, height: 1050 })
  await page.goto(origin, { waitUntil: 'networkidle0' })
  assert.match(await page.title(), /Argui/)
  assert.equal(await page.$eval('html', (element) => element.lang), 'en')
  assert.equal(
    await page.$$('iframe').then((items) => items.length),
    0,
    'Home does not eagerly boot WASM',
  )
  assert.ok(await page.$eval('meta[name="description"]', (element) => element.content.length > 50))
  await page.waitForFunction(() => {
    const image = document.querySelector('.preview-window img')
    return image?.complete && image.naturalWidth > 0
  })
  await screenshot('home-light')
  await page.click('.theme-button')
  await page.waitForFunction(() => document.documentElement.dataset.theme === 'dark')
  await screenshot('home-dark')
  await page.reload({ waitUntil: 'networkidle0' })
  await page.waitForFunction(() => document.documentElement.dataset.theme === 'dark')
  await page.click('.theme-button')

  for (const path of ['/features', '/get-started']) {
    await page.goto(origin + path, { waitUntil: 'networkidle0' })
    await screenshot(path.slice(1))
  }
  await browser
    .defaultBrowserContext()
    .overridePermissions(origin, ['clipboard-read', 'clipboard-sanitized-write'])
  await page.bringToFront()
  await page.click('.copy-button')
  await page.waitForSelector('.copy-button[aria-label="Copied"]')
  assert.match(
    await page.evaluate(() => navigator.clipboard.readText()),
    /git clone https:\/\/github.com\/ExtraBinoss\/argui/,
  )

  await page.goto(`${origin}/components/`, { waitUntil: 'networkidle0' })
  assert.equal(
    await page.$eval('#component-navigation nav a', (element) => element.textContent.trim()),
    'All widgets gallery',
  )
  assert.equal(
    await page.$eval('.all-gallery-link', (element) => element.getAttribute('aria-current')),
    'page',
  )
  await page.type('.component-search input', 'colour-does-not-exist')
  await page.waitForSelector('.search-empty')
  await page.click('.search-empty button')
  await page.type('.component-search input', 'updater')
  assert.equal(await page.$$('.component-nav-group a').then((items) => items.length), 1)
  await page.click('.component-nav-group a')
  await page.waitForFunction(() => location.pathname === '/components/updater')
  assert.match(
    await page.$eval('.source-link', (element) => element.textContent),
    /argui-widgets\/src\/updater.rs/,
  )
  await page.waitForSelector('.component-note')
  await screenshot('component-updater')
  await page.click('.component-search button')

  await page.goto(`${origin}/components/button`, { waitUntil: 'networkidle0' })
  await page.waitForSelector('.status-dot.live', { timeout: 90_000 })
  assert.ok(
    await page.$eval(
      '.component-nav-group a',
      (element) => parseFloat(getComputedStyle(element).fontSize) >= 16,
    ),
  )
  assert.ok(
    await page.$eval(
      '.component-page-heading p',
      (element) => parseFloat(getComputedStyle(element).fontSize) >= 16,
    ),
  )
  assert.ok(
    await page.$eval(
      '.source-link',
      (element) => parseFloat(getComputedStyle(element).fontSize) >= 12,
    ),
  )
  const iframe = await page.$('iframe')
  const frame = await iframe.contentFrame()
  await frame.waitForSelector('button[aria-label="Primary"]')
  const canvasSize = await frame.$eval('canvas', (canvas) => ({
    buffer: [canvas.width, canvas.height],
    css: [canvas.clientWidth, canvas.clientHeight],
    scale: devicePixelRatio,
  }))
  for (let axis = 0; axis < 2; axis++)
    assert.ok(
      Math.abs(canvasSize.buffer[axis] - canvasSize.css[axis] * canvasSize.scale) <= 1,
      'The preview canvas must match its displayed size',
    )
  const frameBounds = await iframe.boundingBox()
  const primaryBounds = await frame.$eval('button[aria-label="Primary"]', (element) =>
    element.getBoundingClientRect().toJSON(),
  )
  await page.mouse.click(
    frameBounds.x + primaryBounds.x + primaryBounds.width / 2,
    frameBounds.y + primaryBounds.y + primaryBounds.height / 2,
  )
  await frame.waitForSelector('[aria-label="Button activations: 1"]')
  await screenshot('component-button-live')
  await page.click('button[aria-label="Stop gallery"]')
  assert.equal(await page.$$('iframe').then((items) => items.length), 0)
  await page.click('.gallery-launch button')
  await page.waitForSelector('.status-dot.live', { timeout: 90_000 })

  await page.setViewport({ width: 390, height: 844 })
  await page.goto(origin, { waitUntil: 'networkidle0' })
  await screenshot('home-mobile')
  await page.click('.mobile-menu-toggle')
  await page.click('#mobile-navigation a[href="/components"]')
  await page.waitForSelector('.component-nav-toggle')
  await page.click('.component-nav-toggle')
  await page.type('.component-search input', 'dialog')
  await page.click('.component-nav-group a[href="/components/dialog"]')
  await page.waitForFunction(() => location.pathname === '/components/dialog')
  assert.equal(
    await page.$eval('.component-nav-toggle', (element) => element.getAttribute('aria-expanded')),
    'false',
  )
  await screenshot('component-mobile')
  await page.click('.theme-button')
  await screenshot('component-mobile-dark')
  await page.waitForSelector('.status-dot.live', { timeout: 90_000 })
  const mobileFrame = await (await page.$('iframe')).contentFrame()
  await mobileFrame.waitForSelector('button[aria-label="Dialog"]')
  assert.equal(await mobileFrame.evaluate(() => innerWidth), 760)
  assert.ok(
    await page.$eval('.gallery-stage', (element) => element.scrollWidth > element.clientWidth),
  )
  await page.$eval('.gallery-stage', (element) => {
    element.scrollLeft = 260
  })
  await screenshot('component-mobile-live')
  await page.click('button[aria-label="Stop gallery"]')

  const response = await page.goto(`${origin}/components/does-not-exist`, {
    waitUntil: 'networkidle0',
  })
  assert.equal(response.status(), 404)
  await page.click('.error-page button')
  await page.waitForFunction(() => location.pathname === '/')

  assert.deepEqual(errors, [])
  assert.deepEqual(hydration, [])

  if (process.env.UPDATE_ASSETS === '1') {
    await page.setViewport({ width: 1200, height: 630 })
    if ((await page.$eval('html', (element) => element.dataset.theme)) === 'dark')
      await page.click('.theme-button')
    await page.goto(origin, { waitUntil: 'networkidle0' })
    await page.screenshot({ path: resolve(website, 'public/social.png') })
  }

  const loading = await browser.newPage()
  await loading.setViewport({ width: 1440, height: 1050 })
  await loading.setRequestInterception(true)
  let wasmRequest
  loading.on('request', (request) => {
    if (request.url().endsWith('.wasm') && !wasmRequest) wasmRequest = request
    else void request.continue()
  })
  await loading.evaluateOnNewDocument(() => {
    window.rendererStates = []
    addEventListener('argui:renderer-state', (event) =>
      window.rendererStates.push(event.detail.state),
    )
  })
  await loading.goto(`${origin}/components/button`, { waitUntil: 'domcontentloaded' })
  await loading.waitForSelector('.gallery-launch .spin')
  for (let attempt = 0; !wasmRequest && attempt < 100; attempt++) await pause(100)
  assert.ok(wasmRequest, 'The preview starts its WASM download automatically')
  assert.equal(await loading.$('.status-dot.live'), null)
  assert.equal(
    await loading.$eval('.gallery-stage', (element) => element.getAttribute('aria-busy')),
    'true',
  )
  await loading.screenshot({ path: resolve(output, 'component-loading.png'), fullPage: true })
  await wasmRequest.continue()
  await loading.waitForSelector('.status-dot.live', { timeout: 90_000 })
  const engine = await (await loading.$('iframe')).contentFrame()
  assert.deepEqual(await engine.evaluate(() => window.rendererStates), ['ready'])
  await engine.evaluate(() =>
    dispatchEvent(new CustomEvent('argui:renderer-state', { detail: { state: 'error' } })),
  )
  await loading.waitForFunction(() =>
    document.querySelector('.gallery-launch')?.textContent.includes('could not start'),
  )
  await loading.click('.gallery-launch button')
  await loading.waitForSelector('.status-dot.live', { timeout: 90_000 })
  await loading.click('.component-nav-group a[href="/components/checkbox"]')
  await loading.waitForFunction(() => location.pathname === '/components/checkbox')
  await loading.waitForSelector('.status-dot.live', { timeout: 90_000 })
  const checkbox = await (await loading.$('iframe')).contentFrame()
  await checkbox.waitForSelector('[role="checkbox"]')
  await loading.close()

  const fallback = await browser.newPage()
  await fallback.evaluateOnNewDocument(() =>
    Object.defineProperty(navigator, 'gpu', { get: () => undefined, configurable: true }),
  )
  await fallback.goto(`${origin}/components`, { waitUntil: 'networkidle0' })
  await fallback.waitForFunction(
    () => document.querySelector('.gallery-launch')?.textContent.includes('could not start'),
    { timeout: 30_000 },
  )
  assert.equal(await fallback.$$('iframe').then((items) => items.length), 0)
  await fallback.close()
  console.log(
    'Passed: desktop/mobile, themes, search, routing, copy, live WASM interaction, fallback, 404 and hydration.',
  )
} finally {
  await browser.close()
}
