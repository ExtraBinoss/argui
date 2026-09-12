#!/usr/bin/env python3
"""Compare saved executables on selected workloads; never builds or samples the runtime."""
import argparse
import hashlib
import json
import re
import statistics
import subprocess
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("before", type=Path)
    parser.add_argument("after", type=Path)
    parser.add_argument("--cases", nargs="+", required=True)
    parser.add_argument("--frames", type=int, default=100)
    parser.add_argument("--runs", type=int, default=3)
    parser.add_argument("--cpu", type=int, default=0)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.runs < 3 or args.frames < 1:
        parser.error("at least three paired runs and one frame are required")
    directories = {"before": args.before, "after": args.after}
    results = []
    for case in args.cases:
        binary = "incremental_profile" if case in {"paint", "batch", "panel", "structure", "wide"} else "update_profile"
        if case == "boundary":
            binary = "boundary_profile"
        samples = {label: [] for label in directories}
        for run in range(args.runs + 1):
            labels = list(directories)
            if run % 2:
                labels.reverse()
            for label in labels:
                mode = ("normal" if label == "before" else "isolated") if case == "boundary" else case
                command = ["taskset", "-c", str(args.cpu), str((directories[label] / binary).resolve()), mode, str(args.frames)]
                measured = subprocess.run(["/usr/bin/time", "-f", "ARGUI_RESOURCE %U %S %M", *command], capture_output=True, text=True, check=True)
                sample = json.loads(measured.stdout)
                user, system, rss = re.search(r"ARGUI_RESOURCE ([\d.]+) ([\d.]+) (\d+)", measured.stderr).groups()
                sample.update(user_s=float(user), system_s=float(system), rss_kib=int(rss))
                if run:
                    samples[label].append(sample)
        invariant = lambda sample: {key: value for key, value in sample.items() if key not in {"elapsed_ns", "user_s", "system_s", "rss_kib"}}
        expected = invariant(samples["before"][0])
        assert all(invariant(row) == expected for rows in samples.values() for row in rows), (case, samples)
        medians = {label: statistics.median(row["elapsed_ns"] for row in rows) for label, rows in samples.items()}
        change = 100 * (medians["after"] / medians["before"] - 1)
        results.append({"case": case, "binary": binary, "samples": samples, "median_ns": medians, "change_percent": change})
        print(f"{case}: {medians['before'] / 1e6:.2f} -> {medians['after'] / 1e6:.2f} ms ({change:+.1f}%)", flush=True)
    hashes = {
        label: {binary: hashlib.sha256((directory / binary).read_bytes()).hexdigest() for binary in {case["binary"] for case in results}}
        for label, directory in directories.items()
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps({"cpu": args.cpu, "frames": args.frames, "runs": args.runs, "binary_sha256": hashes, "cases": results}, indent=2) + "\n")


if __name__ == "__main__":
    main()
