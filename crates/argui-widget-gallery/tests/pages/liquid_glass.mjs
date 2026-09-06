import assert from 'node:assert/strict';
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const puppeteer = imported.puppeteer ?? imported.default;
const browser = await puppeteer.launch({ executablePath: process.env.CHROME_PATH, headless: true,
    args: ['--no-sandbox', '--enable-unsafe-webgpu', '--enable-unsafe-swiftshader', '--use-angle=swiftshader'] });
try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8081/widgets/', { waitUntil: 'networkidle0' });
    const click = async label => {
        const selector = `button[aria-label="${label}"]`;
        await page.waitForSelector(selector);
        await page.$eval(selector, button => button.click());
    };
    await click('GPU effects / WGSL');
    const increment = async (label, expected) => {
        const selector = `[role="slider"][aria-label="${label}"]`;
        await page.waitForSelector(selector);
        await page.$eval(selector, slider => slider.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true })));
        await page.waitForFunction((selector, expected) => Number(document.querySelector(selector)?.getAttribute('aria-valuenow')) === expected, {}, selector, expected);
    };
    await increment('Refraction', 13);
    await increment('Blur', 2.25);
    await increment('Tint amount', Math.fround(0.13));
    await click('Octaves: 3');
    await page.waitForSelector('button[aria-label="Octaves: 4"]');
    await click('Seed: 0');
    await page.waitForSelector('button[aria-label="Seed: 1"]');
    await click('Tint: blue');
    await click('Reset settings');
    await page.waitForFunction(() => Number(document.querySelector('[role="slider"][aria-label="Blur"]')?.getAttribute('aria-valuenow')) === 2);
    await click('Effect: on');
    await click('Effect: off');
    await page.waitForSelector('button[aria-label="Effect: on"]');
    assert.deepEqual(errors, []);
    console.log('Gallery Liquid Glass: WASM initialization, shader registration and live controls passed');
} finally { await browser.close(); }
