# Argui CLI

The `argui` command is the application workflow for Argui 0.4. It creates
projects, checks them, runs native or Web targets, builds releases, and tests
native TSX applications. Run these commands inside the generated application
unless a path is shown. The [crates and features guide](crates.md) explains
which Rust dependencies the CLI generates and when to add others.

## Install the 0.4 CLI

For Linux and macOS:

```sh
curl -fsSL https://raw.githubusercontent.com/ExtraBinoss/argui/main/scripts/install-cli.sh | bash
argui doctor
```

The installer selects the OS and CPU archive from the latest release, verifies
its published SHA-256, and puts `argui` in `~/.local/bin` by default. Add that
directory to `PATH` if the installer asks. Set `ARGUI_INSTALL_DIR` to choose
another directory. To pin the 0.4 release, set
`ARGUI_VERSION=v0.4.0` for the install command.

On Windows, run the PowerShell installer:

```powershell
irm https://raw.githubusercontent.com/ExtraBinoss/argui/main/scripts/install-cli.ps1 | iex
argui doctor
```

To build the CLI from this checkout, use Rust 1.89 or newer:

```sh
cargo install --path crates/argui-cli --locked
argui doctor
```

A source build keeps the CLI aligned with this checkout. The installer requires
the matching CLI archive and SHA-256 file on the GitHub release; use the source
build if those assets are temporarily unavailable.

For Solid or React projects, install [Bun](https://bun.sh/) too. The CLI runs
`bun install` during `init` when Bun is present; `--no-install` skips that step.
For Web output, run `argui doctor web` to check the Rust WebAssembly target,
`wasm-pack`, and `wasm-opt`. The browser renderer also requires WebGPU.

## Create an application

```sh
argui init solid --dir my-app --yes
cd my-app
argui check
argui dev --target native
```

Use `react` instead of `solid` for React. Both TSX templates target native
desktop and Web by default, sharing one `src/main.tsx`. To create only a Web
app, add `--targets web`. For a native Rust application:

```sh
argui init rust --dir rust-app --yes
cd rust-app
argui check
argui dev --target native
```

The Rust scaffold currently targets native desktop. The CLI does not scaffold
an iOS or Android package. `argui init --feature tasks` wires the runtime task
feature into a generated app. `--feature automation` is available for native
Solid or React projects; the supported `init` features are deliberately
limited. Other Rust capabilities are selected in `Cargo.toml`, as described in
[crates and features](crates.md).

## Command reference

| Command | Use |
| --- | --- |
| `argui` or `argui --help` | Show the start screen or all commands. |
| `argui doctor` / `argui doctor web` | Check native or Web build prerequisites. |
| `argui init rust\|solid\|react --dir NAME --yes` | Create a project; `--targets native,web` narrows TSX output. |
| `argui check [path]` | Validate a project and its generated assets; `--json` emits editor diagnostics. |
| `argui format [path]` | Format TSX project source; `--check` only reports differences. |
| `argui dev [path] --target native\|web` | Run the native app with live TSX reload, or start the Web dev server. |
| `argui build [path] dev\|release --target native\|web` | Build without opening a window; release files land in `dist/desktop` or `dist/web`. |
| `argui run [path] dev\|release` | Launch an already selected build mode. |
| `argui list components` | List components in the release registry; accepts `--solid`, `--react`, and `--json`. |
| `argui add button` | Copy a registered TSX widget into the app; accepts `--solid`, `--react`, and `--project PATH`. |
| `argui test FILE.test.ts --out DIR` | Run a native TSX interaction test without a desktop window. |
| `argui screenshot --out FILE.png` | Capture a native frame. |
| `argui icon SOURCE.png` | Generate platform icon files for a native release. |
| `argui update --check` / `argui update` | Check or install a newer verified CLI binary. |

The component registry is a source-copy catalog, not the complete widget API.
`argui list components` shows what the matching published CLI can install.
The release CLI verifies the component source against its version; a locally
built development snapshot can reject `argui add` until its matching release
registry is published. [Widgets and controls](ui/controls.md) distinguishes
the gallery pages from that registry.

## Native and Web builds

Inside a generated Solid or React project:

```sh
argui doctor web
argui dev --target web
argui build release --target web
argui build release --target native
```

Serve `dist/web/` over HTTP; loading its `index.html` as a local file cannot
load the WebAssembly host. A native release is a portable `dist/desktop/`
directory with a launcher, not a signed OS installer. Build each desktop
release on its target OS. See [getting started](getting-started.md) for the
generated TSX and Rust entry points.

## CLI commands and repository commands

Use `argui init`, `check`, `dev`, `build`, `test`, and `add` for an application.
Commands such as `bun run test:ts`, `cargo nextest`, and the gallery build
scripts operate on this repository itself. They appear only in the gallery or
contributor guides; an app created by `argui init` does not need that checkout.
