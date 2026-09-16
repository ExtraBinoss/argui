import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Run with scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined, 'Do not inherit the desktop X11 display');
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const puppeteer = imported.puppeteer ?? imported.default;
const output = process.env.SCREENSHOT_DIR ?? 'target/gpu-canvas-browser';
await mkdir(output, { recursive: true });

const browser = await puppeteer.launch({
    executablePath: process.env.CHROME_PATH,
    headless: false,
    args: [
        '--ozone-platform=wayland',
        '--enable-unsafe-webgpu',
        '--ignore-gpu-blocklist',
        '--enable-features=Vulkan',
        '--use-angle=vulkan',
    ],
});

try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    page.on('console', message => {
        if (message.type() === 'error' && !message.text().includes('simulated failure')) {
            errors.push(message.text());
        }
    });
    await page.setViewport({ width: 1280, height: 780, deviceScaleFactor: 1 });
    await page.evaluateOnNewDocument(() => {
        window.arguiRendererState = 'loading';
        window.arguiFrameCount = 0;
        window.addEventListener('argui:renderer-state', event => {
            window.arguiRendererState = event.detail.state;
        });
        const raf = window.requestAnimationFrame.bind(window);
        window.requestAnimationFrame = callback => raf(time => {
            window.arguiFrameCount++;
            callback(time);
        });
    });
    await page.goto(process.env.GPU_CANVAS_URL ?? 'http://127.0.0.1:8794/', {
        waitUntil: 'networkidle0',
    });
    await page.waitForFunction(() => window.arguiRendererState === 'ready');
    await page.waitForSelector('[role="img"][aria-label="Interactive GPU particle canvas"]');

    const capture = async name => {
        const png = await page.screenshot({ path: `${output}/${name}.png` });
        const populated = await page.evaluate(async base64 => {
            const image = new Image();
            image.src = `data:image/png;base64,${base64}`;
            await image.decode();
            const canvas = document.createElement('canvas');
            canvas.width = image.width;
            canvas.height = image.height;
            const context = canvas.getContext('2d');
            context.drawImage(image, 0, 0);
            const pixels = context.getImageData(0, 0, image.width, image.height).data;
            const colors = new Set();
            for (let index = 0; index < pixels.length; index += 64) {
                colors.add((pixels[index] << 16) | (pixels[index + 1] << 8) | pixels[index + 2]);
                if (colors.size > 48) return true;
            }
            return false;
        }, Buffer.from(png).toString('base64'));
        assert.ok(populated, `Blank GPU Canvas Lab capture: ${name}`);
    };
    const button = label => `button[aria-label="${label}"]`;
    const click = async label => {
        await page.waitForSelector(button(label));
        await page.$eval(button(label), element => element.click());
        await new Promise(resolve => setTimeout(resolve, 250));
    };
    const waitForAccessibleText = value => page.waitForFunction(
        expected => [...document.querySelectorAll('[aria-label]')]
            .some(element => element.getAttribute('aria-label')?.includes(expected)),
        {},
        value,
    );
    const canvas = await page.$('[role="img"][aria-label="Interactive GPU particle canvas"]');
    const rect = await canvas.boundingBox();
    assert.ok(rect && rect.width > 400 && rect.height > 300, 'Canvas viewport is visible');

    await capture('initial');
    await page.mouse.move(rect.x + rect.width * 0.55, rect.y + rect.height * 0.55);
    await page.mouse.down();
    await page.mouse.move(rect.x + rect.width * 0.68, rect.y + rect.height * 0.44, { steps: 8 });
    await page.mouse.up();
    await page.mouse.wheel({ deltaY: -240 });
    await page.waitForFunction(() => [...document.querySelectorAll('[aria-label]')]
        .map(element => element.getAttribute('aria-label'))
        .some(label => {
            const match = label?.match(/(\d+\.\d{2})×/);
            return match && Number.parseFloat(match[1]) > 1.05;
        }),
        { timeout: 10000 });
    await click('Zoom +');
    await capture('panned-zoomed');

    await click('Test recovery');
    await waitForAccessibleText('simulated failure');
    await capture('diagnostic-placeholder');
    await click('Recover canvas');
    await waitForAccessibleText('recovered');

    await click('Pause animation');
    await new Promise(resolve => setTimeout(resolve, 400));
    const before = await page.evaluate(() => window.arguiFrameCount);
    await new Promise(resolve => setTimeout(resolve, 700));
    const after = await page.evaluate(() => window.arguiFrameCount);
    assert.equal(after, before, 'Paused canvas requests no animation frames');

    await page.setViewport({ width: 980, height: 700, deviceScaleFactor: 2 });
    await click('Resume animation');
    await new Promise(resolve => setTimeout(resolve, 350));
    await capture('hidpi-resized-resumed');
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ screenshots: output, pausedFrames: after - before }, null, 2));
} finally {
    await browser.close();
}
