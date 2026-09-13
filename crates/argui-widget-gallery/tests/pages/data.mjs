import assert from 'node:assert/strict';
import { mkdir, writeFile } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Use scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH, headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const output = process.env.SCREENSHOT_DIR ?? 'target/web-vlist';
const label = process.env.PROFILE_LABEL ?? 'current';
await mkdir(output, { recursive: true });
const measurements = [];
try {
    for (let run = 0; run < 2; run++) {
        const page = await browser.newPage();
        await page.setViewport({ width: 1220, height: 900 });
        await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: 'light' }]);
        await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:3100/gallery/index.html', { waitUntil: 'networkidle0' });
        await page.waitForSelector('button[aria-label="VList"]');
        await page.$eval('button[aria-label="VList"]', element => element.click());
        await page.waitForSelector('[role="listbox"][aria-label="data"]');
        await new Promise(resolve => setTimeout(resolve, 1500));
        const bounds = await page.$eval('[role="listbox"][aria-label="data"]', element => element.getBoundingClientRect().toJSON());
        const client = await page.createCDPSession();
        await client.send('Profiler.enable');
        await client.send('Profiler.setSamplingInterval', { interval: 1000 });
        await client.send('Profiler.start');
        await page.evaluate(() => {
            window.frameIntervals = [];
            window.longTasks = [];
            window.recordFrames = true;
            let previous;
            const frame = now => {
                if (!window.recordFrames) return;
                if (previous) window.frameIntervals.push(now - previous);
                previous = now;
                requestAnimationFrame(frame);
            };
            requestAnimationFrame(frame);
            window.longTaskObserver = new PerformanceObserver(list => window.longTasks.push(...list.getEntries().map(entry => entry.duration)));
            window.longTaskObserver.observe({ type: 'longtask' });
        });
        await page.bringToFront();
        await page.mouse.move(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2);
        for (let step = 0; step < 100; step++) {
            await page.mouse.wheel({ deltaY: 60 });
            await new Promise(resolve => setTimeout(resolve, 16));
        }
        const result = await page.evaluate(() => {
            window.recordFrames = false;
            window.longTaskObserver.disconnect();
            const intervals = window.frameIntervals.sort((a, b) => a - b);
            return {
                frames: intervals.length,
                medianMs: intervals[Math.floor(intervals.length / 2)],
                p95Ms: intervals[Math.floor(intervals.length * .95)],
                over32ms: intervals.filter(value => value > 32).length,
                longTasks: window.longTasks,
                mountedRows: document.querySelectorAll('[role="option"]').length,
                firstRow: document.querySelector('[role="option"]')?.getAttribute('aria-label'),
            };
        });
        const { profile } = await client.send('Profiler.stop');
        await writeFile(`${output}/${label}-${run}.cpuprofile`, JSON.stringify(profile));
        assert.ok(result.mountedRows > 0 && result.mountedRows < 60);
        console.log('scroll result', bounds, result);
        assert.notEqual(result.firstRow, 'Item 1', 'The list must actually scroll');
        const png = await page.screenshot({ path: `${output}/${label}-${run}.png` });
        assert.ok(png.length > 15000, 'Blank capture');
        measurements.push(result);
        console.log(label, run, JSON.stringify(result));
        await page.close();
    }
    await writeFile(`${output}/${label}.json`, JSON.stringify(measurements, null, 2));
} finally { await browser.close(); }
