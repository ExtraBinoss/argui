# Argui CLI

The `argui` binary creates Solid and React TSX applications for desktop and
browser. Desktop apps use the QuickJS host; browser apps use the Rust renderer
compiled to WebAssembly inside a canvas.
Commands and project files are the same on Linux, macOS, and Windows.

```sh
cargo install --path crates/argui-cli
argui
argui doctor
argui doctor web
argui init solid my-app
bun install
cd apps/my-app
argui check
argui dev
argui build release
```

`argui init react my-app` creates a React variant. Without a name, `init`
uses `argui-solid-app` or `argui-react-app`. Inside an Argui checkout, it
creates `apps/my-app`. Elsewhere, it clones the matching Argui release into
`my-app` and creates `my-app/apps/my-app`; this keeps the application tied to
the exact host and Bun workspace version used by its CLI. Run `bun install`
from the checkout root after creating a project. `argui check --json` emits
JSON diagnostics for editors.

When managing several apps, select one from the checkout root with
`argui check apps/my-app`, `argui build apps/my-app release`, or
`argui dev apps/my-app`. The path can also be absolute. Inside an app,
the path is optional. `argui run apps/my-app dev|release` remains available.

`argui init counter-web` creates a Solid browser counter;
`argui init counter-native` creates its desktop counterpart. Use
`argui init react counter-web` for React in the browser. Web projects contain
`index.html`, a responsive Argui canvas, and `src/mount.ts`, which exports
`mountArgui(elementId)` for embedding the app in an existing page. The generated
README shows the required element and import. Run `argui run dev` for a Vite
development server, or `argui build release` to create a static `dist/web/`
directory. Release builds require `wasm-opt` and fail with an install link if
it is missing. `argui doctor web` checks browser build prerequisites. In dev,
Argui recompiles WASM after Rust changes and Vite reloads the page; TSX edits
use Vite's normal reload. Rust state is reset after a WASM rebuild.

`argui build dev` and `argui build release` compile the TSX bundle and the
native host without opening a window. `argui run dev|release` then launches
that host with the project bundle. The Rust host is shared between projects in
`target/argui-native/`. The gallery bundle is built once if its embedded
fallback does not exist yet. `argui dev [path]` watches native TSX sources,
rebuilds the bundle, and lets the running QuickJS host reload the scene.
Set `ARGUI_VALIDATE_ONLY=1` when running a desktop app to mount and validate
its TSX bundle without opening a window; this is useful in CI.

Release builds also create `dist/desktop/` with the native executable,
`app.mjs`, and a platform launcher (`run.sh` or `run.cmd`). This is a portable
desktop directory; it is not a signed or installed `.dmg`, `.msi`, or Linux
package. Build release packages on each target OS. Use `argui icon source.png`
to generate PNG sizes, a Windows ICO, and a macOS iconset; macOS also runs
`iconutil` to make `app.icns`. Existing icon files are copied into the release
directory. Signing and installer generation remain downstream distribution
steps.

`argui add solid button` or `argui add react input-field` reads
[`components/registry.json`](../../components/registry.json) from the project's
Argui checkout. A global `argui init` clones the matching GitHub release, so
component files and dependencies stay on the same version as the native host.
The registry lists each component's source files and dependencies. The CLI
copies only those files from `packages/widgets/src` into `src/argui-ui/` and
records them in `argui.json`. It works offline after initialization. Tracked
files are kept on repeated runs; untracked conflicts, invalid paths, missing
variants, and oversized sources fail with clear messages.

`argui doctor` reports missing Bun, Rust, C compiler, and Linux native build
prerequisites with installation links. QuickJS ships in the Rust host;
its bindings use libclang. The host does not require a separate QuickJS CLI.
