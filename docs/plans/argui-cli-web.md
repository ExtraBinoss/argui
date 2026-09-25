# Argui CLI and web target plan

## Goal

One installed `argui` CLI creates, checks, builds, and runs Solid or React
applications on desktop and in the browser. The browser target uses Argui's
Rust runtime and renderer compiled to WebAssembly and displays it in a canvas.
CLI output and generated project documentation are in English.

## Current baseline in this worktree

- `crates/argui-cli` has native Solid/React project creation, TypeScript checks,
  native builds/runs, icons, prerequisite diagnostics, and an English start screen.
- Global `init` clones the GitHub release matching the CLI version, then creates
  `apps/<name>` in that checkout. A local `init` creates `apps/<name>` directly.
- `add` reads `components/registry.json` and copies declared files from
  `packages/widgets/src` in that same checkout. This keeps the host, widgets,
  and registry on one version and works offline after initialization.
- The native QuickJS gallery host can load an external app bundle through
  `ARGUI_APP_BUNDLE`; release builds create a portable desktop directory.
- Install scripts and a five-target GitHub Actions matrix are prepared. The
  existing `[PUBLISH]` release job will publish the CLI crate and then attach
  CLI binaries/checksums. Publication needs a **new workspace version** after
  these changes reach `main`; `v0.3.2` already exists.
- Gallery/runtime commit `5e7d726` is merged into this worktree. The local
  registry maps `button`, `input-field`, `select`, `popover`, and `dialog` to
  their separate Solid and React files and includes their local imports.
- The coordinated workspace and standalone host versions are now `0.3.3`.
  Release remains gated by the main branch's `[PUBLISH]` workflow.

## Commands and generated layout

1. Keep `argui init solid <name>` and `argui init react <name>` as native
   compatibility forms. Add `argui init counter-native` and
   `argui init counter-web` as Solid defaults; offer an explicit React variant
   without changing the project names' meaning.
2. Store the target (`native` or `web`) in `argui.json`. `argui check [path]`,
   `argui build [path] dev|release`, and `argui dev [path]` select an app by
   path and its target from that manifest. The path is optional inside an app.
3. A web project contains `index.html`, `src/main.tsx`, a Vite config, and a
   short README showing both the standalone page and how to import/mount the
   app into an existing page element. The default page has a responsive Argui
   canvas, welcome text, and a working counter.
4. Keep native desktop packaging unchanged. Web `build release` writes static
   HTML, JavaScript, and optimized `.wasm` into `dist/web`; `run dev` starts
   Vite and provides its local URL; `run release` previews `dist/web`. Native
   `dev` rebuilds changed TSX and lets the existing QuickJS bundle watcher
   update the running scene.

## Browser bridge

1. Build a small WebAssembly host crate around the existing `argui-host`
   transaction protocol, schema contract, `argui-runtime`, and `argui-render`.
   The JavaScript `NativeBridge` used by Solid and React must preserve the same
   ABI hash and operation/event semantics as QuickJS.
2. Add the browser event-loop entry that connects host commits to the retained
   UI tree and WGPU canvas. Reuse the existing Winit web canvas attachment and
   WebGPU/WebGL renderer paths. No separate DOM widget implementation.
3. Deliver pointer, keyboard, focus, resize, and accessibility events back to
   the TSX callbacks. Reject a stale ABI or invalid transaction with a useful
   browser error. Keep the renderer/runtime independent of the TSX adapters.
4. During development, Vite refreshes the page when TSX changes. Poll Rust
   sources and Cargo manifests across operating systems, rebuild WASM after a
   stable change, and let Vite reload the page when the generated module
   changes. This is a full reload; retained state preservation is a separate
   improvement.

## Build tools and releases

1. Extend `argui doctor` to report browser-target prerequisites: Bun, Rust's
   `wasm32-unknown-unknown` target, `wasm-pack`/`wasm-bindgen`, and `wasm-opt`.
   Messages include install links and identify which target needs each tool.
2. Web `build dev` compiles Rust/WASM and the Vite app with debugging output.
   Web `build release` runs `wasm-opt` on the emitted WASM before finalizing the
   static bundle; failure to find or run it is a build error, not a silent skip.
3. Add web build verification to CI and keep the CLI binary release gated by
   the repository's shared version rule. The install scripts continue to cover
   Linux, macOS, and Windows without asking users to run OS-specific build
   commands inside a project.

## Integration and verification

1. Keep the registry aligned with the split widgets. Its transitive imports
   include `assets`, shared types/theme/input helpers, and `button` where used.
   `add` copies only the selected framework and declared dependencies.
2. Keep the gallery host's `runner.rs` bundle override when reconciling the
   other worktree's edits to that same file.
3. Test Solid and React scaffolds, TypeScript checks, native bundles, selective
   component copying, browser contract/transactions, dev reload, release WASM
   optimization, and the generated embed instructions. Honor the user's request
   to avoid window launches and visual tests for this integration.
4. Run targeted `cargo nextest run --all-features` checks during implementation,
   then the repository's full `quality.sh` gate once after the integrated code
   is complete. The gate requires at least 85% for each coverage metric in
   every crate. Publish only after a passing gate and a new versioned commit on
   `main` triggers the existing release workflow.

## Known constraints

- `argui-runtime` already has a WASM event loop and browser canvas attachment,
  but `run_native_host` currently uses native threads/channels and is gated off
  for WASM. The browser bridge must provide an event-loop-safe commit path.
- Browser builds should prefer WebGPU and use the renderer's existing WebGL
  support where available; unsupported browser/device states need an explicit
  error in the generated page.
- The existing `v0.3.2` GitHub release cannot receive these CLI assets. The
  current `0.3.3` sources must be integrated into `codex/dsl-gallery-live`,
  then merged to `main` with a `[PUBLISH]` commit for CI to create `v0.3.3`.
