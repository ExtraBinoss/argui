# Argui contributor rules

Read `docs/code_quality.md` before changing code.

For Linux GUI checks, read [docs/linux_testing.md](docs/linux_testing.md) and use
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
- During implementation, run only the directly affected crate/tests and measure
  behavior with `cargo nextest run --all-features`. Keep feature flags identical
  between targeted runs so Cargo reuses one artifact variant. Do not repeatedly
  run the whole workspace.
- Never run LLVM coverage concurrently; its instrumented target is shared and
  concurrent variants waste compilation time and disk space.
- Run `./scripts/quality.sh` exactly once after the implementation is complete
  and immediately before committing; nothing passes below 85% on any coverage
  metric.
