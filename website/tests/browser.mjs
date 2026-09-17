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
    await page.evaluate(() => {
      scrollTo(0, 0)
      if (document.activeElement instanceof HTMLElement) document.activeElement.blur()
    })
    await pause(500)
    const image = await page.screenshot({ path: resolve(output, `${name}.png`), fullPage: true })
    assert.ok(image.length > 15000, `Blank capture: ${name}`)
    assert.equal(
      await page.evaluate(() => document.documentElement.scrollWidth > innerWidth),
      false,
      `Horizontal overflow: ${name}`,
    )
  }
  const screenshotViewport = async (name) => {
    await pause(500)
    const image = await page.screenshot({ path: resolve(output, `${name}.png`) })
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
  assert.equal(
    await page.$$('.desktop-nav a[href="/get-started"]').then((items) => items.length),
    0,
  )
  assert.equal(
    await page.$eval('.desktop-nav .onboarding-tooltip > a', (element) =>
      element.textContent.trim(),
    ),
    'Docs',
  )
  assert.match(
    await page.$eval('.desktop-nav .onboarding-tooltip-message', (element) => element.textContent),
    /New\? Start here/,
  )
  assert.equal(
    await page.$eval('.hero-actions .action-primary', (element) => new URL(element.href).pathname),
    '/docs/start/installation',
  )
  assert.match(await page.$eval('.component-teaser h2', (element) => element.textContent), /74\+/)
  assert.ok(await page.$eval('meta[name="description"]', (element) => element.content.length > 50))
  assert.equal(
    await page.$eval('.discord-link img', (element) => new URL(element.src).pathname),
    '/discord.svg',
  )
  assert.notEqual(
    await page.$eval('.github-stars svg', (element) => getComputedStyle(element).color),
    await page.$eval('.github-stars', (element) => getComputedStyle(element).color),
  )
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

  for (const path of ['/features']) {
    await page.goto(origin + path, { waitUntil: 'networkidle0' })
    await screenshot(path.slice(1))
  }
  await page.goto(`${origin}/get-started`, { waitUntil: 'networkidle0' })
  assert.equal(new URL(page.url()).pathname, '/docs/start/installation')
  await browser
    .defaultBrowserContext()
    .overridePermissions(origin, ['clipboard-read', 'clipboard-sanitized-write'])
  await page.bringToFront()
  await page.click('.copy-button')
  await page.waitForSelector('.copy-button[aria-label="Copied"]')
  assert.match(
    await page.evaluate(() => navigator.clipboard.readText()),
    /argui = \{ version = "0\.3\.0", features = \["widget-button"\] \}/,
  )

  await page.goto(`${origin}/docs`, { waitUntil: 'networkidle0' })
  assert.equal(await page.$$('.docs-path li').then((items) => items.length), 4)
  assert.equal(await page.$$('.docs-card-grid a').then((items) => items.length), 17)
  assert.deepEqual(
    await page.$$eval('.docs-category-heading h2', (headings) =>
      headings.map((heading) => heading.textContent.trim()),
    ),
    ['Essentials', 'Advanced', 'Technicalities', 'Platforms'],
  )
  await page.type('.docs-home-search input', 'custom element')
  assert.match(
    await page.$eval('.docs-search-results', (element) => element.textContent),
    /custom/i,
  )
  await screenshot('docs-desktop')
  await page.goto(`${origin}/docs/start/first-window`, { waitUntil: 'networkidle0' })
  assert.equal(await page.$$('.docs-sources a').then((items) => items.length), 3)
  assert.match(await page.$eval('.docs-live-example', (element) => element.textContent), /Compiled/)
  assert.match(
    await page.$eval('.docs-live-example iframe', (element) => element.src),
    /\/examples\/docs\/index\.html\?example=first-window$/,
  )
  await page.$eval('.docs-live-example', (element) =>
    element.scrollIntoView({ block: 'center', behavior: 'instant' }),
  )
  await page.waitForSelector('.docs-live-example .status-dot.live', { timeout: 90_000 })
  assert.equal(
    await page.$eval('.docs-live-example iframe', (element) => document.activeElement === element),
    false,
    'An embedded Web canvas must not take focus while it loads',
  )
  const docsFrame = await (await page.$('.docs-live-example iframe')).contentFrame()
  await docsFrame.waitForSelector('[aria-label="Hello from Argui"]')
  await screenshot('docs-guide-desktop')

  await page.goto(`${origin}/docs/essentials/styling`, { waitUntil: 'networkidle0' })
  await page.$eval('.docs-live-example', (element) =>
    element.scrollIntoView({ block: 'center', behavior: 'instant' }),
  )
  await page.waitForSelector('.docs-live-example .status-dot.live', { timeout: 90_000 })
  const themeFrame = await (await page.$('.docs-live-example iframe')).contentFrame()
  await themeFrame.waitForSelector('button[aria-label="Dark"]')
  await themeFrame.$eval('button[aria-label="Dark"]', (element) => element.click())
  await themeFrame.$eval('button[aria-label="Violet"]', (element) => element.click())
  await themeFrame.$eval('button[aria-label="Override tokens"]', (element) => element.click())
  await themeFrame.waitForSelector('button[aria-label="Reset tokens"]')
  await screenshot('docs-theme-configurator')

  await page.goto(`${origin}/docs/essentials/interaction-api`, { waitUntil: 'networkidle0' })
  assert.equal(await page.$$('.docs-table-wrap tbody tr').then((items) => items.length), 9)
  assert.match(
    await page.$eval('.docs-table-wrap', (element) => element.textContent),
    /on_change[\s\S]*on_commit/,
  )
  await page.$eval('.docs-live-example iframe', (element) =>
    element.scrollIntoView({ block: 'center', behavior: 'instant' }),
  )
  await page.waitForSelector('.docs-live-example .status-dot.live', { timeout: 90_000 })
  const interactionFrame = await (await page.$('.docs-live-example iframe')).contentFrame()
  await interactionFrame.waitForSelector('button[aria-label="Save preset"]')
  await interactionFrame.$eval('button[aria-label="Save preset"]', (element) => element.click())
  await interactionFrame.waitForSelector('[aria-label="Saved 1 time(s)"]')
  await screenshotViewport('docs-interaction-api')

  await page.goto(`${origin}/docs/technicalities/performance`, { waitUntil: 'networkidle0' })
  assert.match(
    await page.$eval('.docs-table-wrap', (element) => element.textContent),
    /FrameCoalesced/,
  )
  assert.match(
    await page.$eval('#main-content', (element) => element.textContent),
    /requestAnimationFrame/,
  )
  await page.$eval('.docs-live-example iframe', (element) =>
    element.scrollIntoView({ block: 'center', behavior: 'instant' }),
  )
  await page.waitForSelector('.docs-live-example .status-dot.live', { timeout: 90_000 })
  await screenshotViewport('docs-performance-chrome')

  await page.goto(`${origin}/docs/architecture/clean-code`, { waitUntil: 'networkidle0' })
  await page.$eval('.docs-live-example iframe', (element) =>
    element.scrollIntoView({ block: 'center', behavior: 'instant' }),
  )
  await page.waitForSelector('.docs-live-example .status-dot.live', { timeout: 90_000 })
  const cleanFrame = await (await page.$('.docs-live-example iframe')).contentFrame()
  for (const label of [
    'Complete next (1 of 3)',
    'Complete next (2 of 3)',
    'Complete next (3 of 3)',
  ]) {
    await cleanFrame.waitForSelector(`button[aria-label="${label}"]`)
    await cleanFrame.$eval(`button[aria-label="${label}"]`, (element) => element.click())
  }
  await cleanFrame.waitForSelector('button[aria-label="All complete"]')
  assert.equal(
    await cleanFrame.$eval(
      'button[aria-label="All complete"]',
      (element) => element.disabled || element.getAttribute('aria-disabled') === 'true',
    ),
    true,
  )
  await screenshotViewport('docs-clean-code-complete')

  await page.goto(`${origin}/docs/architecture/custom-elements`, { waitUntil: 'networkidle0' })
  await page.$eval('.docs-live-example iframe', (element) =>
    element.scrollIntoView({ block: 'center', behavior: 'instant' }),
  )
  await page.waitForSelector('.docs-live-example .status-dot.live', { timeout: 90_000 })
  const timelineFrame = await (await page.$('.docs-live-example iframe')).contentFrame()
  await timelineFrame.waitForSelector('button[aria-label="Forward 1s"]')
  await timelineFrame.$eval('button[aria-label="Forward 1s"]', (element) => element.click())
  await timelineFrame.waitForFunction(
    () =>
      Number(
        document
          .querySelector('[role="slider"][aria-label="Timeline playhead"]')
          ?.getAttribute('aria-valuenow'),
      ) === 8.5,
  )
  const playhead = await timelineFrame.$('[role="slider"][aria-label="Timeline playhead"]')
  const playheadBounds = await playhead.boundingBox()
  await page.mouse.move(
    playheadBounds.x + playheadBounds.width / 2,
    playheadBounds.y + playheadBounds.height / 2,
  )
  await page.mouse.down()
  await page.mouse.move(
    playheadBounds.x + playheadBounds.width / 2 + 64,
    playheadBounds.y + playheadBounds.height / 2,
    { steps: 3 },
  )
  await page.mouse.up()
  await timelineFrame.waitForFunction(
    () =>
      Number(
        document
          .querySelector('[role="slider"][aria-label="Timeline playhead"]')
          ?.getAttribute('aria-valuenow'),
      ) === 10.5,
  )
  const bRollClip = await timelineFrame.$('[aria-label^="B-roll clip"]')
  const bRollBounds = await bRollClip.boundingBox()
  await page.mouse.move(
    bRollBounds.x + bRollBounds.width / 2,
    bRollBounds.y + bRollBounds.height / 2,
  )
  await page.mouse.down()
  await page.mouse.move(
    bRollBounds.x + bRollBounds.width / 2,
    bRollBounds.y + bRollBounds.height / 2 + 58,
    { steps: 4 },
  )
  await page.mouse.up()
  await timelineFrame.waitForSelector('[aria-label="B-roll clip at 14.0s on track 2"]')

  await page.evaluate(() => navigator.clipboard.writeText(''))
  const timelineCanvas = await timelineFrame.$('canvas')
  const timelineBounds = await timelineCanvas.boundingBox()
  await page.mouse.move(
    timelineBounds.x + timelineBounds.width - 20,
    timelineBounds.y + timelineBounds.height - 20,
  )
  await page.mouse.down()
  await page.mouse.move(timelineBounds.x + 20, timelineBounds.y + 20, { steps: 6 })
  await page.mouse.up()
  await timelineCanvas.focus()
  await page.keyboard.down('Control')
  await page.keyboard.press('c')
  await page.keyboard.up('Control')
  await page.waitForFunction(async () => (await navigator.clipboard.readText()).length > 0)
  assert.match(
    await page.evaluate(() => navigator.clipboard.readText()),
    /Product launch edit|VIDEO 1|Music bed/,
  )
  await screenshotViewport('docs-text-selection-from-whitespace')
  await page.mouse.click(
    timelineBounds.x + timelineBounds.width - 20,
    timelineBounds.y + timelineBounds.height - 20,
  )
  await screenshotViewport('docs-video-editor-timeline')

  await page.goto(`${origin}/docs/advanced/i18n`, { waitUntil: 'networkidle0' })
  await page.$eval('.docs-live-example iframe', (element) =>
    element.scrollIntoView({ block: 'center', behavior: 'instant' }),
  )
  await page.waitForSelector('.docs-live-example .status-dot.live', { timeout: 90_000 })
  const i18nFrame = await (await page.$('.docs-live-example iframe')).contentFrame()
  await i18nFrame.waitForSelector('button[aria-label="العربية"]')
  await i18nFrame.$eval('button[aria-label="العربية"]', (element) => element.click())
  await i18nFrame.waitForSelector('[aria-label^="مرحبًا"]')
  await screenshotViewport('docs-i18n-arabic')

  await page.goto(`${origin}/docs/advanced/overlays`, { waitUntil: 'networkidle0' })
  await page.$eval('.docs-live-example', (element) =>
    element.scrollIntoView({ block: 'center', behavior: 'instant' }),
  )
  await page.waitForSelector('.docs-live-example .status-dot.live', { timeout: 90_000 })
  await page.$eval('.docs-live-example iframe', (element) =>
    element.scrollIntoView({ block: 'center', behavior: 'instant' }),
  )
  const overlayFrame = await (await page.$('.docs-live-example iframe')).contentFrame()
  await overlayFrame.waitForSelector('button[aria-label="Solid popover"]')
  await overlayFrame.$eval('button[aria-label="Solid popover"]', (element) => element.click())
  await overlayFrame.waitForSelector('button[aria-label="Close solid popover"]')
  await screenshotViewport('docs-overlay-solid')
  await overlayFrame.$eval('button[aria-label="Close solid popover"]', (element) => element.click())
  await overlayFrame.$eval('button[aria-label="Blurred popover"]', (element) => element.click())
  await overlayFrame.waitForSelector('button[aria-label="Close blurred popover"]')
  await screenshotViewport('docs-overlay-blurred')

  await page.goto(`${origin}/docs/platforms/roadmap`, { waitUntil: 'networkidle0' })
  assert.match(await page.$eval('.docs-table-wrap', (element) => element.textContent), /Shipping/)
  assert.match(
    await page.$eval('.docs-table-wrap', (element) => element.textContent),
    /Planned · High/,
  )
  assert.equal(await page.$$('.docs-table-wrap tbody tr').then((items) => items.length), 33)
  await screenshot('docs-platform-roadmap')
  await page.$eval('.docs-live-example iframe', (element) =>
    element.scrollIntoView({ block: 'center', behavior: 'instant' }),
  )
  await page.waitForSelector('.docs-live-example .status-dot.live', { timeout: 90_000 })
  const roadmapFrame = await (await page.$('.docs-live-example iframe')).contentFrame()
  await roadmapFrame.waitForSelector('button[aria-label="Show the full planned roadmap"]')
  await roadmapFrame.$eval('button[aria-label="Show the full planned roadmap"]', (element) =>
    element.click(),
  )
  await roadmapFrame.waitForSelector('button[aria-label="Show shipping only"]')
  await screenshotViewport('docs-platform-roadmap-live')

  await page.goto(`${origin}/components/drag-drop`, { waitUntil: 'networkidle0' })
  await page.waitForSelector('.status-dot.live', { timeout: 90_000 })
  const dragFrame = await (await page.$('iframe')).contentFrame()
  const openingCard = await dragFrame.waitForSelector('[aria-label="Opening titles at position 1"]')
  const openingBounds = await openingCard.boundingBox()
  await page.mouse.move(
    openingBounds.x + openingBounds.width / 2,
    openingBounds.y + openingBounds.height / 2,
  )
  await page.mouse.down()
  await page.mouse.move(
    openingBounds.x + openingBounds.width / 2 + 36,
    openingBounds.y + openingBounds.height / 2 + 190,
    { steps: 5 },
  )
  await dragFrame.waitForSelector('[aria-label="Opening titles at position 3"]')
  await dragFrame.waitForSelector('[aria-label="HOLDING"]')
  await screenshotViewport('component-drag-drop-held')
  await page.mouse.up()
  await dragFrame.waitForFunction(() => !document.querySelector('[aria-label="HOLDING"]'))
  await screenshotViewport('component-drag-drop-reordered')

  await page.goto(`${origin}/components/split-pane`, { waitUntil: 'networkidle0' })
  assert.match(
    await page.$eval('.source-link', (element) => element.textContent),
    /argui-widget-gallery\/src\/pages\/split_pane.rs/,
  )
  await page.waitForSelector('.status-dot.live', { timeout: 90_000 })
  const splitIframe = await page.$('iframe')
  const splitFrame = await splitIframe.contentFrame()
  const explorerSplit = await splitFrame.waitForSelector(
    '[role="separator"][aria-label="Resize file explorer"]',
  )
  const initialSplitSize = Number(
    await explorerSplit.evaluate((element) => element.getAttribute('aria-valuenow')),
  )
  const splitBounds = await explorerSplit.boundingBox()
  await page.mouse.move(
    splitBounds.x + splitBounds.width / 2,
    splitBounds.y + splitBounds.height / 2,
  )
  await page.mouse.down()
  await page.mouse.move(
    splitBounds.x + splitBounds.width / 2 + 80,
    splitBounds.y + splitBounds.height / 2,
    { steps: 4 },
  )
  await page.mouse.up()
  await splitFrame.waitForFunction(
    (initial) =>
      Number(
        document
          .querySelector('[role="separator"][aria-label="Resize file explorer"]')
          ?.getAttribute('aria-valuenow'),
      ) >=
      initial + 70,
    {},
    initialSplitSize,
  )
  await screenshotViewport('component-split-pane-resized')

  const splitIframeBounds = await splitIframe.boundingBox()
  await page.mouse.move(
    splitIframeBounds.x + splitIframeBounds.width * 0.7,
    splitIframeBounds.y + splitIframeBounds.height * 0.7,
  )
  await page.mouse.wheel({ deltaY: 1_200 })
  await pause(500)
  const consoleSplit = await splitFrame.waitForSelector(
    '[role="separator"][aria-label="Resize IDE console"]',
  )
  const initialConsoleSize = Number(
    await consoleSplit.evaluate((element) => element.getAttribute('aria-valuenow')),
  )
  const consoleBounds = await consoleSplit.boundingBox()
  await page.mouse.move(
    consoleBounds.x + consoleBounds.width / 2,
    consoleBounds.y + consoleBounds.height / 2,
  )
  await page.mouse.down()
  await page.mouse.move(
    consoleBounds.x + consoleBounds.width / 2,
    consoleBounds.y + consoleBounds.height / 2 - 70,
    { steps: 14 },
  )
  await page.mouse.up()
  await splitFrame.waitForFunction(
    (initial) =>
      Number(
        document
          .querySelector('[role="separator"][aria-label="Resize IDE console"]')
          ?.getAttribute('aria-valuenow'),
      ) >=
      initial + 60,
    {},
    initialConsoleSize,
  )
  await screenshotViewport('component-split-pane-nested-chrome')

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
  const focusedSearch = await page.$eval('.component-search', (element) => {
    const bounds = element.getBoundingClientRect()
    const content = document.querySelector('#main-content').getBoundingClientRect()
    const style = getComputedStyle(element)
    return {
      inside: bounds.left >= content.left && bounds.right <= content.right,
      outline: style.outlineStyle,
      inset: style.boxShadow.includes('inset'),
    }
  })
  assert.deepEqual(focusedSearch, { inside: true, outline: 'none', inset: true })
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
    border: getComputedStyle(canvas).borderStyle,
    outline: getComputedStyle(canvas).outlineStyle,
  }))
  assert.equal(canvasSize.border, 'none')
  assert.equal(canvasSize.outline, 'none')
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
  const restartedFrame = await (await page.$('iframe')).contentFrame()
  await restartedFrame.waitForSelector('button[aria-label="Async tasks"]')
  await restartedFrame.$eval('button[aria-label="Async tasks"]', (element) => element.click())
  await page.waitForFunction(() =>
    document
      .querySelector('.action-link.action-text')
      ?.getAttribute('href')
      ?.endsWith('/crates/argui-widget-gallery/src/pages/async_tasks.rs'),
  )

  await page.setViewport({ width: 1000, height: 820 })
  await page.goto(`${origin}/examples`, { waitUntil: 'networkidle0' })
  assert.equal(await page.$$('.app-example-list button').then((items) => items.length), 3)
  assert.equal(await page.$$('.more-example-grid a').then((items) => items.length), 4)
  assert.match(await page.$eval('.app-example-list', (element) => element.textContent), /0\.30 s/)
  assert.match(await page.$eval('.app-example-list', (element) => element.textContent), /0\.79 s/)
  await page.setViewport({ width: 1000, height: 1050 })
  await page.click('.app-example-list button:nth-child(3)')
  await page.waitForFunction(() =>
    document.querySelector('iframe')?.src.includes('/examples/gpu-canvas/index.html'),
  )
  await page.waitForSelector('.status-dot.live', { timeout: 90_000 })
  const gpuIframe = await page.$('iframe')
  const gpuFrame = await gpuIframe.contentFrame()
  const gpuCanvas = await gpuFrame.waitForSelector(
    '[role="img"][aria-label="Interactive GPU particle canvas"]',
  )
  const gpuCanvasBounds = await gpuCanvas.evaluate((element) =>
    element.getBoundingClientRect().toJSON(),
  )
  assert.ok(gpuCanvasBounds && gpuCanvasBounds.width > 400 && gpuCanvasBounds.height > 300)
  await gpuFrame.$eval('button[aria-label="Drag Y: natural"]', (element) => element.click())
  await gpuFrame.waitForSelector('button[aria-label="Drag Y: inverted"]')
  await screenshot('app-example-gpu-canvas')
  await page.setViewport({ width: 1000, height: 820 })
  await page.click('.app-example-list button:nth-child(2)')
  await page.waitForFunction(() =>
    document.querySelector('.gallery-status')?.textContent.includes('Widget Gallery'),
  )
  assert.equal(
    await page.$eval('.app-example-list button:nth-child(2)', (element) =>
      element.getAttribute('aria-selected'),
    ),
    'true',
  )
  assert.match(await page.$eval('iframe', (element) => element.src), /\/gallery\/index\.html/)
  await page.click('.app-example-list button:first-child')
  await page.waitForFunction(() =>
    document.querySelector('.gallery-status')?.textContent.includes('AI streaming harness'),
  )
  await page.waitForSelector('.status-dot.live', { timeout: 90_000 })
  const desktopExampleFrame = await (await page.$('iframe')).contentFrame()
  await desktopExampleFrame.waitForSelector('input[aria-label="Prompt"]')
  await page.$eval('iframe', (element) =>
    element.scrollIntoView({ block: 'center', behavior: 'instant' }),
  )
  const harnessLayout = async (example) =>
    example.evaluate(() => {
      const prompt = document.querySelector('input[aria-label="Prompt"]').getBoundingClientRect()
      const send = document.querySelector('button[aria-label="Send"]').getBoundingClientRect()
      const canvasElement = document.querySelector('canvas')
      const canvas = canvasElement.getBoundingClientRect()
      return {
        prompt: prompt.toJSON(),
        send: send.toJSON(),
        canvas: {
          ...canvas.toJSON(),
          bufferWidth: canvasElement.width,
          bufferHeight: canvasElement.height,
          scale: devicePixelRatio,
        },
        labels: [...document.querySelectorAll('[aria-label]')].map((element) =>
          element.getAttribute('aria-label'),
        ),
      }
    })
  const assertHarnessLayout = (layout) => {
    assert.ok(layout.prompt.width > layout.send.width * 2.8, 'The prompt keeps the 80% share')
    assert.ok(layout.prompt.right <= layout.send.left, 'The prompt and Send button do not overlap')
    assert.ok(
      layout.send.right <= layout.canvas.right + 1,
      'The Send button stays inside the canvas',
    )
    assert.ok(
      Math.abs(
        layout.prompt.y + layout.prompt.height / 2 - (layout.send.y + layout.send.height / 2),
      ) <= 4,
      'The prompt and Send button share a centered row',
    )
    assert.ok(
      Math.abs(layout.canvas.bufferWidth - layout.canvas.width * layout.canvas.scale) <= 1,
      'The canvas backing width follows the browser scale',
    )
    assert.ok(
      Math.abs(layout.canvas.bufferHeight - layout.canvas.height * layout.canvas.scale) <= 1,
      'The canvas backing height follows the browser scale',
    )
  }
  const desktopHarness = await harnessLayout(desktopExampleFrame)
  assertHarnessLayout(desktopHarness)
  assert.equal(
    await desktopExampleFrame.$eval('input[aria-label="Prompt"]', (element) => element.value),
    'What is an LLM?',
  )
  assert.ok(desktopHarness.labels.includes('Live telemetry'))
  assert.equal(await desktopExampleFrame.$('button[aria-label="Stop"]'), null)
  assert.equal(await desktopExampleFrame.$('button[aria-label="Clear"]'), null)
  await desktopExampleFrame.$eval('button[aria-label="Send"]', (element) => element.click())
  await desktopExampleFrame.waitForSelector('button[aria-label="Restart"]')
  await pause(850)
  const desktopFrameBounds = await page.$eval('iframe', (element) =>
    element.getBoundingClientRect().toJSON(),
  )
  await page.mouse.move(
    desktopFrameBounds.x + desktopHarness.prompt.x + desktopHarness.prompt.width / 2,
    desktopFrameBounds.y + desktopHarness.prompt.y - 100,
  )
  await page.mouse.wheel({ deltaY: -180 })
  await desktopExampleFrame.waitForSelector('button[aria-label="Go to latest message"]')
  const latest = await desktopExampleFrame.$eval(
    'button[aria-label="Go to latest message"]',
    (element) => element.getBoundingClientRect().toJSON(),
  )
  assert.ok(
    Math.abs(
      latest.x + latest.width / 2 - (desktopHarness.canvas.x + desktopHarness.canvas.width / 2),
    ) <= 2,
    'The latest-message arrow is centered in the app',
  )
  await screenshot('app-example-desktop-streaming')

  await page.setViewport({
    width: 390,
    height: 844,
    deviceScaleFactor: 2,
    isMobile: true,
    hasTouch: true,
  })
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
  const responsiveFrame = await page.$eval('iframe', (element) => ({
    frame: element.clientWidth,
    stage: element.parentElement.clientWidth,
  }))
  assert.equal(responsiveFrame.frame, responsiveFrame.stage)
  assert.ok(responsiveFrame.frame <= 390)
  assert.equal(
    await page.$eval('.gallery-stage', (element) => element.scrollWidth > element.clientWidth),
    false,
  )
  await mobileFrame.waitForSelector('[role="navigation"][aria-label="Component navigation"]')
  await screenshot('component-mobile-live')
  await page.click('button[aria-label="Stop gallery"]')

  await page.goto(`${origin}/docs`, { waitUntil: 'networkidle0' })
  assert.equal(
    await page.$eval('.docs-sidebar', (element) => getComputedStyle(element).display),
    'none',
  )
  await page.click('.docs-nav-toggle')
  assert.equal(
    await page.$eval('.docs-sidebar', (element) => getComputedStyle(element).display),
    'block',
  )
  await screenshot('docs-mobile-menu')
  await page.click('.docs-nav-group a')
  await page.waitForSelector('.docs-guide-heading')
  assert.equal(
    await page.$eval('.docs-nav-toggle', (element) => getComputedStyle(element).display),
    'flex',
  )
  await screenshot('docs-guide-mobile')

  await page.goto(`${origin}/examples`, { waitUntil: 'networkidle0' })
  await page.waitForSelector('.status-dot.live', { timeout: 90_000 })
  const exampleFrame = await (await page.$('iframe')).contentFrame()
  await exampleFrame.waitForSelector('input[aria-label="Prompt"]')
  assert.ok(
    await exampleFrame.$eval('canvas', (canvas) => canvas.clientWidth <= innerWidth),
    'The app example canvas fits its mobile viewport',
  )
  const mobileHarness = await harnessLayout(exampleFrame)
  assertHarnessLayout(mobileHarness)
  assert.ok(!mobileHarness.labels.includes('Live telemetry'))
  const mobileSend = await exampleFrame.$('button[aria-label="Send"]')
  if (mobileSend) await mobileSend.evaluate((element) => element.click())
  await exampleFrame.waitForSelector('button[aria-label="Restart"]')
  await pause(600)
  const visibleMessageWidth = await exampleFrame.$$eval('[aria-label][role="text"]', (elements) =>
    Math.max(
      ...elements.map((element) => {
        const bounds = element.getBoundingClientRect()
        return bounds.width > 1 && bounds.y > 60 ? bounds.width : 0
      }),
    ),
  )
  assert.ok(
    visibleMessageWidth >= mobileHarness.canvas.width * 0.7,
    'Streamed VList messages keep the available mobile width',
  )
  await screenshot('app-example-mobile-live')

  await page.setViewport({ width: 390, height: 844, deviceScaleFactor: 2 })
  await page.goto(`${origin}/examples`, { waitUntil: 'networkidle0' })
  await page.click('.app-example-list button:nth-child(3)')
  await page.waitForFunction(() =>
    document.querySelector('iframe')?.src.includes('/examples/gpu-canvas/index.html'),
  )
  await page.$eval('iframe', (element) => element.scrollIntoView({ block: 'center' }))
  await page.waitForSelector('.status-dot.live', { timeout: 90_000 })
  const mobileGpuFrame = await (await page.$('iframe')).contentFrame()
  await mobileGpuFrame.waitForSelector('button[aria-label="Drag Y: natural"]')
  const mobileGpuLayout = await mobileGpuFrame.$eval('canvas', (canvas) => ({
    canvas: canvas.clientWidth,
    viewport: innerWidth,
    overflow: document.documentElement.scrollWidth > innerWidth,
  }))
  assert.ok(mobileGpuLayout.canvas <= mobileGpuLayout.viewport)
  assert.equal(mobileGpuLayout.overflow, false)
  await screenshot('app-example-gpu-canvas-mobile')

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
    document.querySelector('.gallery-launch')?.textContent.includes('WebGPU is unavailable'),
  )
  await loading.click('.browser-setting button')
  await loading.waitForFunction(() =>
    document.querySelector('.browser-setting button')?.textContent.includes('Copied'),
  )
  assert.equal(await loading.evaluate(() => navigator.clipboard.readText()), 'chrome://gpu')
  await loading.close()

  const fallback = await browser.newPage()
  await fallback.evaluateOnNewDocument(() =>
    Object.defineProperty(navigator, 'gpu', { get: () => undefined, configurable: true }),
  )
  await fallback.goto(`${origin}/components`, { waitUntil: 'networkidle0' })
  await fallback.waitForFunction(
    () => document.querySelector('.gallery-launch')?.textContent.includes('WebGPU is unavailable'),
    { timeout: 30_000 },
  )
  assert.match(await fallback.$eval('.gallery-launch', (element) => element.textContent), /Chrome/)
  assert.match(
    await fallback.$eval('.gallery-launch', (element) => element.textContent),
    /chrome:\/\/gpu/,
  )
  assert.equal(await fallback.$$('iframe').then((items) => items.length), 0)
  await fallback.close()
  console.log(
    'Passed: desktop/mobile, themes, search, routing, copy, live WASM interaction, fallback, 404 and hydration.',
  )
} finally {
  await browser.close()
}
