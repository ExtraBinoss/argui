import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Use scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH, headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const output = process.env.SCREENSHOT_DIR ?? 'target/animated-text-interactions';
await mkdir(output, { recursive: true });
const pause = (ms = 250) => new Promise(resolve => setTimeout(resolve, ms));
try {
    for (const scheme of ['dark', 'light']) {
        const page = await browser.newPage();
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        // Slow only the test clock so GPU readback can capture an intermediate frame.
        await page.evaluateOnNewDocument(() => {
            const now = performance.now.bind(performance);
            let physical = now(), logical = physical, scale = 1;
            const clock = () => logical + (now() - physical) * scale;
            Object.defineProperty(performance, 'now', { value: clock });
            window.setAnimationTestSpeed = speed => {
                logical = clock(); physical = now(); scale = speed;
            };
            window.advanceAnimationTestClock = ms => {
                logical = clock() + ms; physical = now();
            };
        });
        await page.setViewport({ width: 1220, height: 900 });
        await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: scheme }]);
        await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8793/widgets/', { waitUntil: 'networkidle0' });
        const button = label => `button[aria-label="${label}"]`;
        await page.waitForSelector(button('Animated text'));
        await page.$eval(button('Animated text'), el => el.click()); await pause(500);
        const rect = selector => page.$eval(selector, el => el.getBoundingClientRect().toJSON());
        const click = async label => {
            const r = await rect(button(label));
            assert.ok(r.width > 0 && r.bottom <= page.viewport().height, `Visible ${label}`);
            await page.mouse.click(r.x + r.width / 2, r.y + r.height / 2);
        };
        const capture = async name => {
            const png = await page.screenshot({ path: `${output}/${scheme}-${name}.png` });
            assert.ok(png.length > 15000, 'Blank capture');
            return Buffer.from(png).toString('base64');
        };
        const counters = value => page.$$(`[aria-label="${value}"]`);
        assert.equal((await counters('10')).length, 3);
        const before = await capture('10');
        const roll = await rect('[aria-label="10"]');
        await page.evaluate(() => window.setAnimationTestSpeed(.1));
        await click('+1'); await pause(250);
        const middle = await capture('10-to-11');
        const changes = await page.evaluate(async ({ before, middle, roll }) => {
            const pixels = async png => {
                const bitmap = await createImageBitmap(await (await fetch(`data:image/png;base64,${png}`)).blob());
                const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
                const cx = canvas.getContext('2d'); cx.drawImage(bitmap, 0, 0);
                return cx.getImageData(Math.round(roll.x), Math.round(roll.y), 55, Math.round(roll.height)).data;
            };
            const a = await pixels(before), b = await pixels(middle);
            let prefix = 0, unit = 0;
            for (let i = 0; i < a.length; i += 4) {
                if (a[i] !== b[i] || a[i+1] !== b[i+1] || a[i+2] !== b[i+2]) {
                    if ((i / 4) % 55 < 22) prefix++; else unit++;
                }
            }
            return { prefix, unit };
        }, { before, middle, roll });
        assert.equal(changes.prefix, 0, 'The unchanged tens digit stays still');
        assert.ok(changes.unit > 15, 'The changed units digit visibly animates');
        await page.evaluate(() => window.setAnimationTestSpeed(1));
        await pause(700);
        assert.equal((await counters('11')).length, 3);
        const final = await capture('11');
        assert.notEqual(middle, final, 'Intermediate capture differs from the completed frame');
        for (let i = 0; i < 4; i++) await click('+1');
        await pause(1200);
        assert.equal((await counters('15')).length, 3, 'Rapid updates settle at the latest requested value');
        await click('99 / 100'); await pause(600);
        await click('99 / 100'); await pause(100); await capture('carry'); await pause(600);
        assert.equal((await counters('100')).length, 3);
        await page.evaluate(() => window.setAnimationTestSpeed(0));
        await click('−1'); await pause(100);
        await page.evaluate(() => window.advanceAnimationTestClock(200)); await pause(100);
        await capture('reverse');
        await page.evaluate(() => window.advanceAnimationTestClock(219)); await pause(100);
        const nearEnd = await capture('reverse-near-end');
        await page.evaluate(() => window.advanceAnimationTestClock(1)); await pause(100);
        const completed = await capture('reverse-complete');
        const cleanupChanges = await page.evaluate(async ({ nearEnd, completed, roll }) => {
            const read = async png => {
                const bitmap = await createImageBitmap(await (await fetch(`data:image/png;base64,${png}`)).blob());
                const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
                const cx = canvas.getContext('2d'); cx.drawImage(bitmap, 0, 0);
                return cx.getImageData(roll.x, roll.y, roll.width, roll.height).data;
            };
            const a = await read(nearEnd), b = await read(completed);
            let changed = 0;
            for (let i = 0; i < a.length; i += 4) {
                if (Math.max(...[0, 1, 2].map(c => Math.abs(a[i + c] - b[i + c]))) > 10) changed++;
            }
            return changed;
        }, { nearEnd, completed, roll });
        assert.ok(cleanupChanges < 10, `100 → 99 settles before cleanup (${cleanupChanges} changed pixels)`);
        assert.equal((await counters('99')).length, 3);
        await click('Toggle saved status'); await pause(100);
        await page.evaluate(() => window.advanceAnimationTestClock(210)); await pause(100);
        await capture('status-fade');
        await click('Toggle saved status'); await pause(100); await capture('status-reverse');
        await page.evaluate(() => window.setAnimationTestSpeed(1)); await pause(600);
        assert.equal((await counters('Draft')).length, 1);
        await click('Toggle saved status'); await pause(600);
        assert.equal((await counters('Saved')).length, 1);
        await page.emulateMediaFeatures([
            { name: 'prefers-color-scheme', value: scheme },
            { name: 'prefers-reduced-motion', value: 'reduce' },
        ]);
        await click('Reset to 10'); await pause(100);
        assert.equal((await counters('10')).length, 3);
        await page.setViewport({ width: 800, height: 900 }); await pause(400);
        await capture('narrow-reduced-motion');
        assert.deepEqual(errors, []);
        console.log(`${scheme}: changed-digit pixels, roll/slide/fade, rapid updates, carry, reverse, reduced motion and narrow layout passed`);
        await page.close();
    }
} finally { await browser.close(); }
