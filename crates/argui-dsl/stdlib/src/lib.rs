//! Embedded, tree-shakeable source modules for the official Argui DSL library.

mod icons;

pub use icons::{icon_component_source, icon_names, icon_svg};

/// Canonical component modules exposed through the `@argui/ui` package import.
pub const UI_MODULES: &[(&str, &str)] = &[
    ("@argui/ui/theme.argui", include_str!("../ui/theme.argui")),
    ("@argui/ui/button.argui", include_str!("../ui/button.argui")),
    ("@argui/ui/input.argui", include_str!("../ui/input.argui")),
    ("@argui/ui/card.argui", include_str!("../ui/card.argui")),
    ("@argui/ui/badge.argui", include_str!("../ui/badge.argui")),
    (
        "@argui/ui/separator.argui",
        include_str!("../ui/separator.argui"),
    ),
    ("@argui/ui/switch.argui", include_str!("../ui/switch.argui")),
    (
        "@argui/ui/virtual-list.argui",
        include_str!("../ui/virtual-list.argui"),
    ),
];
