# Argui contributor rules

Read `docs/contributing/code-quality.md` before changing code.
Before creating any Argui component, using Argui primitives in Rust or TSX,
or changing an Argui gallery/example, read
`skills/argui-common-pitfalls/SKILL.md` and follow the relevant guidance.

For Linux GUI checks, read [docs/contributing/linux-testing.md](docs/contributing/linux-testing.md) and use
`./scripts/linux-hidden-display.sh COMMAND...`. Keep test windows and browsers
on that private display; inspect saved captures instead of opening windows on
the user's desktop. A blank capture is a failed visual check.

- Write the least code that cleanly solves the current step.
- Keep every `.rs` file at 600 physical lines or fewer.
- Split by responsibility; do not create folders or abstractions speculatively.
- Put all tests in `tests/`, mirroring paths under `src/`; test code is forbidden
  under `src/`.
- Add dependencies only to the crate that uses them.
- Keep the renderer/runtime independent from any future DSL.
- Document every new function and method with accurate Rustdoc, and update that
  documentation whenever its signature or behavior changes. Explain the
  purpose of each parameter and return value, and document applicable errors
  and panics.
- During implementation, run only the directly affected crate/tests and measure
  behavior with `cargo nextest run --all-features`. Keep feature flags identical
  between targeted runs so Cargo reuses one artifact variant. Do not repeatedly
  run the whole workspace.
- Never run LLVM coverage concurrently; its instrumented target is shared and
  concurrent variants waste compilation time and disk space.
- Run `./scripts/quality.sh` exactly once after the implementation is complete
  and immediately before committing; nothing passes below 85% on any coverage
  metric.
