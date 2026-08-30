# Themes

`argui-theme` contains no global singleton. `Theme<T>` owns typed light and dark
values and resolves them with `ThemeMode::Light`, `Dark`, or `System`. Any
application data can be themed; `WidgetTheme` is only the standard preset pack
for buttons and text editors.

The runtime exposes `WindowEnvironment` through `Context::environment()`. It
contains the effective color scheme, reduced-motion preference, and
high-contrast preference. Components that read it are retained and rebuilt when
the environment changes; unrelated component caches remain valid.

System mode uses Winit theme notifications on Windows, macOS, and Web. Linux
reads and continuously watches the XDG desktop portal. An unknown or explicitly
neutral system preference resolves to Light. `PreferenceOverrides` always wins
and is applied before the first visible frame.

Vector colors are baked into tessellated assets. `WidgetAssets::embedded`
therefore creates a palette-specific standard asset pack rather than pretending
that vectors can be tinted at paint time. Applications register the returned
assets through `Render::vector_assets` and may replace every preset or asset.
