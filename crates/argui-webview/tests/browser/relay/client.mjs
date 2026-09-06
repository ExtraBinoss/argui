// Exercises the real relay server and browser enforcement, not a simulated backend.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { createServer } from 'node:http';
import { fileURLToPath } from 'node:url';
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const puppeteer = imported.puppeteer ?? imported.default;
const dom = await readFile(new URL('../../../src/browser/dom.js', import.meta.url));
let protectedRequests = 0;
const server = createServer((req, res) => {
    res.setHeader('Content-Type', 'text/html');
    if (req.url === '/dom.js') { res.setHeader('Content-Type', 'text/javascript'); res.end(dom); }
    else if (req.url === '/redirect') { res.writeHead(302, { Location: `${app}/protected` }); res.end(); }
    else if (req.url === '/protected') { protectedRequests++; res.end('App secrets'); }
    else if (req.url === '/site') res.end(`<p>Real website</p><script>localStorage.setItem('visits', Number(localStorage.getItem('visits') || 0) + 1); window.ran = true;</script>`);
    else res.end('<canvas width="800" height="600"></canvas>');
});
await new Promise(resolve => server.listen(0, '0.0.0.0', resolve));
const port = server.address().port;
const app = `http://127.0.0.1:${port}`, site = `http://127.0.0.2:${port}`;
const reservation = createServer();
await new Promise(resolve => reservation.listen(0, '127.0.0.1', resolve));
const relayPort = reservation.address().port;
await new Promise(resolve => reservation.close(resolve));
const relay = `http://127.0.0.1:${relayPort}`;
const child = spawn('python3', [fileURLToPath(new URL('../../../src/browser/relay/server.py', import.meta.url)),
    '--app-origin', app, '--public-origin', relay, '--allowed-origin', site, '--port', String(relayPort)], { stdio: ['ignore', 'ignore', 'pipe'] });
let serverErrors = '';
child.stderr.on('data', chunk => { serverErrors += chunk; });
let browser;
try {
    let response;
    for (let attempt = 0; attempt < 100; attempt++) {
        try { response = await fetch(relay); break; } catch { await new Promise(resolve => setTimeout(resolve, 50)); }
    }
    assert.ok(response?.ok, serverErrors);
    assert.ok(response.headers.get('content-security-policy').includes(`frame-src ${site}`));
    browser = await puppeteer.launch({ executablePath: process.env.CHROME_PATH, headless: true, args: ['--no-sandbox', '--no-proxy-server'] });
    const page = await browser.newPage();
    await page.goto(app);
    await page.evaluate(async ({ relay, site }) => {
        const { createView } = await import('/dom.js');
        window.events = [];
        window.make = (url, email = false, sandbox = 'allow-scripts allow-forms allow-same-origin') => createView(document.querySelector('canvas'), email, (kind, value) => events.push([kind, value]), sandbox, url);
        window.view = make(relay);
        view.bounds(0, 0, 600, 400); view.visible(true); view.load(`${site}/site`);
    }, { relay, site });
    await page.waitForFunction(() => document.querySelector('iframe')?.contentWindow != null);
    const outer = await (await page.$('iframe')).contentFrame();
    await outer.waitForSelector('iframe');
    const website = await (await outer.$('iframe')).contentFrame();
    await website.waitForFunction(() => window.ran);
    assert.equal(await website.evaluate(() => localStorage.getItem('visits')), '1');
    assert.equal(await website.evaluate(site => window.open(`${site}/site`, '_blank') === null, site), true, 'popups blocked by default');
    assert.equal(await website.evaluate(() => { try { return !!top.document; } catch { return false; } }), false);
    await page.evaluate(() => { window.retained = document.querySelector('iframe'); view.visible(false); view.visible(true); });
    assert.equal(await page.evaluate(() => retained === document.querySelector('iframe')), true);
    await page.evaluate(site => view.load(`${site}/site`), site);
    await website.waitForFunction(() => localStorage.getItem('visits') === '2');
    // A forged parent message with an incorrect origin cannot change the inner frame.
    await outer.evaluate(app => window.dispatchEvent(new MessageEvent('message', {
        source: parent, origin: 'https://attacker.invalid', data: { protocol: 'argui-webview-v1', type: 'load', id: 123, url: `${app}/protected`, sandbox: 'allow-scripts allow-forms allow-same-origin' },
    })), app);
    assert.equal(await outer.$eval('iframe', frame => new URL(frame.src).pathname), '/site');
    await page.evaluate(app => view.load(`${app}/protected`), app);
    await page.waitForFunction(() => events.some(([kind, value]) => kind === 'error' && value.includes('not allowed')));
    assert.equal(protectedRequests, 0);
    // Even an allowed site's HTTP redirect cannot load the app in the relay.
    await outer.evaluate(() => {
        window.violations = [];
        document.addEventListener('securitypolicyviolation', event => violations.push(event.effectiveDirective));
    });
    await page.evaluate(site => view.load(`${site}/redirect`), site);
    await outer.waitForFunction(() => violations.includes('frame-src'));
    assert.equal(protectedRequests, 0, 'CSP must block redirects before requesting the app');
    assert.equal(await page.evaluate(() => events.some(([kind]) => kind === 'loaded')), false);
    assert.equal(await page.evaluate(app => {
        try { make(app); return false; } catch { return true; }
    }, app), true, 'same-origin relay rejected');
    assert.equal(await page.evaluate(relay => {
        try { make(relay, true); return false; } catch { return true; }
    }, relay), true, 'email cannot enable scripts or relay mode');
    await page.evaluate(() => view.dispose());
    assert.equal(await page.$('iframe'), null);
    // Explicit external popups are real browser windows, not navigation callbacks.
    await page.evaluate(({ relay, site }) => {
        view = make(relay, false, 'allow-scripts allow-forms allow-same-origin allow-popups allow-popups-to-escape-sandbox');
        view.bounds(0, 0, 600, 400); view.visible(true); view.load(`${site}/site`);
    }, { relay, site });
    const popupOuter = await (await page.$('iframe')).contentFrame();
    await popupOuter.waitForSelector('iframe');
    const popupSite = await (await popupOuter.$('iframe')).contentFrame();
    await popupSite.waitForFunction(() => window.ran);
    const popupTarget = browser.waitForTarget(target => target.type() === 'page' && target.url() === `${site}/site`);
    await popupSite.evaluate(site => { window.open(`${site}/site`, '_blank', 'noopener'); }, site);
    const popup = await (await popupTarget).page();
    await popup.waitForFunction(() => window.ran);
    assert.equal(await popup.evaluate(() => window.opener), null);
    await popup.close();
    await page.evaluate(() => view.dispose());
    // A reachable endpoint that is not the relay must fail explicitly.
    await page.evaluate(site => { events.length = 0; view = make(site); view.load(`${site}/site`); }, site);
    await page.waitForFunction(() => events.some(([kind, value]) => kind === 'error' && value.includes('did not acknowledge')), { timeout: 15000 });
    await page.evaluate(() => view.dispose());
    console.log('Relay: real storage, retained session, reload, isolation, spoof rejection, allowlist, redirect CSP and disposal passed');
} finally {
    await browser?.close();
    child.kill();
    await new Promise(resolve => child.exitCode !== null || child.signalCode !== null ? resolve() : child.once('exit', resolve));
    await new Promise(resolve => server.close(resolve));
}
