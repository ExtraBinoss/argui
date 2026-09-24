# Argui

Argui is a native UI engine for Solid and React TSX. Rust owns layout, text,
input, accessibility, and WGPU rendering; JavaScript describes the interface
through the shared host. The [gallery](apps/gallery/README.md) demonstrates both
framework adapters.

## Run the gallery

Install Rust and Bun, then run:

```sh
bun install
./scripts/gallery-hot-reload.sh desktop solid
```

Use `react` in place of `solid` to run the React gallery. Reusable TSX controls
live in `@argui/widgets/react` and `@argui/widgets/solid`.

## Engine

The retained engine provides flex and grid layout, scrolling, virtualization,
text shaping and editing, pointer and keyboard input, AccessKit semantics,
animation, custom WGSL, and WGPU rendering. The renderer and runtime remain
independent of the TSX adapters. The [architecture](docs/architecture.md) and
[platform guide](docs/README.md) describe those boundaries.

## Contribute

Read the [development guide](docs/contributing/development.md) and
[code quality rules](docs/contributing/code-quality.md). The full Rust quality
gate requires at least 85% coverage for lines, functions, regions, and branches
in each workspace crate.

## License

Dual licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option.
