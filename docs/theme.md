# Themes

`argui-theme` contains no global singleton. `Theme<T>` owns typed light and dark
values and resolves them with `ThemeMode::Light`, `Dark`, or `System`. Any
application data can be themed; `WidgetTheme` is only the standard preset pack
for buttons and text editors.

Theme colors are authored in sRGB and stored internally as linear sRGB. The
standard `shadcn` theme uses exact Zinc light/dark tokens and derives hover,
pressed, scrollbar, and focus colors in OKLab. See [Color](color.md) for the
renderer-wide contract.

The runtime exposes `WindowEnvironment` through `Context::environment()`. It
contains the effective color scheme, reduced-motion preference, and
high-contrast preference. Components that read it are retained and rebuilt when
the environment changes; unrelated component caches remain valid.

System mode uses Winit theme notifications on Windows, macOS, and Web. Linux
reads and continuously watches the XDG desktop portal. An unknown or explicitly
neutral system preference resolves to Light. `PreferenceOverrides` always wins
and is applied before the first visible frame.

Standard widget icons use SVG `currentColor`. Their alpha masks are cached in the
shared vector atlas while `Element::vector_color` supplies the resolved theme
color per instance. Theme changes therefore repaint existing vectors without
duplicating or rerasterizing their assets. Applications register the assets
through `Render::vector_assets` and may replace every preset or asset.
