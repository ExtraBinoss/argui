#!/usr/bin/env python3
"""Check the empty, complete and individual widget feature configurations."""
import pathlib
import subprocess
import tempfile
import tomllib

ROOT = pathlib.Path(__file__).resolve().parent.parent


def manifest(crate):
    return tomllib.loads((ROOT / "crates" / crate / "Cargo.toml").read_text())


def check(crate, feature=None):
    print(f"features: {crate} / {feature or 'none'}", flush=True)
    command = ["cargo", "check", "--quiet", "-p", crate, "--lib", "--no-default-features"]
    if feature:
        command.extend(["--features", feature])
    subprocess.run(command, cwd=ROOT, check=True)


def main():
    widgets = manifest("argui-widgets")["features"]
    facade = manifest("argui")["features"]
    assert widgets["default"] == facade["default"] == [], "Widgets must be opt-in"
    individual = set(widgets) - {"all", "default"}
    assert set(widgets["all"]) == individual - {"updater"}, \
        "all must include every standalone widget feature"
    assert facade["widgets-all"] == ["argui-widgets/all"]
    for feature in individual:
        assert facade[f"widget-{feature}"] == [f"argui-widgets/{feature}"]
    check("argui-widgets")
    for feature in sorted(individual):
        check("argui-widgets", feature)
    check("argui-widgets", "all")
    types = {feature: "".join(part.title() for part in feature.split("-")) for feature in individual}
    types.update({"textarea": "TextArea", "vlist": "VList", "range": "RangeState",
                  "icons": "WidgetAssets", "text-selection": "TextSelectionToolbar",
                  "implicit-animation": "AnimatedOpacity", "updater": "UpdateDialog"})
    with tempfile.TemporaryDirectory(prefix="argui-features-") as directory:
        probe = pathlib.Path(directory)
        (probe / "src").mkdir()
        (probe / "Cargo.toml").write_text(
            '[package]\nname="argui-feature-probe"\nversion="0.0.0"\nedition="2024"\n'
            '[workspace]\n[dependencies]\n'
            f'argui={{path="{ROOT / "crates/argui"}",default-features=false}}\n'
            '[features]\ndefault=[]\nall=["argui/widgets-all"]\n'
            + "".join(f'{feature}=["argui/widget-{feature}"]\n' for feature in sorted(individual))
        )
        imports = []
        for feature, widget in sorted(types.items()):
            condition = (f'feature="{feature}"' if feature == "updater"
                         else f'any(feature="{feature}",feature="all")')
            imports.append(
                f'#[cfg({condition})]\nuse argui::widgets::{widget} as _;\n'
            )
        (probe / "src/main.rs").write_text(
            "#![allow(unused_imports)]\nfn main() {}\n" + "".join(imports)
        )
        for feature in [None, *sorted(individual), "all"]:
            print(f"features: external argui import / {feature or 'none'}", flush=True)
            command = ["cargo", "check", "--quiet", "--offline", "--manifest-path", str(probe / "Cargo.toml"),
                       "--target-dir", str(ROOT / "target"), "--no-default-features"]
            if feature:
                command.extend(["--features", feature])
            subprocess.run(command, cwd=ROOT, check=True)
    tree = subprocess.check_output(
        ["cargo", "tree", "-p", "argui", "--no-default-features", "--edges", "normal", "--prefix", "none"],
        cwd=ROOT, text=True,
    )
    assert "argui-widgets " not in tree, "The facade must not pull widgets in by default"
    print(f"Verified {len(individual)} individual widget features, both empty defaults and both complete configurations.")


if __name__ == "__main__":
    main()
