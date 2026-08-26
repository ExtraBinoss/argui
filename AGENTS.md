# Argui contributor rules

Read `docs/code_quality.md` before changing code.

- Write the least code that cleanly solves the current step.
- Keep every `.rs` file at 600 physical lines or fewer.
- Split by responsibility; do not create folders or abstractions speculatively.
- Put integration tests in `tests/`, mirroring paths under `src/`.
- Add dependencies only to the crate that uses them.
- Keep the renderer/runtime independent from any future DSL.
- Run `./scripts/quality.sh`; nothing passes below 85% on any coverage metric.
