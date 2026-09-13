"""Exercise the real native gallery and optionally sample its DevTools overhead."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import time
from Xlib import X, XK, display
from Xlib.ext import xtest
from PIL import Image

assert os.environ.get("ARGUI_HIDDEN_DISPLAY") == "1"
assert not os.environ.get("WAYLAND_DISPLAY")
server = Path(f"/proc/{int(os.environ['ARGUI_HIDDEN_X11_PID'])}/environ").read_bytes().split(b"\0")
assert f"XDG_RUNTIME_DIR={os.environ['XDG_RUNTIME_DIR']}".encode() in server
assert b"WAYLAND_DISPLAY=argui-test" in server
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--binary", default="target/release/argui-widget-gallery")
parser.add_argument("--output", default="target/devtools-native")
parser.add_argument("--seconds", type=int, default=0)
parser.add_argument("--baseline", action="store_true")
parser.add_argument("--sensors", action="store_true")
parser.add_argument("--measure-only", action="store_true")
args = parser.parse_args()
folder = Path(args.output)
folder.mkdir(parents=True, exist_ok=True)
spec = importlib.util.spec_from_file_location("profile", "scripts/profile-widget-gallery.py")
profile = importlib.util.module_from_spec(spec)
spec.loader.exec_module(profile)
connection = display.Display()
root = connection.screen().root
samples = []

def windows():
    return [window for window in root.query_tree().children
            if window.get_attributes().map_state == X.IsViewable and window.get_wm_protocols()]

def origin(window):
    location = window.get_geometry()
    return location.x, location.y

def motion(window, x, y):
    left, top = origin(window)
    xtest.fake_input(connection, X.MotionNotify, x=left + x, y=top + y)
    connection.sync()
    time.sleep(.06)

def click(window, x, y, button=1):
    motion(window, x, y)
    xtest.fake_input(connection, X.ButtonPress, button)
    xtest.fake_input(connection, X.ButtonRelease, button)
    connection.sync()
    time.sleep(.25)

def key(name, pressed=True):
    xtest.fake_input(connection, X.KeyPress if pressed else X.KeyRelease,
                     connection.keysym_to_keycode(XK.string_to_keysym(name)))
    connection.sync()

def capture(label):
    size = root.get_geometry()
    shot = root.get_image(0, 0, size.width, size.height, X.ZPixmap, 0xFFFFFFFF)
    image = Image.frombytes("RGB", (size.width, size.height), shot.data, "raw", "BGRX")
    assert max(high-low for low, high in image.getextrema()) > 100, "Blank native capture"
    image.save(folder / f"{label}.png")
    return image

def measure(label):
    time.sleep(2)
    image = capture(label)
    if label in {"elements", "profiling", "resources"}:
        point = {"elements": (50, 422), "profiling": (125, 422), "resources": (1140, 474)}[label]
        red, green, blue = image.getpixel(point)
        assert blue > red + 70 and blue > green + 25, f"Requested pane was not activated: {label}"
    if args.seconds <= 0:
        return
    process = Path(f"/proc/{child.pid}")
    started = time.monotonic()
    before = profile.cpu_seconds(process)
    memory = []
    while time.monotonic() - started < args.seconds:
        assert child.poll() is None
        memory.append(profile.memory(process))
        time.sleep(.5)
    result = {"phase": label,
              "cpu_percent": (profile.cpu_seconds(process)-before)/(time.monotonic()-started)*100,
              **{key: sum(row[key] for row in memory)/len(memory) for key in memory[0]},
              **profile.gpu_memory(process)}
    # Reject a stopped presentation instead of interpreting it as a CPU saving.
    after_image = capture(f"{label}-end")
    assert image.crop((866, 253, 902, 285)).tobytes() != after_image.crop((866, 253, 902, 285)).tobytes(), "The gallery spinner stopped presenting during measurement"
    samples.append(result)
    (folder / "samples.json").write_text(json.dumps(samples, indent=2))
    print(json.dumps(result), flush=True)

with (folder / "runtime.log").open("w") as log:
    child = subprocess.Popen([args.binary], stdout=log, stderr=log)
    try:
        deadline = time.monotonic() + 20
        main = None
        while time.monotonic() < deadline:
            main = next((window for window in windows() if window.get_wm_name() == "Argui Widget Gallery"), None)
            if main: break
            time.sleep(.1)
        assert main is not None, "Native gallery did not open"
        main.set_input_focus(X.RevertToParent, X.CurrentTime)
        connection.sync()
        main.configure(width=1220, height=1000)
        connection.sync()
        time.sleep(3)
        measure("closed")
        click(main, 1165, 30)
        # Keep both panes large enough to inspect; this is the same operation in both builds.
        time.sleep(.5)
        motion(main, 600, 677)
        xtest.fake_input(connection, X.ButtonPress, 1)
        for y in range(677, 400, -10):
            motion(main, 600, y)
        motion(main, 600, 400)
        xtest.fake_input(connection, X.ButtonRelease, 1)
        connection.sync()
        motion(main, 450, 429)
        measure("elements")
        click(main, 152, 429)
        measure("profiling")
        if not args.baseline:
            click(main, 1170, 480)
            measure("resources")
            if not args.measure_only:
                # Scroll the resource pane to its hardware controls and allocator counters.
                for _ in range(15): click(main, 1090, 880, button=5)
                capture("resources-details")
                if args.sensors:
                    click(main, 880, 940)
                    time.sleep(3)
                    for _ in range(8): click(main, 1090, 880, button=5)
                    capture("hardware-sensors")
                click(main, 80, 429)
                click(main, 1115, 429)
                capture("dock-menu")
                click(main, 1080, 553)
                time.sleep(2)
                tools = next((window for window in windows() if window.id != main.id), None)
                assert tools is not None, "Detached tools window did not open"
                tools.configure(x=600, y=450)
                tools.set_input_focus(X.RevertToParent, X.CurrentTime)
                connection.sync()
                time.sleep(1)
                capture("detached")
                click(tools, 180, 130)
                for char in "primary":
                    key(char); key(char, False)
                time.sleep(.5)
                capture("detached-filter")
        (folder / "windows.json").write_text(json.dumps([
            {"name": window.get_wm_name(), "width":window.get_geometry().width, "height":window.get_geometry().height}
            for window in windows()], indent=2))
    finally:
        child.terminate()
        try: child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            child.kill(); child.wait(timeout=5)
        connection.close()
