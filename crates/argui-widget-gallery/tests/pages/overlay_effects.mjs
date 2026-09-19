import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Use scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH, headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const output = process.env.SCREENSHOT_DIR ?? 'target/overlay-interactions';
await mkdir(output, { recursive: true });
const pause = (ms = 200) => new Promise(resolve => setTimeout(resolve, ms));
try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    await page.setViewport({ width: 1220, height: 900 });
    await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8793/widgets/', { waitUntil: 'networkidle0' });
    const button = label => `button[aria-label="${label}"]`;
    const rect = selector => page.$eval(selector, element => element.getBoundingClientRect().toJSON());
    const point = async selector => {
        const r = await rect(selector);
        assert.ok(r.width > 0 && r.bottom <= 900 && r.right <= 1220, selector);
        return [r.x + r.width / 2, r.y + r.height / 2];
    };
    const click = async selector => { await page.mouse.click(...await point(selector)); await pause(); };
    const navigate = async label => {
        await page.waitForSelector(button(label));
        await page.$eval(button(label), element => element.click());
        await pause(400);
    };
    const capture = async name => {
        const png = await page.screenshot({ path: `${output}/${name}.png` });
        assert.ok(png.length > 15000, 'Blank capture');
    };
    const expanded = selector => page.$eval(selector, element => element.getAttribute('aria-expanded'));
    for (const scheme of ['light', 'dark']) {
        await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: scheme }]);
        const tips = () => page.$$eval('[role="tooltip"]', elements => elements.length);
        await navigate('Button');
        await page.mouse.move(...await point(button('Primary'))); await pause(550);
        assert.equal(await tips(), 1, 'Buttons show their tooltip by default');
        await capture(`${scheme}-button-tooltip`);
        await click(button('Primary'));
        assert.equal(await tips(), 0, 'Activation dismisses help');
        await page.mouse.move(1100, 820); await pause(200);
        await page.mouse.move(...await point(button('Primary'))); await pause(550);
        assert.equal(await tips(), 1, 'Hover reopens help while the clicked button keeps focus');
        await page.mouse.move(1100, 820); await pause(200);
        const item = label => `[role="menuitem"][aria-label="${label}"]`;
        for (const name of ['Menu', 'Context menu', 'Menubar']) {
            await navigate(name);
            const trigger = name === 'Menubar' ? item('View') : '[aria-label="Note actions"][aria-haspopup="menu"]';
            await page.mouse.click(...await point(trigger), { button: name === 'Context menu' ? 'right' : 'left' });
            await pause();
            await click(item('Layout'));
            await page.waitForSelector('[role="menuitemradio"][aria-label="Compact"]');
            assert.equal(await tips(), 0, 'Opening a menu suppresses tooltip help');
            await capture(`${scheme}-${name.toLowerCase()}-nested`);
            await page.mouse.click(1100, 820); await pause();
            assert.equal(await page.$$eval('[role="menu"]', elements => elements.length), 0);
        }
        await navigate('Select');
        await click(button('Choose a backend'));
        await page.waitForSelector('[role="listbox"]');
        await capture(`${scheme}-select`);
        await page.keyboard.press('Escape'); await pause(350);
        await navigate('Date picker');
        await click(button('Choose a date'));
        await capture(`${scheme}-date-picker`);
        const yearHeading = await page.$$eval('button', elements => {
            const heading = elements.find(element => {
                const rect = element.getBoundingClientRect();
                return rect.width > 0 && /\b-?\d{4}\b/.test(element.getAttribute('aria-label') ?? '');
            });
            if (!heading) throw new Error('Date picker month/year heading is missing');
            return `#${heading.id}`;
        });
        await click(yearHeading);
        assert.equal(await page.$$eval('[role="cell"]', elements => elements.length), 12);
        await capture(`${scheme}-date-picker-years`);
        await click('[role="cell"]');
        assert.equal(await page.$$eval('[role="cell"]', elements => elements.length), 42);
        await page.keyboard.press('Escape'); await pause();
        await navigate('Toast');
        await click(button('Show notification'));
        await page.waitForSelector(button('Close'));
        await capture(`${scheme}-toast`);
        await click(button('Close'));
        await navigate('Dialog');
        await click(button('Open dialog'));
        await page.waitForSelector(button('Close dialog'));
        await capture(`${scheme}-dialog`);
        await click(button('Close dialog'));
        await navigate('Popover');
        const labels = await page.$$eval('button', elements => elements.filter(el => el.getBoundingClientRect().x === 16).map(el => el.getAttribute('aria-label')));
        const effects = labels.indexOf('Liquid glass');
        const examples = labels.indexOf('Actions');
        for (const group of [labels.slice(0, effects), labels.slice(effects, examples), labels.slice(examples)]) {
            assert.deepEqual(group, group.toSorted((a, b) => a.localeCompare(b, 'en', { sensitivity: 'base' })));
        }
        await capture(`${scheme}-popover-closed`);
        for (const label of ['Rename project', 'Sharing settings', 'Choose accent']) {
            const trigger = button(label);
            await page.mouse.move(...await point(trigger)); await pause(550);
            assert.equal(await tips(), 1, `${label}: button tooltip before opening`);
            await click(trigger);
            assert.equal(await expanded(trigger), 'true');
            assert.equal(await tips(), 0, `${label}: opening hides tooltip`);
            await capture(`${scheme}-popover-${label}`);
            if (label === 'Rename project') {
                await click('input[aria-label="Project name"]');
                await page.keyboard.down('Control'); await page.keyboard.press('a'); await page.keyboard.up('Control');
                await page.keyboard.type('Shared notes');
                await click(button('Save name'));
                assert.equal(await expanded(trigger), 'false');
                await click(trigger);
            } else if (label === 'Sharing settings') {
                const panel = '[role="group"][aria-label="Sharing settings"]';
                const clickInside = async selector => {
                    const r = await rect(selector);
                    for (const [x, y] of [[2, 2], [35, 22], [35, 54], [r.width - 3, r.height - 3]]) {
                        await page.mouse.click(r.x + x, r.y + y); await pause();
                        assert.equal(await expanded(trigger), 'true', 'Panel text and padding keep the parent open');
                        assert.ok(await page.$(selector), 'The clicked panel stays open');
                    }
                };
                await clickInside(panel);
                await click('[role="switch"][aria-label="Anyone with the link"]');
                await capture(`${scheme}-popover-switch-toggled`);
                const nested = button('Link options');
                const child = '[role="group"][aria-label="Link options"]';
                await click(nested);
                assert.equal(await expanded(nested), 'true');
                await clickInside(child);
                const downloads = '[role="switch"][aria-label="Allow downloads"]';
                const before = await page.$eval(downloads, element => element.getAttribute('aria-checked'));
                await click(downloads);
                assert.notEqual(await page.$eval(downloads, element => element.getAttribute('aria-checked')), before);
                assert.equal(await expanded(trigger), 'true');
                assert.equal(await tips(), 0, 'Nested popovers suppress automatic help');
                await capture(`${scheme}-popover-nested`);
                await page.keyboard.press('Escape'); await pause();
                assert.equal(await expanded(nested), 'false', 'Escape closes the inner panel first');
                assert.equal(await expanded(trigger), 'true');
                await click(nested);
                const r = await rect(panel);
                await page.mouse.click(r.x + 35, r.y + 22); await pause();
                assert.equal(await expanded(nested), 'false', 'Clicking the parent closes only the child');
                assert.equal(await expanded(trigger), 'true');
                await click(nested);
                await page.mouse.click(1100, 820); await pause();
                assert.equal(await expanded(nested), 'false', 'Outside click closes the top panel');
                assert.equal(await expanded(trigger), 'true');
            } else {
                await click(button('Rose'));
            }
            await page.keyboard.press('Escape'); await pause();
            assert.equal(await expanded(trigger), 'false');
            await click(trigger);
            await page.mouse.click(1100, 820); await pause();
            assert.equal(await expanded(trigger), 'false');
        }
        const nativeToggle = '[role="switch"][aria-label="Allow outside this window"]';
        const assertInViewport = async selector => {
            const bounds = await rect(selector);
            assert.ok(bounds.x >= 0 && bounds.y >= 0 && bounds.right <= 1220 && bounds.bottom <= 900,
                'PreferNative falls back inside the browser canvas');
        };
        await click(nativeToggle);
        assert.equal(await page.$eval(nativeToggle, el => el.getAttribute('aria-checked')), 'true');
        await click(button('Sharing settings'));
        await click(button('Link options'));
        await assertInViewport('[role="group"][aria-label="Sharing settings"]');
        await assertInViewport('[role="group"][aria-label="Link options"]');
        await capture(`${scheme}-popover-native-fallback`);
        await page.keyboard.press('Escape'); await pause();
        await page.keyboard.press('Escape'); await pause();
        await click(nativeToggle);
        await navigate('Tooltip');
        const tooltipLabels = ['Save draft', 'Preview page', 'View history'];
        for (const label of tooltipLabels) {
            const trigger = button(label);
            await page.mouse.move(...await point(trigger));
            await pause(550);
            assert.equal(await tips(), 1, `${label}: hover opens`);
            const id = await page.$eval('[role="tooltip"]', element => element.id);
            assert.equal(await page.$eval(trigger, element => element.getAttribute('aria-describedby')), id);
            await capture(`${scheme}-tooltip-${label}`);
            await page.mouse.move(...await point('[role="tooltip"]'));
            await pause(200);
            assert.equal(await tips(), 1, `${label}: content stays hoverable`);
            await page.mouse.move(1100, 820); await pause(250);
            assert.equal(await tips(), 0, `${label}: leave closes`);
            await click(trigger);
            assert.equal(await tips(), 0, `${label}: click dismisses`);
            await page.mouse.move(1100, 820); await pause(200);
            await page.mouse.move(...await point(trigger)); await pause(550);
            assert.equal(await tips(), 1, `${label}: hover after click reopens`);
            await page.mouse.move(1100, 820); await pause(250);
        }
        for (const label of tooltipLabels) await page.mouse.move(...await point(button(label)));
        await page.mouse.move(1100, 820); await pause(550);
        assert.equal(await tips(), 0, 'Fast passes leave no tooltip behind');
        await page.mouse.move(...await point(button('Save draft'))); await pause(550);
        await page.keyboard.press('Escape'); await pause();
        assert.equal(await tips(), 0, 'Escape closes hover help while focus is elsewhere');
        await page.mouse.move(1100, 820); await pause();
        await click(button('Save draft'));
        await page.keyboard.press('Tab'); await pause();
        assert.equal(await tips(), 1, 'Tab shows keyboard help');
        const focusId = await page.$eval(button('Preview page'), element => element.id);
        assert.equal(await page.$eval('canvas', element => element.getAttribute('aria-activedescendant')), focusId);
        await capture(`${scheme}-tooltip-keyboard`);
        await page.keyboard.press('Escape'); await pause();
        assert.equal(await tips(), 0);
        await click(nativeToggle);
        await page.mouse.move(...await point(button('Preview page'))); await pause(550);
        assert.equal(await tips(), 1);
        await assertInViewport('[role="tooltip"]');
        await capture(`${scheme}-tooltip-native-fallback`);
        await page.mouse.move(1100, 820); await pause(250);
        await click(nativeToggle);
        console.log(`${scheme}: menus/submenus, select, date picker, toast, dialog, three effect surfaces and tooltip interactions passed`);
    }
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ screenshots: output, errors }));
} finally { await browser.close(); }
