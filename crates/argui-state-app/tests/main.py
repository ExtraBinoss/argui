"""Run and close the profiled app on the private X11 server."""

import os
import subprocess
import sys
import threading
import time
from pathlib import Path

from Xlib import X, display, protocol


assert os.environ.get("ARGUI_HIDDEN_DISPLAY") == "1"
assert os.environ.get("WAYLAND_DISPLAY") is None
assert os.environ.get("DISPLAY")
runtime_dir = os.environ["XDG_RUNTIME_DIR"]
assert Path(runtime_dir).name.startswith("argui-display.")
server_pid = int(os.environ["ARGUI_HIDDEN_X11_PID"])
server_env = Path(f"/proc/{server_pid}/environ").read_bytes().split(b"\0")
assert f"XDG_RUNTIME_DIR={runtime_dir}".encode() in server_env
assert b"WAYLAND_DISPLAY=argui-test" in server_env
assert len(sys.argv) == 2, "expected the argui-state binary path"

connection = display.Display()
root = connection.screen().root
pid_atom = connection.intern_atom("_NET_WM_PID")
delete_atom = connection.intern_atom("WM_DELETE_WINDOW")
app = subprocess.Popen(
    [sys.argv[1]],
    env={**os.environ, "ARGUI_PROFILE": "1"},
    stdout=subprocess.DEVNULL,
    stderr=subprocess.PIPE,
    text=True,
    bufsize=1,
)
events = threading.Condition()
lines = []
counts = [0, 0]
window = None


def read_profiles():
    for line in app.stderr:
        with events:
            lines.append(line.rstrip())
            counts[0] += line.startswith("[argui][ui]")
            counts[1] += line.startswith("[argui][gpu]")
            events.notify_all()


def descendants(parent):
    for child in parent.query_tree().children:
        yield child
        yield from descendants(child)


def find_app_window():
    for candidate in descendants(root):
        if candidate.get_wm_name() != "Argui state showcase":
            continue
        if candidate.get_attributes().map_state != X.IsViewable:
            continue
        pid = candidate.get_full_property(pid_atom, X.AnyPropertyType)
        if pid is None or int(pid.value[0]) != app.pid:
            continue
        if delete_atom in (candidate.get_wm_protocols() or []):
            return candidate
    return None


def wait_for_profiles(target, seconds):
    deadline = time.monotonic() + seconds
    with events:
        while counts[0] < target[0] or counts[1] < target[1]:
            if app.poll() is not None:
                raise AssertionError(f"app exited before profiles arrived: {lines!r}")
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                raise AssertionError(f"expected profiles {target}, received {counts}")
            events.wait(min(remaining, 0.1))


def wait_for_idle():
    # Let resize activity settle before testing shutdown from an idle window.
    deadline = time.monotonic() + 5
    with events:
        while time.monotonic() < deadline:
            previous = counts.copy()
            events.wait_for(lambda: counts != previous or app.poll() is not None, timeout=0.25)
            if app.poll() is not None:
                raise AssertionError(f"app exited before becoming idle: {lines!r}")
            if counts == previous:
                return
    raise AssertionError(f"profiled app did not become idle: {counts}")


reader = threading.Thread(target=read_profiles, daemon=True)
reader.start()
try:
    deadline = time.monotonic() + 25
    while time.monotonic() < deadline:
        if app.poll() is not None:
            raise AssertionError(f"app exited before creating its window: {lines!r}")
        window = find_app_window()
        if window is not None:
            break
        time.sleep(0.05)
    assert window is not None, "profiled app window did not appear"

    wait_for_profiles([1, 1], 20)
    original = window.get_geometry()
    with events:
        next_frame = [counts[0] + 1, counts[1] + 1]
    window.configure(width=original.width + 32, height=original.height + 24)
    connection.sync()
    resized = window.get_geometry()
    assert resized.width > original.width and resized.height > original.height
    wait_for_profiles(next_frame, 20)
    wait_for_idle()

    message = protocol.event.ClientMessage(
        window=window,
        client_type=connection.intern_atom("WM_PROTOCOLS"),
        data=(32, [delete_atom, X.CurrentTime, 0, 0, 0]),
    )
    window.send_event(message, event_mask=X.NoEventMask)
    connection.flush()
    try:
        status = app.wait(timeout=10)
    except subprocess.TimeoutExpired as error:
        app.kill()
        app.wait()
        raise AssertionError(f"app did not exit after WM_DELETE_WINDOW; profiles: {lines!r}") from error
    reader.join(timeout=2)
    assert status == 0, f"app did not exit normally: {status}; {lines!r}"
    assert counts[0] >= 2 and counts[1] >= 2, f"too few profiled frames: {counts}"
    print(f"profiled app closed normally after {counts[0]} UI and {counts[1]} GPU frames")
finally:
    if app.poll() is None:
        try:
            if window is not None:
                message = protocol.event.ClientMessage(
                    window=window,
                    client_type=connection.intern_atom("WM_PROTOCOLS"),
                    data=(32, [delete_atom, X.CurrentTime, 0, 0, 0]),
                )
                window.send_event(message, event_mask=X.NoEventMask)
                connection.flush()
            app.wait(timeout=3)
        except Exception:
            app.kill()
            app.wait()
    reader.join(timeout=2)
    connection.close()
