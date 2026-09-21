//! Pinned Tabler Icons catalog and on-demand DSL component source.

#[cfg(feature = "icons")]
include!(concat!(env!("OUT_DIR"), "/icons.rs"));

/// Iterates the complete pinned Tabler import names, including the `Spinner` alias.
///
/// Returns no names when the `icons` feature is disabled.
pub fn icon_names() -> impl Iterator<Item = &'static str> {
    let names = if cfg!(feature = "icons") {
        include_str!("../icons/catalog.txt")
    } else {
        ""
    };
    names
        .lines()
        .chain(cfg!(feature = "icons").then_some("Spinner"))
}

/// Returns source for one icon component exposed by `@argui/icons`.
///
/// * `name` — exported PascalCase Tabler name, without the `Outline` suffix.
///   `Spinner` is a static alias for Tabler's `Loader2`; callers choose whether
///   to animate its exposed rotation property.
///
/// Returns `None` when the icon is unknown or the `icons` feature is disabled.
#[must_use]
pub fn icon_component_source(name: &str) -> Option<String> {
    let icon = if name == "Spinner" { "Loader2" } else { name };
    icon_svg(icon)?;
    Some(format!(
        "import {{ Svg }} from \"@argui/native\"\n\
         export component {name} {{\n\
             in property size: length = 20px\n\
             in property color: color = var(--argui-foreground)\n\
             in property rotation: float = 0.0\n\
             in property opacity: float = 1.0\n\
             Svg {{ source: asset(\"@argui/icons/{icon}.svg\") width: size height: size color: color rotation: rotation opacity: opacity }}\n\
         }}\n"
    ))
}

/// Returns the complete SVG document for one pinned Tabler icon.
///
/// * `name` — exported PascalCase name, optionally ending in `Filled`.
///   `Spinner` resolves to Tabler `Loader2`.
///
/// Returns `None` for an unknown name or without the `icons` feature. The SVG
/// uses `currentColor` so the generic SVG asset renderer can tint it at use.
#[must_use]
pub fn icon_svg(name: &str) -> Option<String> {
    #[cfg(feature = "icons")]
    {
        let name = if name == "Spinner" { "Loader2" } else { name };
        let icon = icon_data(name)?;
        Some(format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="{}" fill="{}" stroke="{}" stroke-width="{}" stroke-linecap="{}" stroke-linejoin="{}">{}</svg>"##,
            icon.width.unwrap_or("24"),
            icon.height.unwrap_or("24"),
            icon.view_box.unwrap_or("0 0 24 24"),
            icon.fill.unwrap_or("none"),
            icon.stroke.unwrap_or("currentColor"),
            icon.stroke_width.unwrap_or("2"),
            icon.stroke_linecap.unwrap_or("round"),
            icon.stroke_linejoin.unwrap_or("round"),
            icon.data,
        ))
    }
    #[cfg(not(feature = "icons"))]
    {
        let _ = name;
        None
    }
}
