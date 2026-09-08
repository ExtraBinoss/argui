import assert from 'node:assert/strict';
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const puppeteer = imported.puppeteer ?? imported.default;
const browser = await puppeteer.launch({
    executablePath: process.env.CHROME_PATH,
    headless: process.env.HEADLESS !== 'false',
    args: ['--no-sandbox', '--enable-unsafe-webgpu', '--enable-unsafe-swiftshader', '--use-angle=swiftshader'],
});
try {
    const page = await browser.newPage();
    await page.setViewport({ width: 1400, height: 1800 });
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8793/widgets/', { waitUntil: 'networkidle0' });
    await page.waitForSelector('button[aria-label="Custom Timeline"]');
    await page.$eval('button[aria-label="Custom Timeline"]', button => button.click());
    const handles = '[role="slider"][aria-label="Clip 1 duration"]';
    await page.waitForFunction(selector => document.querySelectorAll(selector).length === 2, {}, handles);
    const values = () => page.$$eval(handles, nodes => nodes.map(node => node.getAttribute('aria-valuenow')));
    assert.deepEqual(await values(), ['4', '4']);
    await page.$eval(handles, element => element.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true })));
    await page.waitForFunction(selector => [...document.querySelectorAll(selector)].every(node => node.getAttribute('aria-valuenow') === '4.5'), {}, handles);
    const beforeZoom = await page.$eval(handles, node => node.getBoundingClientRect().x);
    await page.$$eval('button[aria-label="Zoom +"]', buttons => buttons[0].click());
    await page.waitForFunction((selector, previous) => document.querySelector(selector).getBoundingClientRect().x !== previous, {}, handles, beforeZoom);
    const widths = await page.$$eval(handles, nodes => nodes.map(node => node.getBoundingClientRect().width));
    assert.deepEqual(widths, [12, 12], 'handle hit widths stay usable independently of zoom');
    const handle = await page.$eval(handles, node => {
        const rect = node.getBoundingClientRect();
        return { x: rect.x + rect.width / 2, y: rect.y + rect.height / 2 };
    });
    await page.mouse.move(handle.x, handle.y);
    await page.mouse.down();
    await page.mouse.move(handle.x + 50, handle.y + 70, { steps: 5 });
    await page.mouse.up();
    await page.waitForFunction(selector => [...document.querySelectorAll(selector)].every(node => node.getAttribute('aria-valuenow') === '5.5'), {}, handles);
    await page.$eval('button[aria-label="Button"]', button => button.click());
    await page.waitForFunction(selector => document.querySelectorAll(selector).length === 0, {}, handles);
    assert.deepEqual(errors, []);
    console.log('Custom Timeline Web: two shared views, semantic resize, captured pointer resize, zoom and unmount passed');
    if (process.env.SCREENSHOT_PATH) {
        await page.$eval('button[aria-label="Custom Timeline"]', button => button.click());
        await page.waitForSelector(handles);
        await page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
        await page.screenshot({ path: process.env.SCREENSHOT_PATH });
    }
} finally {
    await browser.close();
}
