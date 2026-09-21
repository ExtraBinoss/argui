# Argui DSL

This directory contains the language toolchain and standard library. The
[implementation plan](../../docs/dsl/ARGUI_DSL_IMPLEMENTATION_PLAN.md) defines
the architecture and exit criteria. The [phase-by-phase checklist](../../docs/dsl/README.md)
tracks what is implemented, what still needs verification, and what remains
open. An unchecked phase is not delivered merely because another part works.

Develop the independent widget gallery with live reload from
`app_examples/widget-gallery-dsl/`:

```sh
cargo run --manifest-path ../../Cargo.toml -p argui-cli --bin argui -- dev
```

The gallery's [README](../../app_examples/widget-gallery-dsl/README.md) covers
its components, assets, theme modes, release command, and acceptance checks.
Run targeted `cargo nextest run -p PACKAGE --all-features` tests while editing;
the repository-wide quality gate belongs at the end of implementation.
