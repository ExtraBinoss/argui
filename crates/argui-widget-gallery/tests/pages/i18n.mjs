import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';

assert.equal(process.env.ARGUI_HIDDEN_DISPLAY, '1', 'Use scripts/linux-hidden-display.sh');
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? 'puppeteer');
const browser = await (imported.puppeteer ?? imported.default).launch({
    executablePath: process.env.CHROME_PATH,
    headless: false,
    args: ['--ozone-platform=wayland', '--enable-unsafe-webgpu', '--ignore-gpu-blocklist', '--enable-features=Vulkan', '--use-angle=vulkan'],
});
const output = process.env.SCREENSHOT_DIR ?? 'target/i18n-interactions';
await mkdir(output, { recursive: true });
const pause = (ms = 300) => new Promise(resolve => setTimeout(resolve, ms));
try {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    page.on('console', message => { if (message.type() === 'error') errors.push(message.text()); });
    await page.setViewport({ width: 1220, height: 780 });
    await page.emulateMediaFeatures([{ name: 'prefers-color-scheme', value: 'light' }]);
    await page.goto(process.env.GALLERY_URL ?? 'http://127.0.0.1:8793/widgets/', { waitUntil: 'networkidle0' });
    const button = label => `button[aria-label="${label}"]`;
    const click = async label => {
        await page.waitForSelector(button(label));
        await page.$eval(button(label), element => element.click());
        await pause();
    };
    const capture = async name => {
        const png = await page.screenshot({ path: `${output}/${name}.png` });
        assert.ok(png.length > 15000, `${name} capture is blank`);
        return Buffer.from(png).toString('base64');
    };
    const x = label => page.$eval(button(label), element => element.getBoundingClientRect().x);

    await click('Internationalization');
    await page.waitForSelector(button('Save changes'));
    const englishSaveX = await x('Save changes');
    const englishLessX = await x('Fewer');
    const english = await capture('english');

    await click('Français');
    await page.waitForSelector(button('Enregistrer'));
    await click('Plus');
    const french = await capture('french-plural');

    await click('العربية');
    await page.waitForSelector(button('حفظ التغييرات'));
    const arabicSaveX = await x('حفظ التغييرات');
    const arabicLessX = await x('أقل');
    const arabic = await capture('arabic-rtl');

    assert.ok(englishSaveX > englishLessX, 'LTR keeps the save action last');
    assert.ok(arabicSaveX < arabicLessX, 'RTL reverses the localized control row');
    assert.notEqual(english, french);
    assert.notEqual(french, arabic);
    assert.deepEqual(errors, []);
    console.log(JSON.stringify({ screenshots: output, englishSaveX, arabicSaveX, errors }, null, 2));
} finally {
    await browser.close();
}
