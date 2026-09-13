import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Use scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH, headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const output = process.env.SCREENSHOT_DIR ?? 'target/color-picker-interactions';
await mkdir(output, { recursive: true });
const pause = (ms = 400) => new Promise(resolve => setTimeout(resolve, ms));
try {
    for (const scheme of ['dark', 'light']) {
        const page = await browser.newPage();
        const errors = [];
        page.on('pageerror', error => errors.push(String(error)));
        await page.setViewport({ width: 1220, height: 1000 });
        await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: scheme }]);
        await page.evaluateOnNewDocument(() => {
            const now = performance.now.bind(performance);
            let frozen;
            Object.defineProperty(performance, 'now', { value: () => frozen ?? now() });
            window.freezePickerTestClock = () => { frozen = now(); };
            window.resumePickerTestClock = () => { frozen = undefined; };
        });
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
        await page.$eval(button('Color picker'), el => el.click()); await pause();
        const hex = 'input[aria-label="Accent color HEX"]';
        await fill(hex, '#FF0000FF');
        const pad = await rect('[role=group][aria-label^="Saturation "]');
        const preview = await rect(button('Live button preview'));
        await page.evaluate(() => window.freezePickerTestClock());
        await page.mouse.move(pad.x + pad.width*.5, pad.y+pad.height*.5);
        await page.mouse.down(); await pause();
        for (const [s, v, expected] of [[.5, .5, [128,64,64]], [.75,.75,[191,48,48]], [.25,.5,[128,96,96]]]) {
            await page.mouse.move(pad.x + pad.width*s, pad.y+pad.height*(1-v), { steps: 6 }); await pause(50);
            const png = await capture(`gallery-${s}-${v}`);
            for (const [x,y] of [[preview.x+12,preview.y+12], [650,355]]) {
                const actual = await pixel(png,x,y);
                assert.ok(actual.every((c,i) => Math.abs(c-expected[i]) <= 2), `Color reaches both previews without advancing the animation clock: ${actual} vs ${expected}`);
            }
            assert.deepEqual(await rect('[role=group][aria-label^="Saturation "]'), pad);
        }
        await page.mouse.up();
        await page.evaluate(() => window.resumePickerTestClock());
        await click(button('DevTools'));
        const splitter = await rect('[role=separator][aria-orientation=horizontal]');
        await page.mouse.move(600,splitter.y+3); await page.mouse.down();
        await page.mouse.move(600,400,{steps:10}); await page.mouse.up(); await pause();
        await fill('input[aria-label="Filter elements"]','color-preview-swatch');
        await click('[role=treeitem][aria-label*="#color-preview-swatch"]');
        await click(button('Layout'));
        await click('button[aria-label^="#"]');
        await fill('input[aria-label="background HEX"]', '#00FF00FF');
        const toolsPad = await (await page.$$('[role=group][aria-label^="Saturation "]')).at(-1).evaluate(el => el.getBoundingClientRect().toJSON());
        await page.evaluate(() => window.freezePickerTestClock());
        await page.mouse.move(toolsPad.x+toolsPad.width*.5, toolsPad.y+toolsPad.height*.5);
        await page.mouse.down(); await pause();
        for (const [s,v,expected] of [[.5,.5,[64,128,64]],[.75,.75,[48,191,48]],[.25,.5,[96,128,96]]]) {
            await page.mouse.move(toolsPad.x+toolsPad.width*s,toolsPad.y+toolsPad.height*(1-v),{steps:6}); await pause(50);
            const png=await capture(`devtools-${s}-${v}`);
            const actual=await pixel(png,650,355);
            assert.ok(actual.every((c,i)=>Math.abs(c-expected[i])<=2),`Property override updates without advancing the animation clock: ${actual} vs ${expected}`);
            const bounds=await (await page.$$('[role=group][aria-label^="Saturation "]')).at(-1).evaluate(el=>el.getBoundingClientRect().toJSON());
            assert.deepEqual(bounds,toolsPad,'Popover stays fixed throughout a drag');
        }
        await page.mouse.up(); await page.evaluate(() => window.resumePickerTestClock());
        await page.keyboard.press('Escape'); await pause();
        assert.equal(await page.$('input[aria-label="background HEX"]'),null);
        assert.deepEqual(errors,[]);
        console.log(`${scheme}: gallery and DevTools colors update without advancing animation time, with a stationary pad/popover`);
        await page.close();
    }
} finally { await browser.close(); }
