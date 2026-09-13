import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Use scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH, headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const output = process.env.SCREENSHOT_DIR ?? 'target/updater-interactions';
await mkdir(output, { recursive: true });
const pause = (ms = 250) => new Promise(resolve => setTimeout(resolve, ms));
try {
    for (const scheme of ['light', 'dark']) {
        const page = await browser.newPage();
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await page.evaluateOnNewDocument(() => {
            const now = performance.now.bind(performance);
            let physical = now(), logical = physical, scale = 1;
            const clock = () => logical + (now() - physical) * scale;
            Object.defineProperty(performance, 'now', { value: clock });
            window.advanceUpdateClock = ms => { logical = clock() + ms; physical = now(); };
            window.freezeUpdateClock = () => { logical = clock(); physical = now(); scale = 0; };
        });
        await page.setViewport({ width: 1220, height: 780 });
        await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: scheme }]);
        await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8793/widgets/', { waitUntil: 'networkidle0' });
        const button = label => `button[aria-label="${label}"]`;
        await page.waitForSelector(button('Updater'));
        await page.$eval(button('Updater'), element => element.click());
        await pause();
        const click = async label => {
            const r = await page.$eval(button(label), element => element.getBoundingClientRect().toJSON());
            assert.ok(r.width > 0 && r.height > 0, label);
            await page.mouse.click(r.x + r.width / 2, r.y + r.height / 2);
            await pause();
        };
        const capture = async name => {
            const png = await page.screenshot({ path: `${output}/${scheme}-${name}.png` });
            assert.ok(png.length > 15000, 'Blank capture');
        };
        const step = async ms => { await page.evaluate(ms => window.advanceUpdateClock(ms), ms); await pause(); };
        await click('Preview update');
        await page.waitForSelector('[role="dialog"][aria-label="Application update"]');
        await capture('available');
        await page.keyboard.press('Escape'); await pause();
        assert.equal(await page.$('[role="dialog"]'), null);
        await click('Preview update');
        await page.evaluate(() => window.freezeUpdateClock());
        await click('Download update');
        await step(3000);
        await page.waitForSelector('[role="progressbar"][aria-valuenow="25"]');
        await capture('downloading');
        await click('Cancel download');
        await page.waitForSelector('[aria-label="Download cancelled. You can try again."]');
        await capture('cancelled');
        await click('Close');
        await click('Use unknown size');
        await click('Preview update');
        await click('Download update');
        await step(3000);
        const value = await page.$eval('[role="progressbar"]', element => element.getAttribute('aria-valuenow'));
        assert.equal(value, null);
        await capture('unknown-size');
        await step(9000);
        await capture('verifying');
        await step(750);
        await page.waitForSelector(button('Install update'));
        await capture('ready');
        await click('Install update');
        await step(750);
        await page.waitForSelector('[aria-label="Update installed. Restart the application to use it."]');
        await capture('installed');
        await click('Close');
        await click('Connection error');
        await page.waitForSelector('[role="alert"]');
        await capture('error');
        await click('Try again');
        await step(500);
        await page.waitForSelector(button('Download update'));
        await page.setViewport({ width: 800, height: 720 }); await pause(500);
        await capture('narrow');
        assert.deepEqual(errors, []);
        await page.close();
    }
    console.log('Updater dialog: actions, cancellation, unknown totals, retry, installation and captures passed.');
} finally { await browser.close(); }
