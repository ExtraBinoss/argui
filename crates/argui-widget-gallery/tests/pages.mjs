import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Run with scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined, 'The desktop X11 display must not be inherited');
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const puppeteer = imported.puppeteer ?? imported.default;
const output = process.env.SCREENSHOT_DIR ?? 'target/widget-interactions';
await mkdir(output, { recursive: true });
const browser = await puppeteer.launch({
    executablePath: process.env.CHROME_PATH,
    headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    await page.setViewport({ width: 1220, height: 780 });
    await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: 'light' }]);
    await page.evaluateOnNewDocument(() => {
        const raf = window.requestAnimationFrame.bind(window);
        window.arguiFrameCount = 0;
        window.requestAnimationFrame = callback => raf(time => {
            window.arguiFrameCount++;
            callback(time);
        });
    });
    await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8793/widgets/', { waitUntil: 'networkidle0' });
    const button = label => `button[aria-label="${label}"]`;
    const settle = () => new Promise(resolve => setTimeout(resolve, 350));
    const navigate = async name => {
        await page.waitForSelector(button(name));
        // Sidebar entries can be outside its scroll viewport. Controls below use real input.
        await page.$eval(button(name), element => element.click());
        await settle();
    };
    const pointClick = async selector => {
        const point = await page.$eval(selector, element => {
            const rect = element.getBoundingClientRect();
            assertVisible(rect);
            function assertVisible(rect) {
                if (rect.width <= 0 || rect.height <= 0 || rect.bottom > innerHeight || rect.right > innerWidth) {
                    throw new Error('Control is outside the viewport');
                }
            }
            return { x: rect.x + rect.width / 2, y: rect.y + rect.height / 2 };
        });
        await page.mouse.click(point.x, point.y);
        await settle();
    };
    const focusWithTab = async selector => {
        const id = await page.$eval(selector, element => element.id);
        await page.$eval('canvas', element => element.focus());
        for (let attempt = 0; attempt < 80; attempt++) {
            await page.keyboard.press('Tab');
            await new Promise(resolve => setTimeout(resolve, 30));
            if (await page.$eval('canvas', element => element.getAttribute('aria-activedescendant')) === id) return;
        }
        assert.fail(`Tab did not reach ${selector}`);
    };
    const capture = async name => {
        const png = await page.screenshot({ path: `${output}/${name}.png` });
        const valid = await page.evaluate(async base64 => {
            const image = new Image();
            image.src = `data:image/png;base64,${base64}`;
            await image.decode();
            if (image.width < 700 || image.height < 700) return false;
            const canvas = document.createElement('canvas');
            canvas.width = image.width;
            canvas.height = image.height;
            const context = canvas.getContext('2d');
            context.drawImage(image, 0, 0);
            const { data } = context.getImageData(0, 0, canvas.width, canvas.height);
            const colors = new Set();
            for (let i = 0; i < data.length; i += 64) {
                colors.add((data[i] << 16) | (data[i + 1] << 8) | data[i + 2]);
                if (colors.size > 32) return true;
            }
            return false;
        }, Buffer.from(png).toString('base64'));
        assert.ok(valid, `Blank or undersized capture: ${name}`);
    };
    await page.waitForSelector(button('System'));
    await pointClick(button('System'));
    for (const [width, height] of [[1220, 780], [800, 720]]) {
        await page.setViewport({ width, height });
        for (const [theme, toggle] of [['light', null], ['dark', 'Light']]) {
            if (toggle) await pointClick(button(toggle));
            for (const [name, selector] of [
                ['Label', '[aria-label="Display name"]'],
                ['Breadcrumb', '[role="link"][aria-label="Home"]'],
                ['Pagination', '[aria-label="Page 1"]'],
                ['Skeleton', '[aria-label="Loading article"]'],
                ['Collapsible', '[aria-label="Archived files"]'],
                ['Menubar', '[role="menubar"]'],
                ['Calendar', '[role="grid"]'],
                ['Data table', '[role="grid"]'],
            ]) {
                await navigate(name);
                await page.waitForSelector(selector);
                await capture(`${name.toLowerCase()}-${theme}-${width}`);
            }
        }
        await pointClick(button('Dark'));
        await pointClick(button('System'));
    }
    await page.setViewport({ width: 1220, height: 780 });
    await navigate('Label');
    await pointClick('[aria-label="Display name"]');
    await page.waitForFunction(() => {
        const id = document.querySelector('canvas').getAttribute('aria-activedescendant');
        return document.getElementById(id)?.getAttribute('role') === 'textbox';
    });
    await page.keyboard.type(' Test');
    await page.waitForFunction(() => document.querySelector('input[aria-labelledby][aria-disabled="false"]')?.value.endsWith(' Test'));
    await pointClick('[aria-label="Workspace ID"]');
    const workspace = await page.$eval('[aria-label="Workspace ID"]', label => document.querySelector(`input[aria-labelledby="${label.id}"]`).id);
    await page.waitForFunction(id => document.querySelector('canvas').getAttribute('aria-activedescendant') === id, {}, workspace);
    await page.keyboard.down('Control');
    await page.keyboard.press('a');
    await page.keyboard.up('Control');
    await page.keyboard.type('my-workspace');
    await page.waitForFunction(id => document.getElementById(id)?.value === 'my-workspace', {}, workspace);
    await navigate('Button');
    await navigate('Label');
    assert.equal(await page.$eval('[aria-label="Workspace ID"]', label => document.querySelector(`input[aria-labelledby="${label.id}"]`).value), 'my-workspace');
    await capture('label-focus');

    await navigate('Breadcrumb');
    await pointClick('[role="link"][aria-label="Components"]');
    await page.waitForSelector('[role="group"][aria-labelledby]');
    await navigate('Breadcrumb');
    await focusWithTab('[role="link"][aria-label="Home"]');
    await page.keyboard.press('Enter');
    await page.waitForSelector(button('Primary'));

    await navigate('Pagination');
    const results = '[role="group"][aria-label="Pagination"]';
    await pointClick(`${results} ${button('Next')}`);
    await page.waitForSelector('[aria-label="Page 2"][aria-description="Current page"]');
    await page.keyboard.press('Enter');
    await page.waitForSelector('[aria-label="Page 3"][aria-description="Current page"]');
    await page.keyboard.press('Space');
    await page.waitForSelector('[aria-label="Page 4"][aria-description="Current page"]');
    await pointClick(button('Page 12'));
    assert.equal(await page.$eval(`${results} ${button('Next')}`, element => element.getAttribute('aria-disabled')), 'true');
    await capture('pagination-last');

    await navigate('Collapsible');
    await pointClick(button('Project files'));
    await pointClick(button('Archived files'));
    assert.equal(await page.$eval(button('Archived files'), element => element.getAttribute('aria-expanded')), 'true');
    await capture('collapsible-expanded');
    await pointClick(button('Archived files'));
    assert.equal(await page.$eval(button('Archived files'), element => element.getAttribute('aria-expanded')), 'false');

    await navigate('Menubar');
    const menuItem = label => `[role="menuitem"][aria-label="${label}"]`;
    await pointClick(menuItem('File'));
    await capture('menubar-file');
    await pointClick(menuItem('New note'));
    await page.waitForSelector('[aria-label="Project note 2"]');
    await pointClick(menuItem('View'));
    await capture('menubar-view');
    await pointClick('[role="menuitemcheckbox"][aria-label="Show details"]');
    await page.waitForSelector('[role="menuitemcheckbox"][aria-checked="false"]');
    await pointClick(menuItem('Layout'));
    await pointClick('[role="menuitemradio"][aria-label="Compact"]');
    await page.keyboard.press('Escape');
    await page.keyboard.press('Escape');
    await settle();
    await capture('menubar-notebook-compact');

    await navigate('Calendar');
    const days = await page.$$eval('[role="cell"]', elements => elements.slice(10, 17).map(element => ({ id: element.id, rect: element.getBoundingClientRect().toJSON() })));
    assert.equal(days.length, 7);
    for (const { rect } of [...days, ...days.toReversed()]) {
        await page.mouse.move(rect.x + rect.width / 2, rect.y + rect.height / 2);
    }
    await pointClick(`#${days[3].id}`);
    await page.mouse.move(1100, 700);
    await settle();
    assert.equal(await page.$eval(`#${days[3].id}`, element => element.getAttribute('aria-selected')), 'true');
    await capture('calendar-selected-after-hover');

    await navigate('Data table');
    const cells = await page.$$eval('[role="cell"]', elements => elements.slice(0, 16).map(element => element.getBoundingClientRect().toJSON()));
    assert.ok(cells.length >= 12);
    for (const rect of [...cells, ...cells.toReversed()]) {
        await page.mouse.move(rect.x + 4, rect.y + rect.height / 2);
        await page.mouse.move(rect.x + rect.width / 2, rect.y + rect.height / 2);
    }
    const cell = cells[4];
    await page.mouse.move(cell.x + cell.width / 2, cell.y + cell.height / 2);
    await capture('data-table-row-hover');
    await page.mouse.move(1100, 700);
    await capture('data-table-hover-cleared');

    const frames = async () => {
        await settle();
        const before = await page.evaluate(() => window.arguiFrameCount);
        await new Promise(resolve => setTimeout(resolve, 650));
        return await page.evaluate(() => window.arguiFrameCount) - before;
    };
    await navigate('Skeleton');
    const animated = await frames();
    assert.ok(animated > 0, 'Mounted skeletons animate');
    await pointClick(button('Show content'));
    await page.waitForSelector('[aria-label="Article preview"]');
    await capture('skeleton-loaded');
    assert.equal(await frames(), 0, 'Loaded content does not animate');
    await pointClick(button('Show loading'));
    await navigate('Breadcrumb');
    assert.equal(await frames(), 0, 'Unmounted skeletons do not animate');
    await page.emulateMediaFeatures([{ name: 'prefers-reduced-motion', value: 'reduce' }]);
    await page.reload({ waitUntil: 'networkidle0' });
    await navigate('Skeleton');
    assert.equal(await frames(), 0, 'Reduced motion does not animate');
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ screenshots: output, animatedFrames: animated, loadedFrames: 0, unmountedFrames: 0, reducedMotionFrames: 0, errors }, null, 2));
} finally {
    await browser.close();
}
