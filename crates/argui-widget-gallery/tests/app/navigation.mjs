import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Use scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH, headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const output = process.env.SCREENSHOT_DIR ?? 'target/navigation-interactions';
await mkdir(output, { recursive: true });
const pause = (ms = 200) => new Promise(resolve => setTimeout(resolve, ms));
try {
    for (const scheme of ['light', 'dark']) {
        const page = await browser.newPage();
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await page.setViewport({ width: 1220, height: 900 });
        await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: scheme }]);
        await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8793/widgets/', { waitUntil: 'networkidle0' });
        const search = 'input[aria-label="Search components"]';
        const button = label => `button[aria-label="${label}"]`;
        const value = selector => page.$eval(selector, el => el.value);
        const click = async selector => {
            const r = await page.$eval(selector, el => el.getBoundingClientRect().toJSON());
            await page.mouse.click(r.x + r.width / 2, r.y + r.height / 2);
            await pause();
        };
        const navigate = async label => {
            await page.$eval(button(label), el => el.click());
            await pause();
        };
        const chord = async key => {
            await page.keyboard.down('Control'); await page.keyboard.press(key); await page.keyboard.up('Control');
            await pause();
        };
        const escape = async () => {
            await page.keyboard.press('Escape'); await pause();
            assert.equal(await value(search), '');
        };
        const capture = async name => {
            const png = await page.screenshot({ path: `${output}/${scheme}-${name}.png` });
            assert.ok(png.length > 15000, 'Blank capture');
            return png;
        };
        const assertLabelInk = async (png, label, light) => {
            const rect = await page.$eval(button(label), el => el.getBoundingClientRect().toJSON());
            const count = await page.evaluate(async ({ data, rect, light }) => {
                const bitmap = await createImageBitmap(await (await fetch(`data:image/png;base64,${data}`)).blob());
                const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
                const ctx = canvas.getContext('2d');
                ctx.drawImage(bitmap, 0, 0);
                const pixels = ctx.getImageData(rect.x + 14, rect.y + rect.height / 2 - 9, 55, 18).data;
                let count = 0;
                for (let i = 0; i < pixels.length; i += 4) {
                    if (light ? Math.min(...pixels.slice(i, i + 3)) > 220 : Math.max(...pixels.slice(i, i + 3)) < 90) count++;
                }
                return count;
            }, { data: Buffer.from(png).toString('base64'), rect, light });
            assert.ok(count > 15, `${label}: glyph colors must reach the rendered frame`);
        };
        await page.waitForSelector(search);
        await page.bringToFront();
        // Focus the browser's application surface, without choosing an Argui control.
        await page.$eval('canvas', el => el.focus());
        await page.keyboard.type('popover', { delay: 40 }); await pause();
        assert.equal(await value(search), 'popover', 'Typing immediately after launch keeps every letter in order');
        assert.equal(await page.$eval('canvas', el => el.getAttribute('aria-activedescendant')), await page.$eval(search, el => el.id));
        assert.ok(await page.$(button('Popover')));
        assert.equal(await page.$(button('Button')), null);
        await capture('direct-search');
        await page.keyboard.press('Enter'); await pause();
        assert.ok(await page.$(button('Sharing settings')));
        await escape();
        await click(button('Button'));
        await page.keyboard.type('slider', { delay: 30 }); await pause();
        assert.equal(await value(search), 'slider', 'Typing from a component button starts a new search');
        await page.keyboard.press('Backspace'); await pause();
        assert.equal(await value(search), 'slide');
        await escape();
        await page.mouse.click(1100, 820);
        await page.keyboard.type('tooltip', { delay: 30 }); await pause();
        assert.equal(await value(search), 'tooltip', 'Typing from empty page space searches too');
        await escape();
        await click(button('Button'));
        await chord('a');
        assert.equal(await value(search), '', 'Ctrl+A is not a search');
        for (const [label, selector, text] of [
            ['Input & Search', 'input[aria-label="Full name"]', 'Typed in the field'],
            ['Text area', 'textarea[role="textbox"]', 'First line\nSecond line'],
        ]) {
            await navigate(label); await click(selector); await chord('a');
            await page.keyboard.type(text, { delay: 20 }); await pause();
            assert.equal(await value(selector), text);
            assert.equal(await value(search), '', `${label}: editor keeps its typing`);
            await capture(label === 'Text area' ? 'textarea' : 'input');
        }
        await navigate('Select');
        await click(button('Choose a backend'));
        await page.keyboard.type('v'); await pause();
        assert.equal(await value(search), '', 'Select keeps its typeahead');
        await page.keyboard.press('Escape'); await pause();
        await navigate('Button');
        await click(button('Badge')); await click(button('Button')); await click(button('Card'));
        const selection = await capture('selection');
        await assertLabelInk(selection, 'Card', true);
        await assertLabelInk(selection, 'Button', scheme === 'dark');
        await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: scheme }, { name: 'prefers-reduced-motion', value: 'reduce' }]);
        await click(button('Button')); await capture('reduced-motion');
        assert.deepEqual(errors, []);
        console.log(`${scheme}: direct search, focus, shortcuts, editors, typeahead and selection passed`);
        await page.close();
    }
} finally { await browser.close(); }
