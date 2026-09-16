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
  browser: 'firefox',
  executablePath: process.env.FIREFOX_PATH,
  headless: false,
  extraPrefsFirefox: {
    'dom.webgpu.enabled': true,
    'gfx.webrender.all': true,
  },
})

const pause = (milliseconds = 250) =>
  new Promise((resolvePause) => setTimeout(resolvePause, milliseconds))
const screenshot = async (page, name) => {
  await pause(350)
  const image = await page.screenshot({ path: resolve(output, `${name}.png`) })
  assert.ok(image.length > 15_000, `Blank Firefox capture: ${name}`)
}

try {
  const page = await browser.newPage()
  await page.setViewport({ width: 1440, height: 1050 })

  await page.goto(`${origin}/docs/technicalities/performance`, { waitUntil: 'networkidle0' })
  assert.match(
    await page.$eval('#main-content', (element) => element.textContent),
    /requestAnimationFrame/,
  )
  await screenshot(page, 'firefox-performance-guide')

  await page.goto(`${origin}/components/drag-drop`, { waitUntil: 'networkidle0' })
  await page.waitForSelector('.status-dot.live', { timeout: 90_000 })
  const dragFrame = await (await page.$('iframe')).contentFrame()
  const card = await dragFrame.waitForSelector('[aria-label="Opening titles at position 1"]')
  const cardBounds = await card.boundingBox()
  const startX = cardBounds.x + cardBounds.width / 2
  const startY = cardBounds.y + cardBounds.height / 2
  await page.mouse.move(startX, startY)
  await page.mouse.down()
  await page.mouse.move(startX + 42, startY + 190, { steps: 18 })
  await dragFrame.waitForSelector('[aria-label="Opening titles at position 3"]')
  await dragFrame.waitForSelector('[aria-label="HOLDING"]')
  assert.ok(
    await dragFrame.$$eval('[role="status"]', (elements) =>
      elements.some((element) =>
        /velocity/.test(element.getAttribute('aria-label') ?? element.textContent ?? ''),
      ),
    ),
    'Firefox exposes the frame-coalesced velocity status while dragging',
  )
  await screenshot(page, 'firefox-drag-held')
  await page.mouse.up()
  await dragFrame.waitForFunction(() => !document.querySelector('[aria-label="HOLDING"]'))
  await pause(700)
  await screenshot(page, 'firefox-drag-settled')

  await page.goto(`${origin}/components/split-pane`, { waitUntil: 'networkidle0' })
  await page.waitForSelector('.status-dot.live', { timeout: 90_000 })
  const splitIframe = await page.$('iframe')
  const splitFrame = await splitIframe.contentFrame()
  const splitIframeBounds = await splitIframe.boundingBox()
  await page.mouse.move(
    splitIframeBounds.x + splitIframeBounds.width * 0.7,
    splitIframeBounds.y + splitIframeBounds.height * 0.7,
  )
  await page.mouse.wheel({ deltaY: 1_200 })
  await pause(500)
  const separator = await splitFrame.waitForSelector(
    '[role="separator"][aria-label="Resize IDE console"]',
  )
  const initial = Number(
    await separator.evaluate((element) => element.getAttribute('aria-valuenow')),
  )
  const separatorBounds = await separator.boundingBox()
  const splitX = separatorBounds.x + separatorBounds.width / 2
  const splitY = separatorBounds.y + separatorBounds.height / 2
  await page.mouse.move(splitX, splitY)
  await page.mouse.down()
  await page.mouse.move(splitX, splitY - 70, { steps: 18 })
  await page.mouse.up()
  await pause(500)
  const resized = Number(
    await splitFrame.$eval('[role="separator"][aria-label="Resize IDE console"]', (element) =>
      element.getAttribute('aria-valuenow'),
    ),
  )
  console.log(`Firefox IDE console resize: ${initial}px → ${resized}px`)
  assert.ok(resized >= initial + 60, `Expected at least ${initial + 60}px after a 70px drag`)
  const detail = await splitFrame.waitForSelector('[aria-label^="$ cargo nextest run"]')
  const detailBounds = await detail.boundingBox()
  const movedSeparatorBounds = await separator.boundingBox()
  assert.ok(detailBounds.y >= movedSeparatorBounds.y + movedSeparatorBounds.height)
  await screenshot(page, 'firefox-split-nested')

  console.log('Firefox drag, split-pane and frame-coalescing checks passed.')
} finally {
  await browser.close()
}
