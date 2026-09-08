import assert from 'node:assert/strict';
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const puppeteer = imported.puppeteer ?? imported.default;
const browser = await puppeteer.launch({
    executablePath: process.env.CHROME_PATH, headless: true,
    args: ['--no-sandbox', '--enable-unsafe-webgpu', '--enable-unsafe-swiftshader', '--use-angle=swiftshader'],
});
try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8093/widgets/', { waitUntil: 'networkidle0' });
    await page.waitForSelector('button[aria-label="Async tasks"]');
    await page.$eval('button[aria-label="Async tasks"]', button => button.click());
    await page.waitForSelector('input[aria-label="Draft search"]');
    const search = async query => page.$eval('input[aria-label="Draft search"]', (input, query) => {
        input.value = query;
        input.dispatchEvent(new Event('input', { bubbles: true }));
    }, query);
    await search('id:9998');
    await page.waitForSelector('[aria-label="2 matching drafts"]');
    await search('id:abc');
    await page.waitForSelector('[aria-label^="Invalid query:"]');
    await search('Design');
    await search('id:9999');
    await page.waitForSelector('[aria-label="1 matching drafts"]');
    assert.deepEqual(errors, []);
    console.log('Async gallery: browser completion without pointer input, error and latest-result policy passed');
} finally { await browser.close(); }
