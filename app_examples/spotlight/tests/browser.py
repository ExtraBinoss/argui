#!/usr/bin/env python3
"""Exercise Spotlight's WebAssembly build in a private visible Chromium."""

import io
import os
from pathlib import Path
import time

from PIL import Image
from playwright.sync_api import sync_playwright


def contrasting(png: bytes) -> bool:
    """Return whether a PNG contains enough contrast to be a real rendered frame."""
    image = Image.open(io.BytesIO(png)).convert("RGB")
    return max(high - low for low, high in image.getextrema()) > 80


def main() -> None:
    """Open, filter, resize, and capture the browser Spotlight example."""
    assert os.environ.get("ARGUI_HIDDEN_DISPLAY") == "1"
    assert "DISPLAY" not in os.environ
    output = Path(os.environ.get("SCREENSHOT_DIR", "target/spotlight-browser"))
    output.mkdir(parents=True, exist_ok=True)
    url = os.environ.get("SPOTLIGHT_URL", "http://127.0.0.1:8796/")
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
            context = browser.new_context(viewport={"width": 960, "height": 720})
            page = context.new_page()
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
            initial = page.screenshot(path=output / "initial.png")
            assert contrasting(initial), "blank initial Spotlight capture"

            search = page.get_by_role("searchbox")
            search.wait_for(state="visible")
            search_bounds = search.bounding_box()
            assert search_bounds
            page.mouse.click(
                search_bounds["x"] + search_bounds["width"] / 2,
                search_bounds["y"] + search_bounds["height"] / 2,
            )
            page.keyboard.press("Control+=")
            page.wait_for_function(
                "({ height }) => document.querySelector('[role=searchbox]')"
                "?.getBoundingClientRect().height > height * 1.05",
                arg={"height": search_bounds["height"]},
            )
            zoomed_bounds = search.bounding_box()
            assert zoomed_bounds and zoomed_bounds["height"] > search_bounds["height"]
            assert search.input_value() == "", "UI zoom shortcut leaked into the text field"
            page.keyboard.press("Control+0")
            page.wait_for_function(
                "({ height }) => Math.abs(document.querySelector('[role=searchbox]')"
                "?.getBoundingClientRect().height - height) < 0.5",
                arg={"height": search_bounds["height"]},
            )
            page.keyboard.press("ArrowDown")
            page.keyboard.press("Enter")
            time.sleep(0.32)
            page.screenshot(path=output / "keyboard.png")
            page.get_by_role(
                "status", name="Activated · Browse recent files"
            ).wait_for(
                state="visible", timeout=5_000
            )
            page.keyboard.type("gpu", delay=55)
            page.wait_for_function(
                "!document.querySelector('[aria-label=\"Open command palette\"]')"
            )
            gpu_result = page.get_by_role("button", name="Inspect GPU frame")
            gpu_result.wait_for(state="visible")
            result_bounds = gpu_result.bounding_box()
            assert result_bounds
            page.mouse.click(
                result_bounds["x"] + result_bounds["width"] / 2,
                result_bounds["y"] + result_bounds["height"] / 2,
            )
            page.get_by_role("status", name="Activated · Inspect GPU frame").wait_for(
                state="visible"
            )
            time.sleep(0.32)
            filtered = page.screenshot(path=output / "filtered.png")
            assert contrasting(filtered), "blank filtered Spotlight capture"
            assert filtered != initial, "typing did not change the Spotlight frame"

            bounds = canvas.bounding_box()
            assert bounds and bounds["width"] <= 704 and bounds["height"] <= 452
            assert errors == [], errors
            context.close()

            mobile = browser.new_context(
                viewport={"width": 390, "height": 844},
                has_touch=True,
                is_mobile=True,
            )
            mobile_page = mobile.new_page()
            mobile_page.on("pageerror", lambda error: errors.append(str(error)))
            mobile_page.goto(url, wait_until="networkidle", timeout=90_000)
            mobile_page.wait_for_function(
                "document.querySelector('#status')?.hidden === true", timeout=90_000
            )
            mobile_search = mobile_page.get_by_role("searchbox")
            mobile_search.wait_for(state="visible")
            search_bounds = mobile_search.bounding_box()
            assert search_bounds
            mobile_page.touchscreen.tap(
                search_bounds["x"] + search_bounds["width"] / 2,
                search_bounds["y"] + search_bounds["height"] / 2,
            )
            mobile_page.wait_for_function(
                "document.activeElement?.getAttribute('aria-label') === "
                "'Search apps and commands'"
            )
            mobile_page.keyboard.insert_text("gpu")
            mobile_page.locator('[aria-label="Inspect GPU frame"]').wait_for(
                state="visible"
            )
            assert errors == [], errors
            mobile.close()
        finally:
            browser.close()


if __name__ == "__main__":
    main()
