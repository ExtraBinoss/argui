#!/usr/bin/env python3
"""Exercise Astra Editor's WebAssembly build in a private visible Chromium."""

import io
import os
from pathlib import Path
import time

from PIL import Image
from playwright.sync_api import sync_playwright


def contrasting(png: bytes) -> bool:
    """Return whether a PNG contains enough contrast to be a real rendered frame."""
    image = Image.open(io.BytesIO(png)).convert("RGB")
    return max(high - low for low, high in image.getextrema()) > 100


def main() -> None:
    """Open, interact with, resize, and capture the browser editor."""
    assert os.environ.get("ARGUI_HIDDEN_DISPLAY") == "1"
    assert "DISPLAY" not in os.environ
    output = Path(os.environ.get("SCREENSHOT_DIR", "target/astra-editor-browser"))
    output.mkdir(parents=True, exist_ok=True)
    url = os.environ.get(
        "ASTRA_EDITOR_URL", "http://127.0.0.1:8795/examples/astra-editor/"
    )
    errors: list[str] = []

    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(
            executable_path=os.environ["CHROME_PATH"],
            headless=False,
            args=[
                "--ozone-platform=wayland",
                "--enable-unsafe-webgpu",
                "--ignore-gpu-blocklist",
                "--enable-features=Vulkan",
                "--use-angle=vulkan",
            ],
        )
        try:
            page = browser.new_page(viewport={"width": 1280, "height": 800})
            page.on("pageerror", lambda error: errors.append(str(error)))
            page.on(
                "console",
                lambda message: errors.append(message.text)
                if message.type == "error"
                else None,
            )
            page.goto(url, wait_until="networkidle", timeout=90_000)
            page.wait_for_function(
                "document.querySelector('#status')?.hidden === true", timeout=90_000
            )
            canvas = page.locator("canvas")
            canvas.wait_for(state="visible")
            bounds = canvas.bounding_box()
            assert bounds and bounds["width"] == 1280 and bounds["height"] == 800
            initial = page.screenshot(path=output / "desktop.png")
            assert contrasting(initial), "blank desktop WebAssembly capture"

            canvas.focus()
            page.keyboard.press("Control+b")
            time.sleep(0.35)
            explorer_hidden = page.screenshot(path=output / "explorer-hidden.png")
            assert explorer_hidden != initial, "Ctrl+B did not change the browser frame"
            page.keyboard.press("Control+b")
            page.keyboard.press("Control+Shift+f")
            page.get_by_role("dialog", name="Search across files").wait_for(
                state="visible"
            )
            page.keyboard.press("Escape")

            page.set_viewport_size({"width": 430, "height": 860})
            time.sleep(0.4)
            compact = page.screenshot(path=output / "compact.png")
            assert contrasting(compact), "blank compact WebAssembly capture"
            assert page.evaluate("document.documentElement.scrollWidth === innerWidth")
            assert errors == [], errors
        finally:
            browser.close()


if __name__ == "__main__":
    main()
