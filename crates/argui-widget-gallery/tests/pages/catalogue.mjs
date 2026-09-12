import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Use scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH, headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const output = process.env.SCREENSHOT_DIR ?? 'target/catalogue-interactions';
await mkdir(output, { recursive: true });
const pause = (ms = 200) => new Promise(resolve => setTimeout(resolve, ms));
const pages = ['Accordion', 'Alert dialog', 'Attachment', 'Bubble', 'Button group', 'Carousel', 'Chart',
    'Combobox', 'Direction', 'Drawer', 'Field', 'Hover card', 'Input group', 'Input OTP', 'Item', 'Marker',
    'Message', 'Message scroller', 'Native select', 'Navigation menu', 'Questionnaire', 'Scroll area',
    'Sheet', 'Sidebar', 'Toggle', 'Toggle group', 'Typography & selection'];
try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    const button = label => `button[aria-label="${label}"]`;
    const rect = selector => page.$eval(selector, element => element.getBoundingClientRect().toJSON());
    const click = async selector => {
        const r = await rect(selector);
        assert.ok(r.width > 0 && r.height > 0, selector);
        await page.mouse.click(r.x + r.width / 2, r.y + r.height / 2); await pause();
    };
    const navigate = async label => {
        await page.$eval(button(label), element => element.click()); await pause(300);
    };
    const capture = async name => {
        const png = await page.screenshot({ path: `${output}/${name}.png` });
        assert.ok(png.length > 15000, 'Blank capture');
    };
    const state = (selector, name) => page.$eval(selector, (element, name) => element.getAttribute(name), name);
    const value = selector => page.$eval(selector, element => element.value ?? element.getAttribute('aria-valuetext'));
    const focused = async selector => assert.equal(await state('canvas', 'aria-activedescendant'), await state(selector, 'id'));
    for (const width of [1220, 800]) {
        const height = width === 1220 ? 780 : 720;
        await page.setViewport({ width, height });
        for (const scheme of ['light', 'dark']) {
            await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: scheme }]);
            await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8793/widgets/', { waitUntil: 'networkidle0' });
            await page.waitForSelector(button('Accordion'));
            for (const label of pages) {
                await navigate(label);
                await capture(`${scheme}-${width}-${label.toLowerCase().replaceAll(/[^a-z0-9]+/g, '-')}`);
            }
            await navigate('Toggle');
            await click(button('Bold'));
            assert.equal(await state(button('Bold'), 'aria-pressed'), 'true');
            await page.keyboard.press('Space'); await pause();
            assert.equal(await state(button('Bold'), 'aria-pressed'), 'false');
            await navigate('Toggle group');
            await click(button('Bold')); await page.keyboard.press('ArrowRight'); await pause();
            await focused(button('Italic'));
            await page.keyboard.press('Space'); await pause();
            assert.equal(await state(button('Italic'), 'aria-pressed'), 'true');
            await navigate('Accordion');
            await click(button('Can I use the keyboard?'));
            assert.equal(await state(button('Can I use the keyboard?'), 'aria-expanded'), 'true');
            await capture(`${scheme}-${width}-accordion-open`);
            await navigate('Field');
            await click('[role="textbox"]'); await page.keyboard.type('invalid'); await pause();
            assert.equal(await state('[role="textbox"]', 'aria-invalid'), 'true');
            assert.ok(await state('[role="textbox"]', 'aria-describedby'));
            await capture(`${scheme}-${width}-field-error`);
            await navigate('Input OTP');
            await click('[role="textbox"]'); await page.keyboard.type('12a34567'); await pause();
            assert.equal(await value('[role="textbox"]'), '123456');
            await capture(`${scheme}-${width}-otp-complete`);
            await navigate('Combobox');
            await click('[role="combobox"]'); await page.keyboard.type('sv'); await pause();
            await page.keyboard.press('ArrowDown'); await page.keyboard.press('Enter'); await pause();
            assert.equal(await value('[role="combobox"]'), 'Svelte');
            assert.equal(await state('[role="combobox"]', 'aria-expanded'), 'false');
            await navigate('Native select');
            await click('[role="combobox"]'); await pause();
            await page.keyboard.press('ArrowDown'); await page.keyboard.press('Enter'); await pause();
            assert.equal(await value('[role="combobox"]'), 'Français');
            await navigate('Alert dialog');
            await click(button('Archive project'));
            await page.waitForSelector('[role="alertdialog"]');
            await focused(button('Cancel'));
            await capture(`${scheme}-${width}-alert-dialog-open`);
            await page.keyboard.press('Escape'); await pause();
            await focused(button('Archive project'));
            await navigate('Sheet');
            await click(button('Open settings'));
            await page.waitForSelector('[role="dialog"]');
            await capture(`${scheme}-${width}-sheet-open`);
            await click(button('Done'));
            await navigate('Drawer');
            await click(button('Open project details'));
            await page.waitForSelector('[role="dialog"]');
            await capture(`${scheme}-${width}-drawer-open`);
            await page.keyboard.press('Escape'); await pause();
            await navigate('Carousel');
            await click(button('Next slide'));
            await capture(`${scheme}-${width}-carousel-next`);
            await navigate('Chart');
            await click(button('Line chart'));
            await capture(`${scheme}-${width}-chart-line`);
            await navigate('Hover card');
            await click(button('Ada Lovelace')); await pause(550);
            await click(button('Follow')); await page.keyboard.press('Escape'); await pause();
            assert.equal(await page.$(button('Follow')), null);
            await navigate('Scroll area');
            const viewport = '[role="group"][aria-label="Documents"]';
            const before = await rect('[aria-label="Document 01 · Project notes"]');
            await click(viewport); await page.keyboard.press('End'); await pause();
            const after = await page.$('[aria-label="Document 01 · Project notes"]');
            if (after) assert.ok((await rect('[aria-label="Document 01 · Project notes"]')).y < before.y);
            await capture(`${scheme}-${width}-scroll-end`);
            await navigate('Message scroller');
            await click('[role="group"][aria-label="Conversation"]');
            await page.keyboard.press('Home'); await pause();
            await click(button('Receive a message'));
            await page.waitForSelector(button('Jump to latest (1)'));
            await capture(`${scheme}-${width}-message-unread`);
            await click(button('Jump to latest (1)'));
            assert.equal(await page.$(button('Jump to latest (0)')), null);
            await navigate('Sidebar'); await click(button('Toggle navigation'));
            await capture(`${scheme}-${width}-sidebar-collapsed`);
            await navigate('Navigation menu');
            const guide = await rect(button('Guide'));
            const examples = await rect('[role="link"][aria-label="Examples"]');
            assert.ok(guide.right + 3 <= examples.x, 'Panel trigger reserves its width');
            await click(button('Guide'));
            await capture(`${scheme}-${width}-navigation-menu-open`);
            await click(button('Open getting started'));
            console.log(`${scheme} ${width}: catalogue, keyboard, editors, focus and panels passed`);
        }
    }
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ screenshots: output, errors }));
} finally { await browser.close(); }
