# Application icons and media

Argui's SVG and image primitives accept an application asset reference. Widgets accept JSX or media in their icon slots; they do not choose an icon pack. The application owns the files and decides which pack to use.

## Gallery

The gallery keeps a small Tabler pack and its own navigation icons under `apps/gallery/assets/`. `assets.config.json` names directories as packs and can map individual files to keys. For example:

```json
{
  "packs": { "tabler": "tabler", "my-icons": "my-icons" },
  "files": { "brand/logo.svg": "brand/logo.svg" },
  "entries": ["src/solid/main.tsx", "src/react/main.tsx"]
}
```

Reference a file with a literal key in TSX:

```tsx
import { mediaAssets } from '../../assets.generated'

<svg source={mediaAssets['my-icons/spark.svg']} width={18} height={18} />
```

Import `mediaAssets` directly from `assets.generated` in a module that uses it.
Re-exporting the whole map through another module is rejected because it hides
the references needed to select release files.

Run `bun apps/gallery/scripts/generate-assets.mjs` after changing files or imports. The generator follows local imports from the configured entries. Native debug builds can decode all configured assets, while native release builds embed only keys reached by literal `mediaAssets['…']` references. `assets.generated.json` lists release keys, stable IDs, source paths, and byte counts; `assets.dev.generated.json` lists the full development set. A dynamic lookup such as `mediaAssets[name]` fails with a location because its release files cannot be determined.

## Apps created by `argui init`

A generated TSX app contains the same `assets.config.json` and `scripts/generate-assets.mjs`. Put SVG, PNG, JPEG, or WebP files below `assets/`; map a whole pack under `packs` or one file under `files`. Add literal references from code imported by `src/main.tsx`. `argui check` and `argui build` regenerate the manifests and TypeScript references automatically.

The native debug runner loads `assets.dev.generated.json`. A release package copies only the source files named in `assets.generated.json` into `dist/desktop/assets/`. Its launcher points the host at the adjacent manifest, so the directory can move as one unit. No frame executes JavaScript to resolve or decode icons: the host loads them before the first frame.

The Web mount fetches the same generated manifest and encoded files before starting its WASM renderer. Development uses the complete pack manifest; `argui build release --target web` copies only referenced sources into `dist/web/assets/`. URLs resolve against the document base, so a moved release directory also works under a subpath. The renderer decodes each source once during startup.
