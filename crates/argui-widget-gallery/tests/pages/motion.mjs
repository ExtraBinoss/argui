import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";

assert.equal(
  process.env.ARGUI_HIDDEN_DISPLAY,
  "1",
  "Use scripts/linux-hidden-display.sh",
);
assert.equal(process.env.DISPLAY, undefined);
const imported = await import(process.env.PUPPETEER_MODULE ?? "puppeteer");
const browser = await (imported.puppeteer ?? imported.default).launch({
  executablePath: process.env.CHROME_PATH,
  headless: false,
  args: [
    "--ozone-platform=wayland",
    "--enable-unsafe-webgpu",
    "--ignore-gpu-blocklist",
    "--enable-features=Vulkan",
    "--use-angle=vulkan",
  ],
});
const output = process.env.SCREENSHOT_DIR ?? "target/motion-interactions";
await mkdir(output, { recursive: true });
const pause = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const button = (label) => `button[aria-label="${label}"]`;

try {
  const page = await browser.newPage();
  const errors = [];
  page.on("pageerror", (error) => errors.push(String(error)));
  await page.setViewport({ width: 1060, height: 620 });
  await page.goto(process.env.GALLERY_URL ?? "http://127.0.0.1:8793/widgets/", {
    waitUntil: "networkidle0",
  });
  await page.waitForSelector(button("Animation lab"));
  await page.$eval(button("Animation lab"), (element) => element.click());
  await page.waitForSelector(button("Stop animations"));
  await pause(350);

  const toggle = await page.$eval(button("Stop animations"), (element) =>
    element.getBoundingClientRect().toJSON(),
  );
  assert.ok(
    toggle.top >= 0 && toggle.bottom <= 620,
    "The animation toggle is immediately visible",
  );
  await page.mouse.click(
    toggle.x + toggle.width / 2,
    toggle.y + toggle.height / 2,
  );
  await page.waitForSelector(button("Run animations"));
  await page.$eval(button("Run animations"), (element) => element.click());
  await page.waitForSelector(button("Stop animations"));

  const scrollAndSample = async (x, y, selector) => {
    const position = () =>
      page.$eval(selector, (element) => element.getBoundingClientRect().y);
    await page.mouse.move(x, y);
    await page.mouse.wheel({ deltaY: 32 });
    await pause(38);
    const first = await position();
    await pause(85);
    const second = await position();
    await pause(120);
    const third = await position();
    assert.ok(
      first > second && second >= third,
      `${selector} keeps moving after one wheel event`,
    );
    return [first, second, third];
  };

  const contentSamples = await scrollAndSample(
    650,
    430,
    button("Stop animations"),
  );
  await page.$eval(button("Stop animations"), (element) => element.click());
  await page.waitForSelector(button("Run animations"));
  const sidebarSamples = await scrollAndSample(130, 420, button("Button"));

  const png = await page.screenshot({
    path: `${output}/animation-and-scroll.png`,
  });
  assert.ok(png.length > 15_000, "Blank animation capture");
  assert.deepEqual(errors, []);
  console.log(
    JSON.stringify(
      { contentSamples, sidebarSamples, screenshot: output },
      null,
      2,
    ),
  );
} finally {
  await browser.close();
}
