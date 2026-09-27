# Argui website

The website is a server-rendered Nuxt 4 app with Pinia for the gallery selection state.
The controls in the Components page are the actual Solid and React Argui TSX
gallery, built with the Rust WebAssembly host and rendered on WebGPU. The docs
pages are generated from the current `docs/` Markdown files. Historical plans
and the shadcn reference snapshot are not published as guides.

From the repository root, install the two Bun workspaces and build the gallery:

```sh
bun install --frozen-lockfile
cd website
bun install --frozen-lockfile
bun run gallery:build
bun run dev
```

`gallery:build` needs the `wasm32-unknown-unknown` Rust target and `wasm-pack`.
It generates release assets, compiles the Rust host, builds both TSX adapters,
and copies the complete browser bundles to `website/public/gallery/`. Run it
again after changing Rust, widgets, or gallery TSX. Nuxt's `generate` command
checks that both real bundles exist:

```sh
cd website
bun run typecheck
bun run build      # Nuxt server output for an SSR deployment
bun run generate   # Pre-rendered HTML for static hosting
```

All site content, including documentation and metadata, renders on the server;
the embedded WebGPU canvas starts in the browser. The generated static site is
in `website/.output/public/`. Set
`NUXT_APP_BASE_URL=/argui/` when generating for a subpath such as GitHub Pages;
the gallery uses relative asset URLs inside each adapter bundle. Browser
rendering requires WebGPU. Native window services stay in the desktop gallery.
The GitHub star badge fetches the public repository count in the browser and
refreshes it while the page is open; other page content stays server-rendered.
The social icons use [Tabler Icons](https://github.com/tabler/tabler-icons)
under the [MIT license](../apps/gallery/assets/tabler/LICENSE).

For Linux browser checks, follow `docs/contributing/linux-testing.md` and run
the browser inside `./scripts/linux-hidden-display.sh` from the repository
root. Do not open a test window on the user's display.
