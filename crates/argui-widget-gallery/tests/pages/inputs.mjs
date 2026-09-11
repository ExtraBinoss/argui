import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Use scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH, headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const output = process.env.SCREENSHOT_DIR ?? 'target/editor-interactions';
await mkdir(output, { recursive: true });
const pause = (ms = 150) => new Promise(resolve => setTimeout(resolve, ms));
const chord = async (page, key, ...modifiers) => {
    for (const modifier of modifiers) await page.keyboard.down(modifier);
    await page.keyboard.press(key);
    for (const modifier of modifiers.toReversed()) await page.keyboard.up(modifier);
};
try {
    const reference = await browser.newPage();
    await reference.setContent('<input id="field"><textarea id="area"></textarea>');
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    await page.setViewport({ width: 1220, height: 900 });
    await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: process.env.COLOR_SCHEME ?? 'dark' }]);
    await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8793/widgets/', { waitUntil: 'networkidle0' });
    const navigate = async name => {
        await page.waitForSelector(`button[aria-label="${name}"]`);
        await page.$eval(`button[aria-label="${name}"]`, element => element.click());
        await pause(350);
    };
    const rect = selector => page.$eval(selector, element => element.getBoundingClientRect().toJSON());
    const click = async (selector, button = 'left') => {
        const r = await rect(selector);
        assert.ok(r.width > 0 && r.bottom <= 900 && r.right <= 1220, selector);
        await page.mouse.click(r.x + r.width / 2, r.y + r.height / 2, { button });
        await pause();
    };
    const capture = async name => {
        const png = await page.screenshot({ path: `${output}/${name}.png` });
        assert.ok(png.length > 15000, 'A blank frame is not a visual check');
    };
    let keyboardCases = 0;
    for (const [name, selector, referenceSelector] of [
        ['Input', 'input[aria-label="Full name"]', '#field'],
        ['Text area', 'textarea[role="textbox"]', '#area'],
    ]) {
        await navigate(name === 'Input' ? 'Input & Search' : name);
        await page.waitForSelector(selector);
        const reset = async value => {
            await click(selector);
            await chord(page, 'a', 'Control');
            await page.keyboard.type(value);
            await chord(page, 'Home', 'Control');
            await pause();
            assert.equal(await page.$eval(selector, el => el.value), value);
        };
        const base = name === 'Input' ? 'one,  two! three' : 'abcdef\nx\nabcdef';
        const cases = [
            [['ArrowRight', 'Control', 'Shift']],
            [['ArrowRight', 'Control', 'Shift'], ['ArrowRight', 'Control', 'Shift']],
            [['End', 'Control'], ['ArrowLeft', 'Control', 'Shift']],
            [['End', 'Control'], ['ArrowLeft', 'Control', 'Shift'], ['ArrowLeft']],
            [['ArrowRight', 'Control', 'Shift'], ['ArrowRight']],
            [['Delete', 'Control']],
            [['End', 'Control'], ['Backspace', 'Control']],
            [['End', 'Control'], ['Home', 'Control', 'Shift']],
            [['End', 'Shift']],
        ];
        if (name === 'Text area') cases.push(
            [...Array.from({ length: 5 }, () => ['ArrowRight']), ['ArrowDown'], ['ArrowDown']],
            [['End', 'Control'], ['ArrowUp'], ['ArrowUp']],
            [['End'], ['ArrowRight']],
            [['End'], ['ArrowRight'], ['ArrowLeft']],
        );
        for (const sequence of cases) {
            await reference.$eval(referenceSelector, (element, value) => {
                element.value = value; element.focus(); element.setSelectionRange(0, 0);
            }, base);
            for (const keys of sequence) await chord(reference, ...keys);
            await reference.keyboard.type('X');
            const expected = await reference.$eval(referenceSelector, el => el.value);
            await reset(base);
            for (const keys of sequence) await chord(page, ...keys);
            await page.keyboard.type('X');
            await pause();
            assert.equal(await page.$eval(selector, el => el.value), expected, `${name}: ${JSON.stringify(sequence)}`);
            keyboardCases++;
        }
        const words = name === 'Input' ? 'alpha beta gamma' : 'alpha beta gamma\nsecond line\nthird line';
        await reset(words);
        const r = await rect(selector);
        const x = r.x + 14 + 58; // Middle of beta, Noto Sans 15 px.
        const y = name === 'Input' ? r.y + r.height / 2 : r.y + 24;
        await pause(650);
        for (let i = 1; i <= 2; i++) await page.mouse.click(x, y, { clickCount: i });
        await capture(`${name.toLowerCase()}-word-selected`);
        await page.keyboard.type('X');
        await pause();
        assert.equal(await page.$eval(selector, el => el.value), words.replace('beta', 'X'));
        await reset(words);
        await pause(650);
        for (let i = 1; i <= 3; i++) await page.mouse.click(x, y, { clickCount: i });
        await capture(`${name.toLowerCase()}-line-selected`);
        await page.keyboard.type('X');
        await pause();
        assert.equal(await page.$eval(selector, el => el.value), name === 'Input' ? 'X' : 'Xsecond line\nthird line');
        await reset(words);
        await pause(650);
        await page.mouse.click(x, y);
        await page.mouse.move(x, y);
        await page.mouse.down({ clickCount: 2 });
        await page.mouse.move(x + 44, y, { steps: 8 });
        await page.mouse.up();
        await capture(`${name.toLowerCase()}-word-drag`);
        await page.keyboard.type('X');
        await pause();
        assert.equal(await page.$eval(selector, el => el.value), words.replace('beta gamma', 'X'));
        console.log(`${name}: keyboard, double/triple click and word drag passed`);
    }
    const item = label => `[role="menuitem"][aria-label="${label}"]`;
    for (const name of ['Menu', 'Context menu', 'Menubar']) {
        await navigate(name === 'Input' ? 'Input & Search' : name);
        const trigger = name === 'Menubar' ? item('View') : '[aria-label="Note actions"][aria-haspopup="menu"]';
        await click(trigger, name === 'Context menu' ? 'right' : 'left');
        await click(item('Layout'));
        await page.waitForSelector('[role="menuitemradio"][aria-label="Compact"]');
        await capture(`${name.toLowerCase()}-nested`);
        await page.mouse.click(1100, 820);
        await pause(400);
        assert.equal(await page.$$eval('[role="menu"]', elements => elements.length), 0, `${name} outside dismissal`);
        await click(trigger, name === 'Context menu' ? 'right' : 'left');
        await click(item('Layout'));
        await page.keyboard.press('ArrowRight');
        await page.keyboard.press('Escape');
        await pause(350);
        assert.equal(await page.$$eval('[role="menu"]', elements => elements.length), 1, `${name} escape closes submenu`);
        await page.keyboard.press('Escape');
        await pause(350);
        assert.equal(await page.$$eval('[role="menu"]', elements => elements.length), 0, `${name} escape closes root`);
        console.log(`${name}: outside and Escape dismissal passed`);
    }
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ keyboardCases, screenshots: output, errors }, null, 2));
} finally {
    await browser.close();
}
