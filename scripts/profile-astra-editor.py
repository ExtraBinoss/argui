#!/usr/bin/env python3
"""Profile Astra Editor through the repository's private native X11 display."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import time

from PIL import Image
from Xlib import X, XK, display
from Xlib.ext import xtest


WINDOW_WIDTH = 1360
WINDOW_HEIGHT = 900
TREE_POINT = (115, 80)
EDITOR_POINT = (760, 430)
TREE_REGION = (45, 64, 295, 158)
SIDEBAR_REGION = (0, 0, 350, WINDOW_HEIGHT)
EDITOR_REGION = (300, 70, WINDOW_WIDTH, WINDOW_HEIGHT)


def cpu_seconds(process):
    """Return user and system CPU seconds consumed by `process`."""
    fields = (process / "stat").read_text().rpartition(") ")[2].split()
    return (int(fields[11]) + int(fields[12])) / os.sysconf("SC_CLK_TCK")


def memory(process):
    """Return proportional and private resident memory for `process` in MiB."""
    fields = {}
    for line in (process / "smaps_rollup").read_text().splitlines()[1:]:
        key, value, *_ = line.split()
        fields[key.rstrip(":")] = int(value) / 1024
    return {
        "rss_mib": round(fields["Rss"], 2),
        "pss_mib": round(fields["Pss"], 2),
        "private_mib": round(
            sum(
                fields.get(key, 0)
                for key in ("Private_Clean", "Private_Dirty", "Private_Hugetlb")
            ),
            2,
        ),
    }


def gpu_memory(process):
    """Return resident DRM memory for `process` when the kernel exposes it."""
    clients = {}
    for entry in (process / "fdinfo").iterdir():
        try:
            fields = dict(
                line.split(":", 1)
                for line in entry.read_text().splitlines()
                if ":" in line
            )
        except FileNotFoundError:
            continue
        if "drm-client-id" in fields:
            clients[(fields.get("drm-pdev"), fields["drm-client-id"])] = fields
    resident = [
        int(value.split()[0])
        for fields in clients.values()
        for key, value in fields.items()
        if key.startswith("drm-resident-")
    ]
    return {"gpu_resident_mib": round(sum(resident) / 1024, 2)} if resident else {}


class NativeSession:
    """Drive and capture one Astra Editor process on the private X server."""

    def __init__(self, binary, output, app_cpu):
        """Create a session for executable `binary`, artifacts `output`, and `app_cpu`."""
        self.binary = binary
        self.output = output
        self.app_cpu = app_cpu
        self.connection = display.Display()
        self.root = self.connection.screen().root
        self.child = None
        self.window = None

    def visible_windows(self):
        """Return mapped top-level windows that expose window-manager protocols."""
        return [
            window
            for window in self.root.query_tree().children
            if window.get_attributes().map_state == X.IsViewable
            and window.get_wm_protocols()
        ]

    def origin(self):
        """Return the root-relative origin of the application window."""
        x = 0
        y = 0
        window = self.window
        while window.id != self.root.id:
            geometry = window.get_geometry()
            x += geometry.x
            y += geometry.y
            window = window.query_tree().parent
        return x, y

    def frame(self):
        """Capture the current application window as an RGB image."""
        left, top = self.origin()
        geometry = self.window.get_geometry()
        shot = self.root.get_image(
            left,
            top,
            geometry.width,
            geometry.height,
            X.ZPixmap,
            0xFFFFFFFF,
        )
        return Image.frombytes(
            "RGB",
            (geometry.width, geometry.height),
            shot.data,
            "raw",
            "BGRX",
        )

    def capture(self, name):
        """Save a non-blank application frame named `name` and return it."""
        image = self.frame()
        assert max(high - low for low, high in image.getextrema()) > 100, (
            f"blank native capture: {name}"
        )
        image.save(self.output / f"{name}.png")
        return image

    def motion(self, x, y):
        """Move the private pointer to window-relative coordinates `x`, `y`."""
        left, top = self.origin()
        xtest.fake_input(self.connection, X.MotionNotify, x=left + x, y=top + y)
        self.connection.sync()

    def button(self, button=1):
        """Press and release pointer `button` without inserting an artificial delay."""
        xtest.fake_input(self.connection, X.ButtonPress, button)
        xtest.fake_input(self.connection, X.ButtonRelease, button)

    def key(self, name, pressed=True):
        """Queue one key transition for X keysym `name`."""
        code = self.connection.keysym_to_keycode(XK.string_to_keysym(name))
        assert code, f"unknown X keysym: {name}"
        xtest.fake_input(
            self.connection,
            X.KeyPress if pressed else X.KeyRelease,
            code,
        )

    def tap(self, name):
        """Queue one press and release for X keysym `name`."""
        self.key(name, True)
        self.key(name, False)

    def shortcut(self, *names):
        """Queue a chord containing `names`, then flush it to the X server."""
        for name in names:
            self.key(name, True)
        for name in reversed(names):
            self.key(name, False)
        self.connection.sync()

    def region_digest(self, region):
        """Return a compact digest of `region` in the current application frame."""
        pixels = self.frame().crop(region).tobytes()
        return hashlib.blake2b(pixels, digest_size=12).digest()

    def visual_action(self, region, action, timeout=0.8, stable=0.12):
        """Measure CPU and presented-frame changes caused by `action` in `region`."""
        process = Path(f"/proc/{self.child.pid}")
        previous = self.region_digest(region)
        unique = {previous}
        before_cpu = cpu_seconds(process)
        started = time.monotonic()
        action()
        self.connection.sync()
        first_change = None
        last_change = None
        deadline = started + timeout
        while time.monotonic() < deadline:
            digest = self.region_digest(region)
            now = time.monotonic()
            if digest != previous:
                unique.add(digest)
                previous = digest
                first_change = first_change or now
                last_change = now
            if last_change is not None and now - last_change >= stable:
                break
            time.sleep(0.004)
        finished = time.monotonic()
        return {
            "presented": first_change is not None,
            "first_present_ms": (
                round((first_change - started) * 1000, 2) if first_change else None
            ),
            "last_change_ms": (
                round((last_change - started) * 1000, 2) if last_change else None
            ),
            "sample_wall_ms": round((finished - started) * 1000, 2),
            "distinct_frames": len(unique) - 1,
            "app_cpu_ms": round((cpu_seconds(process) - before_cpu) * 1000, 2),
        }

    def start(self):
        """Launch Astra Editor and return mapping and first-frame latency metrics."""
        log = (self.output / "runtime.log").open("w")
        self.log = log
        launched = time.monotonic()
        command = [self.binary]
        if self.app_cpu is not None:
            command = ["taskset", "-c", str(self.app_cpu), self.binary]
        self.child = subprocess.Popen(command, stdout=log, stderr=log)
        deadline = launched + 20
        while time.monotonic() < deadline:
            self.window = next(
                (
                    window
                    for window in self.visible_windows()
                    if (window.get_wm_name() or "").startswith("Astra Editor")
                ),
                None,
            )
            if self.window is not None:
                break
            time.sleep(0.01)
        assert self.window is not None, "Astra Editor native window did not open"
        mapped = time.monotonic()
        self.window.configure(width=WINDOW_WIDTH, height=WINDOW_HEIGHT)
        self.window.set_input_focus(X.RevertToParent, X.CurrentTime)
        self.connection.sync()
        deadline = launched + 20
        while time.monotonic() < deadline:
            image = self.frame()
            if max(high - low for low, high in image.getextrema()) > 100:
                ready = time.monotonic()
                break
            time.sleep(0.01)
        else:
            raise AssertionError("Astra Editor never presented a non-blank frame")
        return {
            "window_mapped_ms": round((mapped - launched) * 1000, 2),
            "first_frame_ms": round((ready - launched) * 1000, 2),
        }

    def stop(self):
        """Terminate the application and close the private display connection."""
        if self.child is not None:
            self.child.terminate()
            try:
                self.child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.child.kill()
                self.child.wait(timeout=5)
        self.connection.close()
        if hasattr(self, "log"):
            self.log.close()


def check_private_display():
    """Reject execution outside the repository-owned hidden X11 server."""
    assert os.environ.get("ARGUI_HIDDEN_DISPLAY") == "1"
    assert not os.environ.get("WAYLAND_DISPLAY")
    server_pid = int(os.environ["ARGUI_HIDDEN_X11_PID"])
    server_env = Path(f"/proc/{server_pid}/environ").read_bytes().split(b"\0")
    assert f"XDG_RUNTIME_DIR={os.environ['XDG_RUNTIME_DIR']}".encode() in server_env
    assert b"WAYLAND_DISPLAY=argui-test" in server_env


def idle_sample(session, seconds):
    """Measure CPU and memory while `session` receives no input for `seconds`."""
    process = Path(f"/proc/{session.child.pid}")
    before = cpu_seconds(process)
    started = time.monotonic()
    time.sleep(seconds)
    elapsed = time.monotonic() - started
    return {
        "seconds": round(elapsed, 2),
        "cpu_percent_one_core": round(
            100 * (cpu_seconds(process) - before) / elapsed,
            2,
        ),
        **memory(process),
        **gpu_memory(process),
    }


def profile(session, args):
    """Run the complete native scenario and return its measurements."""
    result = {
        "environment": {
            "window": f"{WINDOW_WIDTH}x{WINDOW_HEIGHT}",
            "display": os.environ.get("DISPLAY"),
            "sampler_cpu_affinity": sorted(os.sched_getaffinity(0)),
            "app_cpu": session.app_cpu,
        },
        "launch": session.start(),
    }
    time.sleep(args.warmup_seconds)
    session.capture("initial")
    result["idle"] = idle_sample(session, args.idle_seconds)

    session.motion(*TREE_POINT)
    time.sleep(0.1)
    result["folder_close"] = session.visual_action(
        TREE_REGION,
        session.button,
        timeout=args.animation_timeout,
    )
    session.capture("folder-closed")
    session.motion(*TREE_POINT)
    time.sleep(0.1)
    result["folder_open"] = session.visual_action(
        TREE_REGION,
        session.button,
        timeout=args.animation_timeout,
    )
    settled_tree = session.region_digest(TREE_REGION)
    time.sleep(0.25)
    result["folder_open"]["final_matches_steady_state"] = (
        session.region_digest(TREE_REGION) == settled_tree
    )
    session.capture("folder-opened")

    result["explorer_hide"] = session.visual_action(
        SIDEBAR_REGION,
        lambda: session.shortcut("Control_L", "b"),
    )
    result["explorer_show"] = session.visual_action(
        SIDEBAR_REGION,
        lambda: session.shortcut("Control_L", "b"),
    )
    session.capture("explorer-restored")

    session.motion(*EDITOR_POINT)
    session.button()
    session.connection.sync()
    time.sleep(0.12)
    session.shortcut("Control_L", "End")
    time.sleep(0.08)

    def prepare_large_document():
        for _ in range(args.prepare_lines):
            session.tap("a")
            session.tap("Return")
        session.shortcut("Control_L", "b")

    result["document_preparation"] = session.visual_action(
        SIDEBAR_REGION,
        prepare_large_document,
        timeout=args.queue_timeout,
    )
    result["document_preparation"]["inserted_lines"] = args.prepare_lines
    session.visual_action(
        SIDEBAR_REGION,
        lambda: session.shortcut("Control_L", "b"),
    )
    session.shortcut("Control_L", "Home")
    time.sleep(0.6)

    session.motion(*EDITOR_POINT)
    session.button()
    session.connection.sync()
    time.sleep(0.08)
    result["single_character"] = session.visual_action(
        EDITOR_REGION,
        lambda: session.tap("z"),
        timeout=0.3,
        stable=0.08,
    )

    def typing_burst():
        for _ in range(args.typing_events):
            session.tap("a")
        session.shortcut("Control_L", "b")

    result["typing_burst"] = session.visual_action(
        SIDEBAR_REGION,
        typing_burst,
        timeout=args.queue_timeout,
    )
    result["typing_burst"]["characters"] = args.typing_events
    first_present = result["typing_burst"]["first_present_ms"]
    result["typing_burst"]["characters_per_second"] = (
        round(args.typing_events * 1000 / first_present, 1) if first_present else None
    )
    session.visual_action(
        SIDEBAR_REGION,
        lambda: session.shortcut("Control_L", "b"),
    )
    time.sleep(0.3)

    session.motion(*EDITOR_POINT)
    session.button()
    session.connection.sync()
    session.shortcut("Control_L", "Home")
    time.sleep(0.12)

    def scroll_down():
        for _ in range(args.scroll_events):
            session.button(5)

    result["scroll"] = session.visual_action(
        EDITOR_REGION,
        scroll_down,
        timeout=0.3,
        stable=0.08,
    )
    result["scroll"]["wheel_events"] = args.scroll_events
    session.capture("after-scroll")
    process = Path(f"/proc/{session.child.pid}")
    result["after_stress"] = {**memory(process), **gpu_memory(process)}
    return result


def budget_results(result):
    """Return conservative interactive-performance checks for `result`."""
    checks = {
        "first frame <= 1500 ms": result["launch"]["first_frame_ms"] <= 1500,
        "idle CPU <= 1% of one core": result["idle"]["cpu_percent_one_core"] <= 1,
        "folder animation responds <= 100 ms": (
            result["folder_open"]["first_present_ms"] is not None
            and result["folder_open"]["first_present_ms"] <= 100
        ),
        "folder animation has intermediate frames": (
            result["folder_open"]["distinct_frames"] >= 2
        ),
        "folder animation settles <= 400 ms": (
            result["folder_open"]["last_change_ms"] is not None
            and result["folder_open"]["last_change_ms"] <= 400
        ),
        "folder animation reaches its steady visual": result["folder_open"][
            "final_matches_steady_state"
        ],
        "Ctrl+B responds <= 100 ms": (
            result["explorer_hide"]["first_present_ms"] is not None
            and result["explorer_hide"]["first_present_ms"] <= 100
        ),
        "Ctrl+B restore responds <= 100 ms": (
            result["explorer_show"]["first_present_ms"] is not None
            and result["explorer_show"]["first_present_ms"] <= 100
        ),
        "single character presents <= 100 ms": (
            result["single_character"]["first_present_ms"] is not None
            and result["single_character"]["first_present_ms"] <= 100
        ),
        "typing burst drains <= 1000 ms": (
            result["typing_burst"]["first_present_ms"] is not None
            and result["typing_burst"]["first_present_ms"] <= 1000
        ),
        "large document scroll presents <= 100 ms": (
            result["scroll"]["first_present_ms"] is not None
            and result["scroll"]["first_present_ms"] <= 100
        ),
    }
    return {"passed": all(checks.values()), "checks": checks}


def main():
    """Parse options, execute the profile, save JSON, and optionally enforce budgets."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--output", default="target/astra-editor-profile")
    parser.add_argument("--warmup-seconds", type=float, default=3.0)
    parser.add_argument("--idle-seconds", type=float, default=3.0)
    parser.add_argument("--prepare-lines", type=int, default=300)
    parser.add_argument("--typing-events", type=int, default=200)
    parser.add_argument("--scroll-events", type=int, default=24)
    parser.add_argument("--animation-timeout", type=float, default=0.8)
    parser.add_argument("--queue-timeout", type=float, default=3.0)
    parser.add_argument("--sampler-cpu", type=int)
    parser.add_argument("--app-cpu", type=int)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    if (
        args.warmup_seconds <= 0
        or args.idle_seconds <= 0
        or args.animation_timeout <= 0
        or args.queue_timeout <= 0
    ):
        parser.error("durations must be positive")
    if args.prepare_lines <= 0 or args.typing_events <= 0 or args.scroll_events <= 0:
        parser.error("event counts must be positive")
    binary = Path(args.binary).resolve()
    if not binary.is_file():
        parser.error(f"binary does not exist: {binary}")
    check_private_display()
    allowed_cpus = os.sched_getaffinity(0)
    for name, cpu in (("sampler", args.sampler_cpu), ("app", args.app_cpu)):
        if cpu is not None and cpu not in allowed_cpus:
            parser.error(f"{name} CPU {cpu} is outside the allowed affinity")
    if args.sampler_cpu is not None:
        os.sched_setaffinity(0, {args.sampler_cpu})
    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)
    session = NativeSession(str(binary), output, args.app_cpu)
    try:
        result = profile(session, args)
        result["budgets"] = budget_results(result)
        payload = json.dumps(result, indent=2)
        (output / "profile.json").write_text(payload + "\n")
        print(payload, flush=True)
        if args.check and not result["budgets"]["passed"]:
            raise SystemExit(1)
    finally:
        session.stop()


if __name__ == "__main__":
    main()
