#!/usr/bin/env python3
"""Exercise Spotlight's hide-and-wake flow on the private X11 display."""

import argparse
import os
from pathlib import Path
import subprocess
import tempfile
import time

from PIL import Image
from Xlib import X, XK, display, error
from Xlib.ext import xtest


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("binary", type=Path)
args = parser.parse_args()
assert os.environ.get("ARGUI_HIDDEN_DISPLAY") == "1"
assert os.environ.get("ARGUI_HIDDEN_X11_PID")
assert os.environ.get("DISPLAY")
assert args.binary.is_file()

connection = display.Display()
root = connection.screen().root
data_home = tempfile.TemporaryDirectory(prefix="argui-spotlight-data-")
applications = Path(data_home.name) / "applications"
applications.mkdir()
desktop_entry = applications / "dev.argui.spotlight.desktop"
desktop_entry.write_text(
    "[Desktop Entry]\n"
    "Type=Application\n"
    "Name=Argui Spotlight\n"
    "Exec=argui-example-spotlight\n"
    "NoDisplay=true\n",
    encoding="utf-8",
)
environment = os.environ.copy()
environment["XDG_DATA_HOME"] = data_home.name
application = subprocess.Popen([args.binary.resolve()], env=environment)


def wait_for(description, predicate, timeout=10.0):
    """Wait until `predicate` returns a truthy result or raise an assertion."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        connection.sync()
        try:
            result = predicate()
        except error.BadWindow:
            result = None
        if result:
            return result
        time.sleep(0.05)
    raise AssertionError(f"Timed out waiting for {description}")


def spotlight_window():
    """Return the top-level Spotlight X11 window when it exists."""
    for window in root.query_tree().children:
        try:
            if window.get_wm_name() == "Argui Spotlight":
                return window
        except error.BadWindow:
            continue
    return None


def is_viewable(window):
    """Return whether `window` is currently mapped and visible."""
    return window.get_attributes().map_state == X.IsViewable


def fake_key(name, pressed):
    """Send one synthetic key transition by X11 keysym name."""
    keycode = connection.keysym_to_keycode(XK.string_to_keysym(name))
    assert keycode, f"No X11 keycode for {name}"
    xtest.fake_input(connection, X.KeyPress if pressed else X.KeyRelease, keycode)


def click_primary():
    """Send one primary-button click at the current pointer position."""
    xtest.fake_input(connection, X.ButtonPress, 1)
    xtest.fake_input(connection, X.ButtonRelease, 1)
    connection.sync()


def window_pixel(window, x, y):
    """Read one RGB pixel from the mapped X11 window."""
    geometry = window.get_geometry()
    image = window.get_image(0, 0, geometry.width, geometry.height, X.ZPixmap, 0xFFFFFFFF)
    assert image, "Could not capture the Spotlight X11 window"
    frame = Image.frombytes(
        "RGB", (geometry.width, geometry.height), image.data, "raw", "BGRX"
    )
    return frame.getpixel((x, y))


def is_primary_blue(color):
    """Return whether an RGB sample resembles Spotlight's selected-row blue."""
    red, green, blue = color
    return blue > red + 80 and blue > green + 60


try:
    window = wait_for("the Spotlight window", spotlight_window)
    wait_for("the Spotlight window to map", lambda: is_viewable(window))
    wait_for(
        "the stale development desktop entry to refresh",
        lambda: "X-Argui-Development=true" in desktop_entry.read_text(encoding="utf-8")
        and str(args.binary.resolve()) in desktop_entry.read_text(encoding="utf-8"),
    )
    geometry = window.get_geometry()
    origin = window.translate_coords(root, 0, 0)
    xtest.fake_input(
        connection,
        X.MotionNotify,
        x=origin.x + geometry.width - 43,
        y=origin.y + geometry.height - 25,
    )
    # A private compositor can consume the first click while activating the
    # Xwayland surface; the second click exercises Argui's button hit target.
    click_primary()
    time.sleep(0.1)
    click_primary()
    wait_for("Spotlight to hide", lambda: not is_viewable(window))

    fake_key("Control_L", True)
    fake_key("space", True)
    fake_key("space", False)
    fake_key("Control_L", False)
    connection.sync()
    wait_for("the global shortcut to show Spotlight", lambda: is_viewable(window))

    window.set_input_focus(X.RevertToParent, X.CurrentTime)
    connection.sync()
    fake_key("Down", True)
    fake_key("Down", False)
    connection.sync()
    wait_for(
        "ArrowDown to select the second result",
        lambda: is_primary_blue(window_pixel(window, 350, 188)),
    )
finally:
    connection.close()
    application.terminate()
    try:
        application.wait(timeout=5)
    except subprocess.TimeoutExpired:
        application.kill()
        application.wait(timeout=5)
    data_home.cleanup()
