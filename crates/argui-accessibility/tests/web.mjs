import assert from 'node:assert/strict';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Use scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH, headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const pause = () => new Promise(resolve => setTimeout(resolve, 350));
try {
    const page = await browser.newPage();
    await page.setViewport({ width: 1220, height: 900 });
    await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:3100/gallery/index.html', { waitUntil: 'networkidle0' });
    await page.waitForFunction(() => document.querySelector('#status')?.hidden, { timeout: 90_000 });
    await page.$eval('button[aria-label="VList"]', element => element.click());
    await page.waitForSelector('[role="option"][aria-label="Item 1"]');
    await pause();
    await page.evaluate(() => {
        window.stableButton = document.querySelector('button[aria-label="Button"]');
        window.search = document.querySelector('input[aria-label="Search components"]');
        window.search.focus();
    });
    await pause();
    await page.evaluate(() => {
        window.navigationMoves = 0;
        window.rowChanges = 0;
        window.observer = new MutationObserver(records => {
            for (const record of records) {
                for (const node of [...record.addedNodes, ...record.removedNodes]) {
                    if (node === window.stableButton || node.contains(window.stableButton)) window.navigationMoves++;
                    if (node.nodeType === 1 && (node.matches('[role="option"]') || node.querySelector('[role="option"]'))) window.rowChanges++;
                }
            }
        });
        window.observer.observe(document.querySelector('[data-argui-accessibility]'), { childList: true, subtree: true });
    });
    const bounds = await page.$eval('[role="listbox"][aria-label="data"]', element => element.getBoundingClientRect().toJSON());
    await page.mouse.move(bounds.x + 100, bounds.y + 100);
    for (let step = 0; step < 12; step++) {
        await page.mouse.wheel({ deltaY: 240 });
        await pause();
    }
    const result = await page.evaluate(() => {
        window.observer.disconnect();
        const rows = [...document.querySelectorAll('[role="option"]')];
        return {
            moves: window.navigationMoves, rowChanges: window.rowChanges,
            focused: document.activeElement === window.search,
            stable: document.querySelector('button[aria-label="Button"]') === window.stableButton,
            rows: rows.map(row => Number(row.getAttribute('aria-label').replace('Item ', ''))),
            bounds: rows.map(row => row.getBoundingClientRect().y).filter(y => y > 0),
        };
    });
    assert.equal(result.moves, 0, 'Scrolling must not detach unchanged navigation');
    assert.ok(result.rowChanges > 0, 'The virtual list must actually update its mounted window');
    assert.equal(result.stable, true);
    assert.equal(result.focused, true, 'Assistive DOM focus must survive list updates');
    assert.ok(result.rows.length > 0 && result.rows.length < 60);
    assert.ok(result.rows[0] > 1);
    assert.deepEqual(result.rows, [...result.rows].sort((a, b) => a - b), 'DOM order follows semantic order');
    assert.deepEqual(result.bounds, [...result.bounds].sort((a, b) => a - b), 'Bounds stay relative to semantic parents');
    await page.$eval('button[aria-label="Button"]', element => element.click());
    await page.waitForSelector('button[aria-label="Primary"]');
    assert.equal((await page.$$('[role="option"]')).length, 0, 'Old list nodes are removed');
    await page.$eval('button[aria-label="Primary"]', element => element.click());
    await page.waitForSelector('[aria-label="Button activations: 1"]');
    console.log('Passed: stable DOM, focus, virtual row ordering, bounds, removal and accessibility actions.', result);
} finally { await browser.close(); }
