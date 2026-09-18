#!/usr/bin/env python3
"""Capture a command on linux-hidden-display.sh's private Wayland display."""
import argparse
import os
from pathlib import Path
import subprocess
import time

import gi

gi.require_version("Gio", "2.0")
gi.require_version("Gst", "1.0")
from gi.repository import Gio, GLib, Gst
from PIL import Image


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--output", default="target/wayland-captures")
parser.add_argument("--settle", type=float, default=25)
parser.add_argument("--click", action="append", default=[], metavar="NAME:X:Y")
parser.add_argument(
    "--action",
    action="append",
    default=[],
    metavar="KIND:NAME:VALUE",
    help="ordered click, type, or chord action followed by a named capture",
)
parser.add_argument("command", nargs=argparse.REMAINDER)
args = parser.parse_args()
assert args.command, "Pass the application command after --"
command = args.command[1:] if args.command[0] == "--" else args.command
assert command
assert os.environ.get("ARGUI_HIDDEN_DISPLAY") == "1"
runtime = Path(os.environ["XDG_RUNTIME_DIR"])
assert runtime.name.startswith("argui-display.") and runtime.stat().st_mode & 0o777 == 0o700
assert os.environ.get("WAYLAND_DISPLAY") == "argui-test"
assert not os.environ.get("DISPLAY") and not os.environ.get("WAYLAND_SOCKET")
assert (runtime / "argui-test").is_socket()

# The capture server and policy manager own no hardware monitors, audio or Bluetooth.
# Their socket, configuration and persistent state are confined to this private directory.
for key in ["XDG_STATE_HOME", "XDG_CONFIG_HOME", "XDG_CACHE_HOME"]:
    os.environ[key] = str(runtime / key)
os.environ["PIPEWIRE_RUNTIME_DIR"] = str(runtime)
os.environ["PIPEWIRE_REMOTE"] = "pipewire-0"
os.environ["PIPEWIRE_CORE"] = "pipewire-0"
bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)


def call(destination, path, interface, method, signature=None, values=None):
    parameters = GLib.Variant(signature, values) if signature else None
    return bus.call_sync(destination, path, interface, method, parameters, None,
                         Gio.DBusCallFlags.NONE, 5000, None).unpack()


def spin(seconds):
    until = time.monotonic() + seconds
    while time.monotonic() < until:
        GLib.MainContext.default().iteration(False)
        time.sleep(0.01)


output = Path(args.output)
output.mkdir(parents=True, exist_ok=True)
Gst.init(None)
rd_destination = "org.gnome.Mutter.RemoteDesktop"
sc_destination = "org.gnome.Mutter.ScreenCast"
rd_interface = rd_destination + ".Session"
sc_interface = sc_destination + ".Session"
processes = []
pipeline = None
remote = None
try:
    with (output / "services.log").open("w") as log:
        def launch(command):
            process = subprocess.Popen(command, stdout=log, stderr=log)
            processes.append(process)
            return process

        launch(["pipewire"])
        for _ in range(100):
            if (runtime / "pipewire-0").is_socket():
                break
            spin(0.05)
        assert (runtime / "pipewire-0").is_socket(), "Private PipeWire did not start"
        launch(["wireplumber", "--profile=policy"])
        remote = call(rd_destination, "/org/gnome/Mutter/RemoteDesktop",
                      rd_destination, "CreateSession")[0]
        identity = call(rd_destination, remote, "org.freedesktop.DBus.Properties",
                        "Get", "(ss)", (rd_interface, "SessionId"))[0]
        session = call(sc_destination, "/org/gnome/Mutter/ScreenCast", sc_destination,
                       "CreateSession", "(a{sv})", ({
                           "remote-desktop-session-id": GLib.Variant("s", identity)},))[0]
        stream = call(sc_destination, session, sc_interface, "RecordMonitor", "(sa{sv})",
                      ("", {"cursor-mode": GLib.Variant("u", 1)}))[0]
        nodes = []
        bus.signal_subscribe(sc_destination, sc_destination + ".Stream", "PipeWireStreamAdded",
                             stream, None, Gio.DBusSignalFlags.NONE,
                             lambda *event: nodes.append(event[5].unpack()[0]))
        call(rd_destination, remote, rd_interface, "Start")
        for _ in range(200):
            if nodes:
                break
            spin(0.025)
        assert nodes, "Private screen capture did not publish a stream"
        frames = []
        pipeline = Gst.parse_launch(
            f"pipewiresrc path={nodes[0]} do-timestamp=true ! videoconvert ! "
            "video/x-raw,format=RGB ! appsink name=sink emit-signals=true "
            "max-buffers=1 drop=true sync=false")

        def sample(sink):
            frame = sink.emit("pull-sample")
            caps = frame.get_caps().get_structure(0)
            buffer = frame.get_buffer()
            frames[:] = [(caps.get_value("width"), caps.get_value("height"),
                          buffer.extract_dup(0, buffer.get_size()))]
            return Gst.FlowReturn.OK

        pipeline.get_by_name("sink").connect("new-sample", sample)
        pipeline.set_state(Gst.State.PLAYING)
        application = launch(command)
        spin(args.settle)

        def capture(name):
            assert application.poll() is None, "Application exited before the capture"
            assert frames, "Blank capture: no frames"
            width, height, data = frames[0]
            screenshot = Image.frombytes("RGB", (width, height), data, "raw", "RGB", len(data) // height)
            assert max(high - low for low, high in screenshot.getextrema()) > 100, "Blank capture"
            assert Path(name).name == name, "Capture names must not contain a path"
            screenshot.save(output / f"{name}.png")
            print(f"Captured {name}: {width} × {height}", flush=True)

        capture("initial")

        def click(x, y):
            """Move from the monitor origin and click one logical position."""
            # Relative pointer events work on the private headless Mutter monitor even
            # when its screencast stream does not yet accept absolute motion.
            call(rd_destination, remote, rd_interface, "NotifyPointerMotionRelative",
                 "(dd)", (-10000.0, -10000.0))
            spin(0.15)
            call(rd_destination, remote, rd_interface, "NotifyPointerMotionRelative",
                 "(dd)", (float(x), float(y)))
            spin(0.15)
            call(rd_destination, remote, rd_interface, "NotifyPointerButton", "(ib)", (272, True))
            spin(0.12)
            call(rd_destination, remote, rd_interface, "NotifyPointerButton", "(ib)", (272, False))
            spin(0.3)

        def keysym(value, pressed):
            """Send one Mutter virtual-keyboard keysym transition."""
            call(rd_destination, remote, rd_interface, "NotifyKeyboardKeysym",
                 "(ub)", (value, pressed))

        named_keysyms = {"Control_L": 0xFFE3, "space": 0x20}
        actions = [f"click:{action}" for action in args.click] + args.action
        for action in actions:
            kind, name, value = action.split(":", 2)
            if kind == "click":
                x, y = value.split(":", 1)
                click(x, y)
            elif kind == "type":
                for character in value:
                    keysym(ord(character), True)
                    spin(0.08)
                    keysym(ord(character), False)
                    spin(0.08)
            elif kind == "chord":
                chord = [named_keysyms.get(key, int(key, 0) if key.startswith("0x") else None)
                         for key in value.split("+")]
                assert all(chord), f"Unknown keysym in {value}"
                for key in chord:
                    keysym(key, True)
                    spin(0.08)
                for key in reversed(chord):
                    keysym(key, False)
                    spin(0.08)
            else:
                raise AssertionError(f"Unknown action kind: {kind}")
            spin(2.0)
            capture(name)
finally:
    # Stop streaming before Python releases its callback, then only terminate our own children.
    if pipeline:
        pipeline.set_state(Gst.State.NULL)
    if remote:
        try:
            call(rd_destination, remote, rd_interface, "Stop")
        except GLib.Error:
            pass
    for process in reversed(processes):
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)
