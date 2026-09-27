# Getting started

Argui has two application entry points: Solid or React TSX through the Argui
host, and Rust through `argui-runtime`. The TSX desktop host embeds QuickJS;
the browser host runs the Rust renderer as WebAssembly in a canvas and requires
WebGPU. Both use the same retained UI engine. Start with the CLI when building
an application; use the repository gallery to inspect the current controls.

## Prerequisites

Install Rust 1.89 or newer and [Bun](https://bun.sh/). On Linux, the native
host also needs the build packages listed by `argui doctor`. A Web build needs
the `wasm32-unknown-unknown` Rust target, `wasm-pack`, and `wasm-opt` for
release optimization. `argui doctor web` checks the Web tools. Browser
rendering requires a WebGPU capable browser and adapter; the host reports an
error if it cannot acquire one.

## Install the CLI

For the 0.4 release on Linux or macOS, install the verified CLI archive:

```sh
curl -fsSL https://raw.githubusercontent.com/ExtraBinoss/argui/main/scripts/install-cli.sh | bash
argui doctor
```

The 0.4 CLI binaries must be published before this command can install them.
While working from this checkout before that release, run
`cargo install --path crates/argui-cli --locked` from the repository root.
Windows installation, archive verification, path setup, and the full command
reference are in the [CLI guide](cli.md#install-the-04-cli).
The generated project commands below assume the 0.4 crates have also been
published; see that guide for the current source build limit.

## Solid or React application

```sh
argui init solid --dir my-app --yes
cd my-app
argui check
argui dev --target native
```

Choose `react` instead of `solid` for React. `init` creates a shared TSX source
for native and Web targets and runs `bun install` when Bun is available. Use
`--no-install` to install dependencies later. The generated `src/main.tsx`
exports `mountGallery(bridge, expectedAbiHash)`; the native and Web launchers
call that function with their respective bridges. For example, the Solid
template renders a native rectangle and updates a signal on click:

```tsx solid
import { createSignal } from '@argui/solid'

function App() {
  const [count, setCount] = createSignal(0)
  return <column width="100%" height="100%" padding={32} gap={16}>
    <text fontSize={30}>Hello from Argui</text>
    <rectangle width={180} height={48} background="#2563eb"
      onClick={() => setCount(count() + 1)}>
      <text color="#ffffff">{`Count: ${count()}`}</text>
    </rectangle>
  </column>
}
```

This is the `App` body from the checked-in Solid template; the generated file
also creates and disposes its host. For React, the template uses `useState`
and `createRoot` from `@argui/react`.

Build and serve the browser target from the same project:

```sh
argui doctor web
argui dev --target web
argui build release --target web
```

The release output is `dist/web/`. Serve it over HTTP; opening `index.html`
directly as a file is not a supported WebAssembly loading path. The generated
`src/mount.ts` also exports `mountArgui(elementId)` to embed the canvas in a
host page. The [CLI guide](cli.md) covers build targets,
formatting, asset packs, tests, and desktop release directories.

## Rust application

```sh
argui init rust --dir rust-app --yes
cd rust-app
argui dev --target native
```

The [checked-in Rust template](../crates/argui-cli/assets/templates/rust-main.rs)
implements `Render` for a model and passes it through `SingleWindowModel` to
`run_application`. It uses `Context::callback` for the click handler and a
`ThemeRuntime` shared with the window environment. The Rust path does not
embed a JavaScript runtime.

## Explore the repository gallery

These commands are for contributors working in the Argui checkout. An app
created with `argui init` uses the CLI commands above instead. From the
repository root, after `bun install`:

```sh
bun run check:ts
bun run test:ts
bun run dev
```

Use `bun run dev:react` for React or `bun run dev:web:solid` and
`bun run dev:web:react` for browser builds. The Web commands compile the WASM
host before starting Vite. On Linux, run graphical checks through
[`linux-hidden-display.sh`](contributing/linux-testing.md) and inspect a
saved capture. The [gallery guide](../apps/gallery/README.md) describes its
current pages and build loop.

Continue with [CLI commands](cli.md), [crates and features](crates.md),
[architecture and the host contract](architecture.md),
[layout](ui/layout.md), [widgets](ui/README.md), and
[desktop windows](platform/desktop.md).
