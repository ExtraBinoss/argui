#!/usr/bin/env python3
"""Sample the native release workload on Linux, including its GPU renderer."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import time


def cpu_seconds(process):
    # The command name can contain spaces and parentheses.
    fields = (process / "stat").read_text().rpartition(") ")[2].split()
    return (int(fields[11]) + int(fields[12])) / os.sysconf("SC_CLK_TCK")


def memory(process):
    fields = {}
    for line in (process / "smaps_rollup").read_text().splitlines()[1:]:
        key, value, *_ = line.split()
        fields[key.rstrip(":")] = int(value) / 1024
    return {
        "rss_mib": fields["Rss"],
        "pss_mib": fields["Pss"],
        "private_mib": sum(
            fields.get(key, 0)
            for key in ("Private_Clean", "Private_Dirty", "Private_Hugetlb")
        ),
    }


def gpu_memory(process):
    clients = {}
    for entry in (process / "fdinfo").iterdir():
        try:
            fields = dict(
                line.split(":", 1) for line in entry.read_text().splitlines() if ":" in line
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary")
    parser.add_argument("--app", action="store_true", help="Measure the real gallery's initial page")
    parser.add_argument("--page", choices=("table", "vlist", "data-table"), default="table")
    parser.add_argument("--scroll", action="store_true")
    parser.add_argument("--seconds", type=int, default=20)
    parser.add_argument("--warmup", type=float, default=4)
    args = parser.parse_args()
    if args.seconds <= 0:
        parser.error("--seconds must be positive")
    if not 0 < args.warmup < float("inf"):
        parser.error("--warmup must be finite and positive")
    if args.app and args.scroll:
        parser.error("--app measures the initial page; omit --scroll")
    binary = args.binary or (
        "target/release/argui-widget-gallery" if args.app else "target/release/examples/footprint"
    )
    command = [binary] + ([] if args.app else [args.page] + (["scroll"] if args.scroll else []))
    child = subprocess.Popen(command, stdout=subprocess.DEVNULL)
    process = Path(f"/proc/{child.pid}")
    try:
        time.sleep(args.warmup)
        if child.poll() is not None:
            raise RuntimeError(f"workload exited with status {child.returncode}")
        if args.app:
            print(json.dumps({
                "phase": "startup",
                "after_seconds": args.warmup,
                "cpu_seconds": round(cpu_seconds(process), 3),
                **{key: round(value, 2) for key, value in memory(process).items()},
                **gpu_memory(process),
            }), flush=True)
        for elapsed in range(0, args.seconds, 5):
            duration = min(5, args.seconds - elapsed)
            if child.poll() is not None:
                raise RuntimeError(f"workload exited with status {child.returncode}")
            before = cpu_seconds(process)
            started = time.monotonic()
            peak_rss = 0
            while time.monotonic() - started < duration:
                if child.poll() is not None:
                    raise RuntimeError(f"workload exited with status {child.returncode}")
                resident_pages = int((process / "statm").read_text().split()[1])
                peak_rss = max(peak_rss, resident_pages * os.sysconf("SC_PAGE_SIZE"))
                time.sleep(0.1)
            cpu = cpu_seconds(process) - before
            seconds = time.monotonic() - started
            result = {
                "page": "initial" if args.app else args.page,
                "scroll": args.scroll,
                "elapsed_s": round(elapsed + seconds, 2),
                "cpu_percent_one_core": round(100 * cpu / seconds, 2),
                "peak_rss_mib": round(peak_rss / 2**20, 2),
                **{key: round(value, 2) for key, value in memory(process).items()},
                **gpu_memory(process),
            }
            print(json.dumps(result), flush=True)
    finally:
        child.terminate()
        try:
            child.wait(timeout=5)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait()


if __name__ == "__main__":
    main()
