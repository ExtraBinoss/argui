// Requires a running web gallery built with --features webview.
import assert from 'node:assert/strict';
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const puppeteer = imported.puppeteer ?? imported.default;
const browser = await puppeteer.launch({
    executablePath: process.env.CHROME_PATH,
    headless: true,
    args: ['--no-sandbox', '--enable-unsafe-webgpu', '--enable-unsafe-swiftshader', '--use-angle=swiftshader'],
});
let page;
try {
    page = await browser.newPage();
    // Do not tie DOM checks to the headless compositor's animation cadence.
    const wait = (predicate, options = {}, ...args) => page.waitForFunction(predicate, { polling: 50, ...options }, ...args);
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    await page.setViewport({ width: 1280, height: 1000 });
    await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8081/widgets/', { waitUntil: 'networkidle0' });
    await page.waitForSelector('button[aria-label="WebView"]');
    // Invoke Argui's accessibility actions through the actual WASM event loop.
    const click = async label => {
        await page.waitForSelector(`[role="button"][aria-label="${label}"], [role="tab"][aria-label="${label}"]`);
        await page.evaluate(label => {
        const control = document.querySelector(`[role="button"][aria-label="${label}"], [role="tab"][aria-label="${label}"]`);
        if (!control) throw new Error(`Missing control: ${label}`);
        control.click();
        }, label);
    };
    await click('WebView');
    await wait(() => document.querySelector('iframe[title="Email content"]')?.contentDocument?.body?.textContent.includes('Message 1'));
    await page.evaluate(() => { window.emailFrame = document.querySelector('iframe[title="Email content"]'); });
    await click('Next email');
    await wait(() => emailFrame.contentDocument?.body?.textContent.includes('Message 2'));
    assert.equal(await page.evaluate(() => emailFrame === document.querySelector('iframe[title="Email content"]')), true);
    await page.evaluate(() => emailFrame.contentDocument.querySelector('a').click());
    await wait(() => {
        const frame = document.querySelector('iframe[title="Webpage"]');
        return frame?.src === 'https://example.com/' && frame.style.display === 'block';
    });
    assert.equal(await page.evaluate(() => emailFrame.style.display), 'none');
    await click('Email');
    await wait(() => emailFrame.style.display === 'block');
    const width = await page.evaluate(() => emailFrame.getBoundingClientRect().width);
    await page.setViewport({ width: 1000, height: 900 });
    await wait(width => emailFrame.getBoundingClientRect().width < width, {}, width);
    assert.equal(await page.evaluate(() => emailFrame.contentDocument.body.textContent.includes('Message 2')), true);
    await click('Button');
    await wait(() => [...document.querySelectorAll('iframe[data-argui-webview]')].every(frame => frame.style.display === 'none'));
    await wait(() => document.querySelectorAll('iframe[data-argui-webview]').length === 0, { timeout: 35000 });
    await click('WebView');
    await wait(() => document.querySelector('iframe[title="Email content"]')?.contentDocument?.body?.textContent.includes('Message 2'));
    assert.equal(await page.evaluate(() => emailFrame === document.querySelector('iframe[title="Email content"]')), false, 'eviction recreates only the DOM, not application state');
    if (process.env.GALLERY_RELAY_URL) {
        await click('Webpage');
        await click('Mode: isolated');
        await wait(relay => [...document.querySelectorAll('iframe[title="Webpage"]')].some(frame => frame.src === relay && frame.style.display === 'block'), {}, process.env.GALLERY_RELAY_URL);
        const relayHandle = await page.$(`iframe[src="${process.env.GALLERY_RELAY_URL}"]`);
        const relayFrame = await relayHandle.contentFrame();
        await relayFrame.waitForSelector('iframe');
        assert.equal(await relayFrame.$eval('iframe', frame => frame.sandbox.contains('allow-same-origin')), true);
        await click('Mode: compatible');
        await wait(() => [...document.querySelectorAll('iframe[title="Webpage"]')].some(frame => frame.src === 'https://example.com/' && frame.style.display === 'block'));
        await click('Mode: isolated');
        await wait(relay => [...document.querySelectorAll('iframe[title="Webpage"]')].some(frame => frame.src === relay && frame.style.display === 'block'), {}, process.env.GALLERY_RELAY_URL);
        assert.equal(await relayHandle.evaluate(frame => frame.isConnected), true, 'compatible frame retained when switching mode');
    }
    assert.deepEqual(errors, []);
    console.log('Gallery WASM: Email, reload/reuse, external link routing, tabs, resize, idle eviction and configured compatibility toggle passed');
} catch (error) {
    console.error(await page.evaluate(() => ({
        frames: [...document.querySelectorAll('iframe')].map(frame => ({style: frame.style.cssText, source: frame.srcdoc.slice(-1500)})),
        controls: [...document.querySelectorAll('[aria-label]')].map(node => [node.getAttribute('aria-label'), node.getAttribute('aria-description')]).filter(([label, description]) => description || /Message|Email|email|WebView/.test(label)),
    })));
    throw error;
} finally { await browser.close(); }
