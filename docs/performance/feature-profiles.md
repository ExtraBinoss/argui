# Facade feature profiles

Argui keeps its granular `widget-*` features. The `basic`, `desktop`, and `web`
features are convenience aliases for common applications; they do not enable a
new implementation path and the facade still has no default features.

| Profile | Expands to |
| --- | --- |
| `basic` | tasks plus button, checkbox, dialog, input, popover, radio-group, select, switch, tabs, and textarea widgets |
| `desktop` | `basic` plus backdrop, file picker, global shortcuts, native popups, tray, updater, and their widgets |
| `web` | `basic` plus the webview integration |

Applications that need a smaller graph should continue to select individual
features. Platform integrations are deliberately absent from `basic`.

## Measurement

Measurements were taken on 2026-09-16 on Linux x86_64 with Rust 1.98.0. The
probe is a minimal application that imports the facade and constructs a basic
view. Cargo used the repository's shared target directory. The comparison is
between spelling the complete granular list and selecting its convenience
alias.

| Selection | Clean release build | Stripped native binary |
| --- | ---: | ---: |
| granular `basic` list | 201.26 s | 1,645,456 bytes |
| `basic` alias | 217.84 s | 1,645,456 bytes |

The time difference is clean-build noise: `cargo tree -e features` resolves the
same feature graph, and the byte-identical size confirms that the alias adds no
code. The same invariant applies to `desktop` and `web`, whose aliases expand
only to the documented granular features. Incremental validation in the shared
target directory completed in 2.73 s for `basic`, 13.62 s for `desktop`, and
17.32 s for `web` on `wasm32-unknown-unknown`. Release artifacts therefore do
not need a separate build or target directory merely because an alias is used.

Reproduce the supported-target checks from the repository root:

```sh
cargo check -p argui --no-default-features --features basic
cargo check -p argui --no-default-features --features desktop
cargo check -p argui --no-default-features --features web \
  --target wasm32-unknown-unknown
```

For a before/after comparison, replace the alias with the exact expansion from
`crates/argui/Cargo.toml`. Cargo must resolve the same dependency and feature
graph; a difference is a regression in the profile definition.
