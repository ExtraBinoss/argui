# Argui website

Nuxt 4, Vue, Pinia, Nuxt I18n and `@lucide/vue`. English content, server rendering,
and static generation for the home, features, getting started and component pages.

## Run locally

Use Node 22.12+ and pnpm 11.

```sh
cd website
pnpm install --frozen-lockfile
pnpm gallery:build
pnpm dev
```

`gallery:build` needs `wasm-pack` and the `wasm32-unknown-unknown` Rust target.
It builds the real gallery with all features, including the updater demo, and
at most six Cargo jobs and six WebAssembly optimization workers. If `web/widgets/pkg` is already built, use
`pnpm gallery:copy` instead. The generated binaries are ignored by Git.
The rest of the website can be developed without building WASM.

Component previews start automatically after hydration and open their matching
example through the gallery's accessible navigation. The loading indicator waits
for Argui's `RuntimeEvent::RendererReady`, forwarded by the gallery as the browser
`argui:renderer-state` event (`detail.state`: `ready` or `error`). The runtime
stays independent of the loading UI. Visitors can stop and restart the preview.
Browsers without a usable WebGPU adapter get an explanation and retain access
to the component source. The updater in the browser uses sample states;
native installation belongs to `argui-updater`.

## Build and SEO

Set `NUXT_PUBLIC_SITE_URL` to the final public origin before building. The site
does not assume a domain. Without this value, canonical URLs are omitted and
the sitemap has no URL entries.

```sh
NUXT_PUBLIC_SITE_URL=https://your-domain.example pnpm generate
```

GitHub Actions deploys `main` to <https://extrabinoss.github.io/argui/> after the
website checks and Gitleaks pass. This build sets `NUXT_APP_BASE_URL=/argui/` and
`NUXT_PUBLIC_SITE_URL=https://extrabinoss.github.io/argui`; images, WASM previews,
navigation and SEO URLs all respect this prefix. Local development stays at `/`.

Upload `.output/public` to a static host that serves directory `index.html`
files, a real 404 page, and `.wasm` as `application/wasm`. Serve over HTTPS:
WebGPU requires a secure context (localhost also works). The generated site
includes the gallery bundle, page titles/descriptions, Open Graph metadata,
canonical links, `robots.txt`, and a sitemap for all component URLs.

For a Node server, use `pnpm build` and run `.output/server/index.mjs`. The same
public origin must be supplied when building and running because pages are
prerendered. Nothing is deployed by these commands.

## Content and components

- `app/components/` holds the shared navigation, action links, source links,
  code blocks, feature cards and gallery frame.
- `app/stores/interface.ts` owns component search, mobile component navigation
  and the persisted light/dark preference.
- `i18n/locales/en.json` holds the website copy. Add a locale file and a locale
  in `nuxt.config.ts` when introducing another language. English URLs stay
  unprefixed; additional languages use a prefix.
- `pnpm catalogue` refreshes `app/data/catalogue.json` from the Rust gallery
  navigation and widget documentation. Tests check every source path and public
  widget feature. Component names/descriptions are the English Rust catalogue;
  provide locale-specific catalogue copy when translating those pages.
- The memory number is the measured Linux release gallery's private memory,
  not process RSS or a guarantee for every application. Its source is linked
  next to the number.

## Verify

```sh
pnpm test
pnpm typecheck
pnpm format:check
pnpm generate
pnpm test:generated
```

Browser checks must run on the repository's private Linux display. From the
repository root, with the website running:

```sh
CHROME_PATH=/path/to/chrome \
PUPPETEER_MODULE=/path/to/puppeteer.js \
./scripts/linux-hidden-display.sh node website/tests/browser.mjs
```

`WEBSITE_URL` defaults to `http://127.0.0.1:3100`; start the development server
with `pnpm dev --port 3100` or set it to another local server. Screenshots are
saved to `website/test-results/`. The scenario exercises themes, reloads,
mobile navigation, search, deep links, clipboard, the real WASM Button, the
no-WebGPU fallback, missing pages and hydration warnings. The live gallery keeps
its usable desktop viewport inside a horizontally scrollable frame on small
screens. Inspect the PNGs.
Set `UPDATE_ASSETS=1` to refresh the committed gallery preview and social card
from actual browser captures. No browser is opened on the user's desktop.
