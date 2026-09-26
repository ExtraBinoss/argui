# Gallery media sources

- `tabler/*.svg`: source SVGs from [Tabler Icons](https://github.com/tabler/tabler-icons), MIT license in [`tabler/LICENSE`](tabler/LICENSE). They are imported as vector assets; no DOM icon component is used.
- `gallery/*.svg`: original icons owned by this application for its navigation.
- `illustration/orbit.svg`: original multicolor illustration source.
- `illustration/orbit.png`: 960 × 640 raster prepared from `orbit.svg` for the Media page; the source SVG remains available. Regenerate with `magick -background none -density 384 orbit.svg -resize 960x640 -depth 8 -define png:color-type=6 orbit.png` from `illustration/`.
- `photo/saturn.jpg`: copied from the repository's existing `crates/argui/examples/assets/astra-orig.jpg` example to verify native raster decoding.

`bun apps/gallery/scripts/generate-assets.mjs` reads `assets.config.json`. Configured packs and individual files are available in development. The script follows local imports from both TSX entrypoints and packages only literal `mediaAssets['key.svg']` references for release. `assets.generated.json` records the selected keys and byte total; `assets.dev.generated.json` records all configured sources. Native release embeds only the selected bytes. Dynamic keys fail generation with a source location.
