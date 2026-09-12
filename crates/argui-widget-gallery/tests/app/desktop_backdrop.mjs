import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH, headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const output = process.env.SCREENSHOT_DIR ?? 'target/desktop-backdrop';
await mkdir(output, { recursive: true });
const pause = (ms = 250) => new Promise(resolve => setTimeout(resolve, ms));
try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    await page.setViewport({ width: 1220, height: 900 });
    await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8793/widgets/', { waitUntil: 'networkidle0' });
    await page.waitForFunction(() => document.querySelector('canvas')?.getContext('webgpu')?.getConfiguration()?.alphaMode === 'premultiplied');
    const selector = (role, label) => `[role="${role}"][aria-label="${label}"]`;
    const appearance = 'button[aria-label="Sidebar appearance"]';
    const glass = selector('switch', 'Desktop glass');
    const fallback = selector('switch', 'Allow transparency without blur');
    const rect = selector => page.$eval(selector, element => element.getBoundingClientRect().toJSON());
    const click = async selector => {
        const r = await rect(selector);
        assert.ok(r.width > 0 && r.height > 0 && r.right <= 1220 && r.bottom <= 900);
        await page.mouse.click(r.x + r.width / 2, r.y + r.height / 2);
        await pause();
    };
    const capture = async name => {
        const image = await page.screenshot({ path: `${output}/${name}.png` });
        assert.ok(image.length > 15000, 'Blank capture');
    };
    for (const scheme of ['light', 'dark']) {
        await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: scheme }]);
        await pause(400);
        await click(appearance);
        await capture(`${scheme}-settings`);
        await click(glass);
        assert.equal(await page.$eval(glass, element => element.getAttribute('aria-checked')), 'true');
        await click(fallback);
        for (const label of ['Surface opacity', 'Accent tint', 'Inactive opacity']) {
            const slider = selector('slider', label);
            await click(slider);
            const before = Number(await page.$eval(slider, element => element.getAttribute('aria-valuenow')));
            await page.keyboard.press('ArrowRight'); await pause();
            assert.equal(Number(await page.$eval(slider, element => element.getAttribute('aria-valuenow'))), before + 1);
        }
        await page.evaluate(() => {
            document.body.style.background = 'repeating-linear-gradient(35deg, #4c1d95 0 80px, #0284c7 80px 160px, #d97706 160px 240px)';
        });
        await capture(`${scheme}-tinted-sidebar`);
        await page.keyboard.press('Escape'); await pause();
        assert.equal(await page.$eval(appearance, element => element.getAttribute('aria-expanded')), 'false');
        await capture(`${scheme}-sidebar`);
        await click(appearance);
        await click(glass);
        await click(fallback);
        await page.keyboard.press('Escape'); await pause();
        await capture(`${scheme}-disabled`);
    }
    await page.setViewport({ width: 800, height: 720 });
    await pause(400); await click(appearance); await capture('narrow-settings');
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ captures: output, errors }));
} finally {
    await browser.close();
}
