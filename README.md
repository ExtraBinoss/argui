# Argui

Argui is a retained Rust UI engine for native windows and WebAssembly. Solid
and React TSX applications describe a shared native element tree; Rust owns
layout, text, input, accessibility, animation, and WGPU rendering. Rust
applications can use the runtime directly without JavaScript. The desktop TSX
host embeds QuickJS. The browser host renders into a WebGPU canvas.

## Start an application

Once the 0.4 CLI release archives are published, install the verified binary
on Linux or macOS:

```sh
curl -fsSL https://raw.githubusercontent.com/ExtraBinoss/argui/main/scripts/install-cli.sh | bash
```

Before publication, build this checkout's CLI with Rust 1.89 or newer:

```sh
cargo install --path crates/argui-cli --locked
argui doctor
```

A generated app also needs the 0.4 crates to be published or patched to this
checkout. Once those crates are available, create and run an app with the CLI:

```sh
argui init solid --dir my-app --yes
cd my-app
argui check
argui dev --target native
```

Choose `react` instead of `solid` for React, or `rust` for a native Rust
project. The Solid and React templates use one TSX source for native and Web.
Run `argui doctor web` and `argui dev --target web` inside a TSX project to
serve its browser build. Web rendering needs a WebGPU capable browser and
adapter. The [getting started guide](docs/getting-started.md) covers the
prerequisites, both targets, and the generated application files. The
[CLI guide](docs/cli.md) covers installation, builds, testing, assets, and
release output. The [crates and features guide](docs/crates.md) explains which
Rust dependencies and optional features belong in an application.

The 0.4 CLI release will have [Linux/macOS](scripts/install-cli.sh) and
[Windows](scripts/install-cli.ps1) installers. Each verifies its downloaded
archive before installing it.

## Explore the gallery

Install [Bun](https://bun.sh/), then from the repository root:

```sh
bun install --frozen-lockfile
bun run check:ts
bun run test:ts
bun run dev
```

`bun run dev:react` runs the native React gallery. `bun run dev:web:solid`
and `bun run dev:web:react` build the WASM host and start the browser gallery.
The current gallery has 12 widget pages in each adapter, plus layout,
scrollbar, animation, and desktop capability examples. The reusable controls
are exported from `@argui/widgets/solid` and `@argui/widgets/react`; the
[gallery guide](apps/gallery/README.md) names the current pages and limitations.
On Linux, graphical checks use the
[private display helper](docs/contributing/linux-testing.md).

## Website

The [Nuxt website](website/README.md) renders the current documentation on the
server and embeds the real Solid and React WASM galleries on its Components
page. It uses Bun and Pinia; the browser canvas requires WebGPU. Build both
gallery adapters with `cd website && bun install --frozen-lockfile && bun run
gallery:build`, then run `bun run dev`, `bun run build` for SSR, or `bun run
generate` for pre-rendered static output.

## Documentation

The [documentation index](docs/README.md) links to architecture, layout,
animation, UI and widgets, desktop windows and OS limits, platform services,
and contributor guides. The [host contract](docs/solid-react-native.md)
explains how the Solid and React adapters share the Rust engine.

## Contribute

Read [AGENTS.md](AGENTS.md), the [development guide](docs/contributing/development.md),
and the [code quality rules](docs/contributing/code-quality.md). The complete
quality gate checks source hygiene, Rust and WASM builds, public API examples,
and per-crate coverage.

## License

Dual licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your
option.
