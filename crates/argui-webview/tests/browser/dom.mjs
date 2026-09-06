// Run with node; PUPPETEER_MODULE may point at an installed module exporting puppeteer.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createServer } from 'node:http';
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const puppeteer = imported.puppeteer ?? imported.default;
const moduleSource = await readFile(new URL('../../src/browser/dom.js', import.meta.url));
let remoteRequests = 0;
const server = createServer((req, res) => {
    if (req.url === '/dom.js') { res.setHeader('Content-Type', 'text/javascript'); res.end(moduleSource); }
    else if (req.url === '/pixel') { remoteRequests++; res.end('pixel'); }
    else if (req.url === '/blocked') { res.setHeader('X-Frame-Options', 'DENY'); res.end('Blocked'); }
    else if (req.url === '/page') { res.end('<p>Browser page</p><script>window.ran = true</script>'); }
    else { res.setHeader('Content-Type', 'text/html'); res.end('<body style="margin:0;height:2000px"><canvas width="800" height="600" style="position:absolute;left:40px;top:50px;width:400px;height:300px"></canvas></body>'); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const base = `http://127.0.0.1:${server.address().port}`;
const browser = await puppeteer.launch({ executablePath: process.env.CHROME_PATH, headless: true, args: ['--no-sandbox'] });
try {
    const page = await browser.newPage();
    await page.setViewport({ width: 800, height: 600, deviceScaleFactor: 2 });
    await page.goto(base);
    await page.evaluate(async base => {
        const {createView, createWakeTimer} = await import('/dom.js');
        window.events = [];
        window.view = createView(document.querySelector('canvas'), true, (kind, value) => events.push([kind, value]), 'allow-same-origin', null);
        window.makeDocument = label => `<!doctype html><meta http-equiv="Content-Security-Policy" content="default-src 'none'; script-src 'none'; img-src data:; style-src 'unsafe-inline'; form-action 'none'; base-uri 'none'"><body><p>${label}</p><a href="https://example.com">Link</a><img src="${base}/pixel"><script>parent.hacked=true<\/script><div style="height:1000px"></div></body>`;
        view.bounds(10, 20, 200, 160, 2);
        view.load(makeDocument('First'));
        view.visible(true);
        window.timer = createWakeTimer(() => events.push(['timer', '']));
    }, base);
    await page.waitForFunction(() => events.some(([kind]) => kind === 'loaded'));
    assert.deepEqual(await page.evaluate(() => {
        const frame = document.querySelector('iframe'), box = frame.getBoundingClientRect();
        return [box.x, box.y, box.width, box.height, frame.sandbox.value, !!window.hacked];
    }), [50, 70, 200, 160, 'allow-same-origin', false]);
    assert.equal(remoteRequests, 0, 'email must not fetch tracking pixels');
    await page.evaluate(() => document.querySelector('iframe').contentDocument.querySelector('a').click());
    assert.equal(await page.evaluate(() => events.filter(([kind]) => kind === 'navigation').length), 1);
    assert.equal(await page.evaluate(() => document.querySelector('iframe').contentWindow.location.href), 'about:srcdoc');
    await page.evaluate(() => {
        window.retained = document.querySelector('iframe');
        retained.contentWindow.scrollTo(0, 80);
        view.visible(false); view.visible(true);
    });
    assert.equal(await page.evaluate(() => retained === document.querySelector('iframe') && retained.contentWindow.scrollY === 80), true);
    await page.evaluate(() => view.clip(10, 60, 200, 100));
    assert.equal(await page.evaluate(() => document.elementFromPoint(60, 80).tagName), 'CANVAS');
    assert.equal(await page.evaluate(() => document.elementFromPoint(60, 120).tagName), 'IFRAME');
    await page.evaluate(() => { events.length = 0; view.load(makeDocument('Second')); });
    await page.waitForFunction(() => events.some(([kind]) => kind === 'loaded'));
    assert.equal(await page.evaluate(() => document.querySelector('iframe').contentDocument.body.textContent.includes('Second')), true);
    await page.evaluate(() => document.querySelector('iframe').contentDocument.querySelector('a').click());
    assert.equal(await page.evaluate(() => events.filter(([kind]) => kind === 'navigation').length), 1, 'reload does not duplicate listeners');
    await page.evaluate(() => timer.schedule(1));
    await page.waitForFunction(() => events.some(([kind]) => kind === 'timer'));
    await page.evaluate(() => { events.length = 0; timer.schedule(10); timer.dispose(); view.dispose(); view.dispose(); });
    assert.equal(await page.$('iframe'), null);
    await page.evaluate(async base => {
        const {createView} = await import('/dom.js');
        window.view = createView(document.querySelector('canvas'), false, (kind, value) => events.push([kind, value]), 'allow-scripts allow-forms', null);
        view.bounds(0, 0, 200, 200, 2); view.load(`${base}/page`); view.visible(true);
    }, base);
    await page.waitForFunction(() => document.querySelector('iframe').getAttribute('src').endsWith('/page'));
    const child = await (await page.$('iframe')).contentFrame();
    await child.waitForFunction(() => window.ran === true);
    assert.equal(await child.evaluate(() => { try { return !!parent.document; } catch { return false; } }), false, 'webpage has no app-origin access');
    assert.equal(await page.evaluate(() => events.some(([kind]) => kind === 'timer')), false, 'disposed timers do not wake');
    await page.evaluate(base => { events.length = 0; view.load(`${base}/blocked`); }, base);
    await page.waitForFunction(() => document.querySelector('iframe').src.endsWith('/blocked'));
    assert.equal(await page.evaluate(() => events.some(([kind]) => kind === 'loaded')), false, 'never claim cross-origin content loaded successfully');
    await page.evaluate(() => view.dispose());
    console.log('Browser DOM: isolation, links, geometry, clipping, reuse, reload, disposal and timer checks passed');
} finally {
    await browser.close();
    await new Promise(resolve => server.close(resolve));
}
