import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Use scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH, headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const output = process.env.SCREENSHOT_DIR ?? 'target/scroll-shadow-web';
await mkdir(output, { recursive: true });
const pause = () => new Promise(resolve => setTimeout(resolve, 600));
try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.setViewport({ width: 1220, height: 1050 });
    await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:3100/gallery/index.html', { waitUntil: 'networkidle0' });
    await page.waitForFunction(() => document.querySelector('#status')?.hidden, { timeout: 90_000 });
    await page.$eval('button[aria-label="Scroll shadow"]', element => element.click());
    const rect = selector => page.$eval(selector, element => element.getBoundingClientRect().toJSON());
    const capture = async name => {
        await pause();
        const png = await page.screenshot({ path: `${output}/${name}.png` });
        assert.ok(png.length > 15000, 'Blank capture');
        return png;
    };
    for (const scheme of ['light', 'dark']) {
        await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: scheme }]);
        await capture(`${scheme}-edges`);
        const collection = await rect('[aria-label="Collection 01"]');
        if (scheme === 'light') {
            await page.mouse.move(collection.x + 30, collection.y + 10);
            await page.mouse.wheel({ deltaX: 240 });
            await pause();
            const moved = await rect('[aria-label="Collection 03"]');
            assert.ok(moved.x < collection.x + 320, 'Horizontal content scrolls left');
            const row = await rect('button[aria-label="Application shell"]');
            await page.mouse.move(row.x + 200, row.y + 80);
            await page.mouse.wheel({ deltaY: 180 });
        }
        await capture(`${scheme}-scrolled`);
    }
    for (const scheme of ['light', 'dark']) {
        await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: scheme }]);
        if (scheme === 'light') await page.$eval('button[aria-label="DevTools"]', element => element.click());
        await page.waitForSelector('button[aria-label="Select element"]');
        const toolbar = await capture(`${scheme}-devtools`);
        const picker = await rect('button[aria-label="Select element"]');
        const contrastingPixels = await page.evaluate(async ({ base64, bounds, scheme }) => {
            const image = new Image();
            image.src = `data:image/png;base64,${base64}`;
            await image.decode();
            const canvas = document.createElement('canvas');
            canvas.width = image.width; canvas.height = image.height;
            const context = canvas.getContext('2d');
            context.drawImage(image, 0, 0);
            const pixels = context.getImageData(bounds.x + 7, bounds.y + 5, 18, 18).data;
            let count = 0;
            for (let index = 0; index < pixels.length; index += 4) {
                const rgb = Array.from(pixels.slice(index, index + 3));
                if (scheme === 'light' ? Math.max(...rgb) < 140 : Math.min(...rgb) > 180) count++;
            }
            return count;
        }, { base64: Buffer.from(toolbar).toString('base64'), bounds: picker, scheme });
        assert.ok(contrastingPixels > 20, `${scheme} picker icon must be visibly themed: ${contrastingPixels}`);
        await page.$eval('button[aria-label="Select element"]', element => element.click());
        await capture(`${scheme}-picker-active`);
        await page.$eval('button[aria-label="Select element"]', element => element.click());
    }
    assert.deepEqual(errors, []);
    console.log('Passed: both scroll axes, live light/dark themes and DevTools picker states.');
} finally { await browser.close(); }
