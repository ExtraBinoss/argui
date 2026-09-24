#!/usr/bin/env python3
"""Compare already-built release binaries without compilation during timing."""

import argparse
import hashlib
import json
import statistics
import subprocess
import sys
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--runs", type=int, default=7)
    parser.add_argument("--cpu", type=int, default=0)
    args = parser.parse_args()
    if args.runs < 3:
        parser.error("use at least three runs")
    directories = {"before": args.baseline, "after": args.candidate}
    cases = [
        ("tree_profile", ["1000", "100"]),
        ("tree_profile", ["10000", "100"]),
        ("tree_profile", ["10000", "10"]),
        ("layout_profile", ["1000"]),
        ("layout_profile", ["10000"]),
    ]
    results = []
    for binary, workload in cases:
        runs = args.runs
        samples = {label: [] for label in directories}
        # One unrecorded warm-up, then alternating order to limit thermal/order bias.
        for run in range(runs + 1):
            labels = list(directories)
            if run % 2:
                labels.reverse()
            for label in labels:
                print(f"{binary} {workload}: {label} {run}/{runs}", file=sys.stderr, flush=True)
                command = [
                    "taskset", "-c", str(args.cpu),
                    str((directories[label] / binary).resolve()), *workload,
                ]
                value = json.loads(subprocess.check_output(command, text=True))
                if run:
                    samples[label].append(value)
        medians = {
            label: {key: statistics.median(row[key] for row in rows) for key in rows[0]}
            for label, rows in samples.items()
        }
        result = {"binary": binary, "args": workload, "runs": runs, "medians": medians, "samples": samples}
        results.append(result)
        print(json.dumps({key: value for key, value in result.items() if key != "samples"}), flush=True)
    heap = {}
    for label, directory in directories.items():
        heap[label] = {}
        for name in ("tree", "layout", "churn"):
            profile = directory / f"{name}-heap.json"
            if profile.exists():
                points = json.loads(profile.read_text())["pps"]
                heap[label][name] = {
                    key: sum(point[key] for point in points)
                    for key in ("tb", "tbk", "gb", "eb", "ebk", "rb", "wb")
                }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    hashes = {
        label: {binary: hashlib.sha256((directory / binary).read_bytes()).hexdigest() for binary in {case[0] for case in cases}}
        for label, directory in directories.items()
    }
    args.output.write_text(json.dumps({"cpu": args.cpu, "runs": args.runs, "binary_sha256": hashes, "cases": results, "dhat": heap}, indent=2) + "\n")


if __name__ == "__main__":
    main()
