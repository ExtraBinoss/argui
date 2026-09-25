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

## Install the CLI

On Linux or macOS:

```sh
curl -fsSL https://raw.githubusercontent.com/ExtraBinoss/argui/main/scripts/install-cli.sh | sh
```

On Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/ExtraBinoss/argui/main/scripts/install-cli.ps1 | iex
```

The installers select your OS and CPU, verify the release archive's SHA-256,
and install `argui` for your user. CLI binaries are published with new Argui
releases containing the CLI workflow. To build it from this checkout instead, run
`cargo install --path crates/argui-cli`.

## Create an app

The [Argui CLI](crates/argui-cli/README.md) creates Solid or React desktop and
browser apps, then checks, builds, and runs them on Linux, macOS, and Windows:

```sh
argui init solid my-app
bun install
argui dev apps/my-app
```

`argui init counter-web` creates a Solid WebAssembly canvas app;
`argui init react counter-web` selects React. Run `argui doctor web` to check
its prerequisites and `argui build release` to produce static files in
`dist/web/`. The generated project README explains how to mount the canvas
in an existing page.
Use `argui dev apps/<name>` and `argui build apps/<name> release` from the
checkout root when it contains several apps; inside an app, omit the path.
Web dev rebuilds WASM when Rust sources change and reloads the page through Vite.
Native dev rebuilds TSX when source files change and applies the new bundle in
the running host.

To try the same Solid counter on both targets from this checkout, run
`argui init counter-web`, `argui init counter-native`, and `bun install` at
the checkout root. Start `argui dev apps/counter-web` and
`argui dev apps/counter-native` in separate terminals. Edit the heading in
either app's `src/main.tsx` to verify its reload. The browser app also rebuilds
WASM after a Rust source change under `crates/` or `apps/web-host/src/`.
When `init` runs outside an Argui checkout, it clones the matching release
into `my-app/`; run `bun install` there and use `my-app/apps/my-app` as the
project directory.

When run inside an existing Argui checkout, `init` creates the app in that
checkout's `apps/` directory. From another directory it creates a versioned
Argui checkout named `my-app`, then generates `apps/my-app` inside it.

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
