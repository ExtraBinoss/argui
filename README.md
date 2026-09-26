# Argui

Argui is a native UI engine for Solid and React TSX. Rust owns layout, text,
input, accessibility, and WGPU rendering; JavaScript describes the interface
through the shared host. The [gallery](apps/gallery/README.md) demonstrates both
framework adapters.

## Run the gallery

Install Rust and Bun, then run:

```sh
bun install
bun run gallery
```

Use `bun run gallery:react` for the React gallery. Reusable TSX controls live
in `@argui/widgets/react` and `@argui/widgets/solid`. The current gallery
demonstrates Button, ButtonGroup, InputField, Select, Popover, VirtualList, and the layout
primitives. Previous components are archived in `OLD_API/`.

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

Use `argui update --check` to see whether a newer CLI release is available,
or `argui update` to download and install its SHA-256-verified binary.

## Create an app

The [Argui CLI](crates/argui-cli/README.md) creates Solid or React desktop and
browser apps, then checks, builds, and runs them on Linux, macOS, and Windows:

```sh
argui init solid --dir my-app --yes
cd my-app
argui format
argui check
argui dev --target native
```

`init` puts Oxfmt in the new application's `package.json` and runs
`bun install` when Bun is available. Use `--no-install` to defer installation.
For a React Web-only app, run `argui init react --dir web-app --targets web --yes`,
then `argui dev --target web` inside it. `argui build release --target web`
produces `dist/web/`. Use `argui format --check` to verify TSX formatting
without changing files. The [CLI guide](crates/argui-cli/README.md) covers
target-specific builds, asset packs, and app paths.

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
