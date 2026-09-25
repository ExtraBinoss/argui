#!/usr/bin/env python3
"""Enforce coverage globally and in every workspace crate."""
import json
import re
from pathlib import Path
import sys
import tomllib

METRICS = ("branches", "functions", "lines", "regions")


def workspace_crates():
    workspace = tomllib.loads(Path("Cargo.toml").read_text())["workspace"]
    return sorted(str(Path(member).relative_to("crates"))
                  for member in workspace["members"] if member.startswith("crates/"))


def only_reexports(crate):
    sources = list((Path("crates") / crate / "src").rglob("*.rs"))
    if not sources:
        return False
    for source in sources:
        text = re.sub(r"//[^\n]*", "", source.read_text())
        text = re.sub(r"#!?\[[^\]]*\]", "", text)
        text = re.sub(r"pub\s+use\s+[^;]+;", "", text)
        text = re.sub(r"pub\s+const\s+[A-Z][A-Z0-9_]*\s*:\s*&str\s*=\s*include_str!\([^;]+\);", "", text)
        text = re.sub(r"pub\s+mod\s+[A-Za-z_][A-Za-z0-9_]*\s*\{", "", text)
        text = re.sub(r"(?ms)^macro_rules!\s+[A-Za-z_][A-Za-z0-9_]*\s*\{.*?^\}", "", text)
        text = text.replace("}", "")
        if text.strip():
            return False
    return True


def source_summary(entry):
    """Union generic instantiations at each source branch, retaining both outcomes."""
    branches = {}
    for branch in entry["branches"]:
        outcomes = branches.setdefault(tuple(branch[:4]), [False, False])
        outcomes[0] |= branch[4] > 0
        outcomes[1] |= branch[5] > 0
    metrics = dict(entry["summary"])
    count = len(branches) * 2
    if count != metrics["branches"]["count"]:
        # Folded expressions and macro expansions need LLVM's own accounting.
        return metrics
    metrics["branches"] = {
        "count": count,
        "covered": sum(sum(outcomes) for outcomes in branches.values()),
    }
    return metrics


def aggregate(entries):
    summaries = [source_summary(entry) for entry in entries]
    return {
        metric: {field: sum(summary[metric][field] for summary in summaries)
                 for field in ("count", "covered")}
        for metric in METRICS
    }


def run(report_path, minimum):
    report = json.loads(Path(report_path).read_text())
    files = [entry for data in report["data"] for entry in data["files"]]
    groups = {"workspace": aggregate(files)}
    errors = []
    for crate in workspace_crates():
        entries = [entry for entry in files if f"/crates/{crate}/" in entry["filename"].replace("\\", "/")]
        if not entries and only_reexports(crate):
            groups[crate] = {metric: {"count": 0, "covered": 0} for metric in METRICS}
            continue
        if not entries:
            errors.append(f"{crate}: missing coverage report")
            continue
        groups[crate] = aggregate(entries)
    results = {}
    for name, metrics in groups.items():
        results[name] = {}
        for metric in METRICS:
            covered, count = metrics[metric]["covered"], metrics[metric]["count"]
            percent = covered / count * 100 if count else None
            results[name][metric] = {"covered": covered, "count": count, "percent": percent}
            label = "N/A" if percent is None else f"{percent:.2f}% ({covered}/{count})"
            print(f"coverage {name} {metric}: {label}")
            if count and covered * 100 < minimum * count:
                errors.append(f"{name} {metric}: {percent:.2f}% below {minimum}%")
    Path(report_path).with_name("coverage-gate.json").write_text(json.dumps({"minimum": minimum, "results": results, "errors": errors}, indent=2) + "\n")
    for error in errors:
        print(f"error: {error}", file=sys.stderr)
    return bool(errors)


if __name__ == "__main__":
    sys.exit(run(sys.argv[1], float(sys.argv[2])))
