"""Inject input only into the rootful Xwayland server owned by this private test."""
import os
import time
from pathlib import Path
from Xlib import X, XK, display, protocol
from Xlib.ext import xtest
from PIL import Image

assert os.environ.get("ARGUI_HIDDEN_DISPLAY") == "1"
assert not os.environ.get("WAYLAND_DISPLAY")
server_pid = int(os.environ["ARGUI_HIDDEN_X11_PID"])
server_env = Path(f"/proc/{server_pid}/environ").read_bytes().split(b"\0")
assert f"XDG_RUNTIME_DIR={os.environ['XDG_RUNTIME_DIR']}".encode() in server_env
assert b"WAYLAND_DISPLAY=argui-test" in server_env
connection = display.Display()
root = connection.screen().root
main = None

def descendants(window):
    for child in window.query_tree().children:
        yield child
        yield from descendants(child)

def wait_for_windows():
    deadline = time.monotonic() + 20
    while time.monotonic() < deadline:
        windows = list(descendants(root))
        main_candidates = [window for window in windows if window.get_wm_name() == "Argui native popup test"]
        main = next((window for window in main_candidates if window.get_wm_protocols()), None)
        popups = [window for window in windows if window.get_wm_name() == "Argui popup" and window.get_attributes().map_state == X.IsViewable]
        if main and len(popups) == 2:
            print("native windows", [(w.id, w.get_wm_name(), w.get_wm_class()) for w in main_candidates], [(w.id, w.get_wm_transient_for().id) for w in popups], flush=True)
            popup_ids = {window.id for window in popups}
            parent = next(window for window in popups if window.get_wm_transient_for().id not in popup_ids)
            main = parent.get_wm_transient_for()
            assert main.get_wm_name() == "Argui native popup test"
            nested = next(window for window in popups if window.get_wm_transient_for().id == parent.id)
            return main, parent, nested
        time.sleep(0.05)
    raise AssertionError("two native popup surfaces did not appear")

def origin(window):
    x = y = 0
    while window.id != root.id:
        geometry = window.get_geometry()
        x += geometry.x
        y += geometry.y
        window = window.query_tree().parent
    return x, y

def click(window, x, y, button=1):
    left, top = origin(window)
    xtest.fake_input(connection, X.MotionNotify, x=left + x, y=top + y)
    connection.sync()
    time.sleep(0.06)
    xtest.fake_input(connection, X.ButtonPress, button)
    xtest.fake_input(connection, X.ButtonRelease, button)
    connection.sync()
    time.sleep(0.16)

def key(name, pressed=True):
    code = connection.keysym_to_keycode(XK.string_to_keysym(name))
    xtest.fake_input(connection, X.KeyPress if pressed else X.KeyRelease, code)
    connection.sync()

def wait_for_focus(window):
    deadline = time.monotonic() + 3
    while time.monotonic() < deadline:
        focused = connection.get_input_focus().focus
        if getattr(focused, 'id', focused) == window.id:
            return
        time.sleep(0.05)
    raise AssertionError(f'Expected focus on {window.id}, got {focused}')

def type_text(value):
    for char in value:
        name = "space" if char == " " else char
        key(name)
        key(name, False)
    time.sleep(0.25)

def capture(name):
    size = root.get_geometry()
    shot = root.get_image(0, 0, size.width, size.height, X.ZPixmap, 0xFFFFFFFF)
    folder = Path("target/native-popups")
    folder.mkdir(parents=True, exist_ok=True)
    Image.frombytes("RGB", (size.width, size.height), shot.data, "raw", "BGRX").save(folder / name)

try:
    main, parent, nested = wait_for_windows()
    # This bare test server has no WM to activate the application on launch.
    main.set_input_focus(X.RevertToParent, X.CurrentTime)
    connection.sync()
    wait_for_focus(main)
    ready = Path(os.environ['XDG_RUNTIME_DIR']) / 'native-ready'
    deadline = time.monotonic() + 3
    while not ready.exists() and time.monotonic() < deadline:
        time.sleep(0.05)
    assert ready.exists(), 'Runtime processed the initial application activation'
    time.sleep(0.4)
    mx, my = origin(main)
    px, py = origin(parent)
    nx, ny = origin(nested)
    assert px + parent.get_geometry().width > mx + main.get_geometry().width
    assert nx + nested.get_geometry().width > px + parent.get_geometry().width
    capture("outside-window.png")
    click(parent, 70, 18)
    wait_for_focus(parent)
    key("End")
    key("End", False)
    type_text(" ergonomics")
    key("Control_L")
    key("Shift_L")
    key("Left")
    key("Left", False)
    key("Shift_L", False)
    key("Control_L", False)
    time.sleep(0.2)
    capture("native-selection.png")
    click(nested, 60, 55)
    for _ in range(4):
        click(parent, 110, 180, button=5)
    capture("native-scroll.png")
    # Simulate another application inside this private X server only.
    external = root.create_window(900, 800, 80, 60, 0, X.CopyFromParent)
    external.map()
    external.set_input_focus(X.RevertToParent, X.CurrentTime)
    connection.sync()
    time.sleep(0.4)
    assert connection.get_input_focus().focus.id == external.id, 'Dismissing the group must not steal focus back'
    # Controlled popups remain mounted in this fixture. Explicit user input must
    # reactivate one after suspension, including clicks outside focusable fields.
    click(parent, 110, 180)
    wait_for_focus(parent)
    external.set_input_focus(X.RevertToParent, X.CurrentTime)
    connection.sync()
    time.sleep(0.4)
    assert connection.get_input_focus().focus.id == external.id, 'Reactivation must not disable dismissal'
finally:
    if main:
        message = protocol.event.ClientMessage(window=main, client_type=connection.intern_atom("WM_PROTOCOLS"), data=(32, [connection.intern_atom("WM_DELETE_WINDOW"), X.CurrentTime, 0, 0, 0]))
        main.send_event(message, event_mask=X.NoEventMask)
        connection.flush()
    else:
        # A failed startup must not leave the child runtime waiting forever.
        os.kill(os.getppid(), 15)
    connection.sync()
    connection.close()
