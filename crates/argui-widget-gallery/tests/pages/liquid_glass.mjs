import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Use scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH, headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const output = process.env.SCREENSHOT_DIR ?? 'target/liquid-glass-web';
await mkdir(output, { recursive: true });
const pause = () => new Promise(resolve => setTimeout(resolve, 400));
try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    await page.setViewport({ width: 1220, height: 1000 });
    await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8793/widgets/', { waitUntil: 'networkidle0' });
    const button = label => `button[aria-label="${label}"]`;
    const click = async label => {
        await page.waitForSelector(button(label));
        await page.$eval(button(label), element => element.click());
        await pause();
    };
    const rect = label => page.$eval(`[role="group"][aria-label="${label}"]`, element => element.getBoundingClientRect().toJSON());
    await click('Liquid glass');
    const bar = await rect('Glass navigation');
    const feed = await rect('Scrollable palettes');
    assert.ok(bar.width > 250 && bar.bottom <= 1000, 'Visible navigation');
    const before = await page.screenshot({ path: `${output}/before-scroll.png`, clip: { x: bar.x, y: bar.y, width: bar.width, height: bar.height } });
    await page.mouse.move(feed.x + feed.width / 2, feed.y + 170);
    await page.mouse.wheel({ deltaY: 290 }); await pause();
    const scrolledBar = await rect('Glass navigation');
    for (const axis of ['x', 'y', 'width', 'height']) {
        assert.ok(Math.abs(scrolledBar[axis] - bar[axis]) < 0.1, `Navigation ${axis} stays fixed while the feed scrolls`);
    }
    const after = await page.screenshot({ path: `${output}/after-scroll.png`, clip: { x: bar.x, y: bar.y, width: bar.width, height: bar.height } });
    assert.notDeepEqual(before, after, 'Scrolling changes the colors sampled through the glass');
    const capture = await page.screenshot({ path: `${output}/gallery.png` });
    assert.ok(capture.length > 15000, 'Blank capture');
    const increment = async (label, expected) => {
        const selector = `[role="slider"][aria-label="${label}"]`;
        await page.$eval(selector, slider => slider.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true })));
        await page.waitForFunction((selector, expected) => Math.abs(Number(document.querySelector(selector)?.getAttribute('aria-valuenow')) - expected) < 0.0001, {}, selector, expected);
    };
    await increment('Refraction', 39);
    await increment('Blur', 8.25);
    await increment('Tint amount', 0.28);
    await click('Depth: off'); await click('Depth: on');
    await click('Tint: blue');
    await click('Reset settings');
    await page.waitForFunction(() => Number(document.querySelector('[role="slider"][aria-label="Blur"]')?.getAttribute('aria-valuenow')) === 8);
    await click('Collections'); await click('Saved'); await click('Explore');
    await click('Effect: on'); await click('Effect: off');
    for (const scheme of ['light', 'dark']) {
        await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: scheme }]);
        await pause();
        const png = await page.screenshot({ path: `${output}/${scheme}.png` });
        assert.ok(png.length > 15000, 'Blank theme capture');
    }
    await click('Scroll shadow');
    assert.equal(await page.$('[role="group"][aria-label="Glass navigation"]'), null, 'Separate effect pages');
    assert.equal(await page.$(button('Custom WGSL')), null);
    await page.screenshot({ path: `${output}/scroll-shadow.png` });
    assert.deepEqual(errors, []);
    console.log('Liquid Glass: registered assets, fixed navigation, scrolling backdrop and live controls passed');
} finally { await browser.close(); }
