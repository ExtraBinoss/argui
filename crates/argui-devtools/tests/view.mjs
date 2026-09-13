import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Use scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH, headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const output = process.env.SCREENSHOT_DIR ?? 'target/devtools-interactions';
await mkdir(output, { recursive: true });
const pause = (ms = 250) => new Promise(resolve => setTimeout(resolve, ms));
try {
    for (const scheme of ['dark', 'light']) {
        const page = await browser.newPage();
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await page.setViewport({ width: 1220, height: 1000 });
        await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: scheme }]);
        await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8793/widgets/', { waitUntil: 'networkidle0' });
        const button = label => `button[aria-label="${label}"]`;
        const rect = selector => page.$eval(selector, el => el.getBoundingClientRect().toJSON());
        const value = selector => page.$eval(selector, el => el.value);
        let scrollX = 1100;
        const click = async selector => {
            let r = await rect(selector);
            for (let attempt = 0; (r.width < 20 || r.height < 15 || r.bottom > page.viewport().height) && attempt < 8; attempt++) {
                await page.mouse.move(Math.min(page.viewport().width - 30, r.width < 20 ? scrollX : r.x + r.width / 2), page.viewport().height - 80);
                await page.mouse.wheel({ deltaY: Math.max(150, r.bottom - page.viewport().height + 40) });
                await pause(); r = await rect(selector);
            }
            assert.ok(r.bottom <= page.viewport().height, `Control scrolled into view: ${selector}`);
            assert.ok(r.width >= 20 && r.height >= 15, `Visible control: ${selector}`);
            scrollX = r.x + r.width / 2;
            await page.mouse.click(scrollX, r.y + r.height / 2);
            await pause();
        };
        const chord = async key => {
            await page.keyboard.down('Control'); await page.keyboard.press(key); await page.keyboard.up('Control'); await pause();
        };
        const fill = async (selector, text) => {
            await click(selector); await chord('a'); await page.keyboard.type(text, { delay: 25 }); await pause();
        };
        const capture = async name => {
            const png = await page.screenshot({ path: `${output}/${scheme}-${name}.png` });
            assert.ok(png.length > 15000, 'Blank capture');
            return Buffer.from(png).toString('base64');
        };
        const pixel = (png, x, y) => page.evaluate(async ({ png, x, y }) => {
            const bitmap = await createImageBitmap(await (await fetch(`data:image/png;base64,${png}`)).blob());
            const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
            const cx = canvas.getContext('2d'); cx.drawImage(bitmap, 0, 0);
            return [...cx.getImageData(x, y, 1, 1).data].slice(0, 3);
        }, { png, x, y });
        await page.waitForSelector(button('Color picker'));
        // Navigate a virtualized offscreen sidebar entry through its accessible action.
        await page.$eval(button('Color picker'), el => el.click()); await pause();
        const hex = 'input[aria-label="Accent color HEX"]';
        await fill(hex, '#FF0000FF');
        for (const [mode, field] of [['RGB', 'R'], ['HSL', 'H °'], ['HSV', 'H °'], ['HEX', 'HEX']]) {
            await click(button(mode)); assert.ok(await page.$(`input[aria-label="Accent color ${field}"]`));
        }
        const pad = await rect('[role=group][aria-label^="Saturation "]');
        const color = await capture('color-picker');
        const center = await pixel(color, pad.x + pad.width / 2, pad.y + pad.height / 2);
        assert.ok(center.every((value, i) => Math.abs(value - [128, 64, 64][i]) < 5), `sRGB bilinear fill: ${center}`);
        await page.mouse.move(pad.x + pad.width * .75, pad.y + pad.height * .25);
        await page.mouse.down(); await pause();
        await page.mouse.move(pad.x + pad.width * .5, pad.y + pad.height * .5, { steps: 5 }); await pause();
        await page.mouse.up(); await pause();
        assert.equal(await value(hex), '#804040FF', 'Captured pad gesture reaches the mounted page');
        await click('[role=slider][aria-label="Opacity"]'); await page.keyboard.press('End'); await pause();
        await page.keyboard.press('ArrowLeft'); await pause();
        assert.equal((await value(hex)).slice(-2), 'FC', 'Opacity is keyboard editable');
        await fill(hex, '#804040FF');
        await click(button('DevTools')); await pause(500);
        const main = '[role=separator][aria-orientation=horizontal]';
        let split = await rect(main);
        await page.mouse.move(split.x + split.width / 2, split.y + 3); await page.mouse.down();
        await page.mouse.move(split.x + split.width / 2, 400, { steps: 12 }); await page.mouse.up(); await pause();
        assert.ok(Number(await page.$eval(main, el => el.getAttribute('aria-valuenow'))) > 500);
        const filter = 'input[aria-label="Filter elements"]';
        await click('[role=treeitem]'); await page.keyboard.type('color-preview', { delay: 50 }); await pause();
        assert.equal(await value(filter), 'color-preview', 'Type-to-filter preserves every character');
        assert.equal(await page.$eval('canvas', el => el.getAttribute('aria-activedescendant')), await page.$eval(filter, el => el.id));
        const row = '[role=treeitem][aria-label*="#color-preview-swatch"]';
        const selected = () => page.$$eval('[role=treeitem][aria-selected=true]', es => es.map(el => el.getAttribute('aria-label')));
        await page.mouse.move(1100, 450); await pause();
        const before = await capture('before-hover'); const selection = await selected();
        const r = await rect(row); await page.mouse.move(r.x + 200, r.y + 14); await pause();
        const hover = await capture('hover');
        assert.deepEqual(await selected(), selection, 'Hover leaves selection unchanged');
        assert.notDeepEqual(await pixel(before, 1100, 150), await pixel(hover, 1100, 150), 'Application pixels show hovered bounds');
        await click(row); await pause();
        const properties = '[role=separator][aria-orientation=vertical]';
        split = await rect(properties);
        await page.mouse.move(split.x + 3, split.y + 40); await page.mouse.down();
        await page.mouse.move(split.x - 100, split.y + 40, { steps: 10 }); await page.mouse.up(); await pause();
        assert.ok((await rect(properties)).x < split.x - 80);
        await fill('input[aria-label="width px"]', '210');
        assert.equal(await value('input[aria-label="width px"]'), '210');
        assert.equal(await value(filter), '', 'Property input remains in the editor');
        await capture('properties-resized');
        await click(button('Layout')); await pause(400);
        await click('button[aria-label^="#"]'); await pause();
        await fill('input[aria-label="background HEX"]', '#00FF00FF');
        assert.equal(await value('input[aria-label="background HEX"]'), '#00FF00FF');
        assert.equal(await value('input[aria-label="Search components"]'), '', 'Tools color editor does not type into app search');
        const green = await capture('property-color');
        const swatch = await pixel(green, 650, 355); assert.ok(swatch[1] > 200 && swatch[0] < 100, 'Property override changes app pixels');
        await click('[role=tab][aria-label="Theme"]'); await click(button('Light'));
        const light = await capture('theme-light-preview');
        assert.deepEqual(await pixel(light, 1100, 150), [255, 255, 255]);
        await click(button('Dark')); await click('button[aria-label^="#"]');
        await fill('input[aria-label="background HEX"]', '#123456FF');
        const themed = await capture('theme-custom');
        assert.deepEqual(await pixel(themed, 1100, 150), [18, 52, 86], 'Theme changes reach the inspected application');
        // The theme picker is the last saturation group in the accessible tree.
        const pads = await page.$$('[role=group][aria-label^="Saturation "]');
        const themeBounds = await pads[pads.length - 1].evaluate(el => el.getBoundingClientRect().toJSON());
        await page.mouse.move(themeBounds.x + themeBounds.width / 2, themeBounds.y + themeBounds.height / 2);
        await page.mouse.down(); await pause(); await page.mouse.up(); await pause();
        assert.equal(await value('input[aria-label="background HEX"]'), '#406080FF', 'DevTools pad receives its own layout bounds');
        await capture('theme-pad');
        await click(button('Reset theme')); const reset = await capture('theme-reset');
        assert.deepEqual(await pixel(reset, 1100, 150), scheme === 'light' ? [255, 255, 255] : [9, 9, 11]);
        await click('[role=tab][aria-label="Profiling"]'); await click(button('Resources')); await capture('resources');
        await pause(1300);
        assert.ok(await page.$$eval('[aria-label]', es => es.some(el => el.getAttribute('aria-label').includes('browser sandbox'))));
        await page.setViewport({ width: 560, height: 900 }); await pause(500);
        await capture('resources-narrow');
        await click('[role=tab][aria-label="Theme"]'); await capture('theme-narrow');
        await click('[role=tab][aria-label="Elements"]'); await capture('properties-narrow');
        await click(button('← Elements')); assert.ok(await page.$(filter));
        await capture('tree-narrow');
        assert.deepEqual(errors, []);
        console.log(`${scheme}: pad pixels/gestures, opacity, hover, filter, dividers, properties, live theme and responsive panels passed`);
        await page.close();
    }
} finally { await browser.close(); }
