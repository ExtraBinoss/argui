//! Embedded, tree-shakeable source modules for the official Argui DSL library.

mod icons;

pub use icons::{icon_component_source, icon_names, icon_svg};

/// Canonical component modules exposed through the `@argui/ui` package import.
pub const UI_MODULES: &[(&str, &str)] = &[
    ("@argui/ui/theme.argui", include_str!("../ui/theme.argui")),
    ("@argui/ui/button.argui", include_str!("../ui/button.argui")),
    (
        "@argui/ui/accordion.argui",
        include_str!("../ui/accordion.argui"),
    ),
    ("@argui/ui/alert.argui", include_str!("../ui/alert.argui")),
    (
        "@argui/ui/attachment.argui",
        include_str!("../ui/attachment.argui"),
    ),
    ("@argui/ui/bubble.argui", include_str!("../ui/bubble.argui")),
    (
        "@argui/ui/button-group.argui",
        include_str!("../ui/button-group.argui"),
    ),
    (
        "@argui/ui/calendar.argui",
        include_str!("../ui/calendar.argui"),
    ),
    ("@argui/ui/drawer.argui", include_str!("../ui/drawer.argui")),
    ("@argui/ui/field.argui", include_str!("../ui/field.argui")),
    (
        "@argui/ui/file-picker.argui",
        include_str!("../ui/file-picker.argui"),
    ),
    (
        "@argui/ui/hover-card.argui",
        include_str!("../ui/hover-card.argui"),
    ),
    (
        "@argui/ui/input-group.argui",
        include_str!("../ui/input-group.argui"),
    ),
    (
        "@argui/ui/input-otp.argui",
        include_str!("../ui/input-otp.argui"),
    ),
    ("@argui/ui/marker.argui", include_str!("../ui/marker.argui")),
    (
        "@argui/ui/context-menu.argui",
        include_str!("../ui/context-menu.argui"),
    ),
    (
        "@argui/ui/data-table.argui",
        include_str!("../ui/data-table.argui"),
    ),
    (
        "@argui/ui/data-table-resize-handle.argui",
        include_str!("../ui/data-table-resize-handle.argui"),
    ),
    (
        "@argui/ui/message.argui",
        include_str!("../ui/message.argui"),
    ),
    ("@argui/ui/kbd.argui", include_str!("../ui/kbd.argui")),
    ("@argui/ui/label.argui", include_str!("../ui/label.argui")),
    (
        "@argui/ui/skeleton.argui",
        include_str!("../ui/skeleton.argui"),
    ),
    ("@argui/ui/toggle.argui", include_str!("../ui/toggle.argui")),
    (
        "@argui/ui/tooltip.argui",
        include_str!("../ui/tooltip.argui"),
    ),
    ("@argui/ui/chart.argui", include_str!("../ui/chart.argui")),
    (
        "@argui/ui/color-picker.argui",
        include_str!("../ui/color-picker.argui"),
    ),
    (
        "@argui/ui/pagination.argui",
        include_str!("../ui/pagination.argui"),
    ),
    (
        "@argui/ui/list-row.argui",
        include_str!("../ui/list-row.argui"),
    ),
    ("@argui/ui/toast.argui", include_str!("../ui/toast.argui")),
    (
        "@argui/ui/reorder-item.argui",
        include_str!("../ui/reorder-item.argui"),
    ),
    (
        "@argui/ui/animated-text.argui",
        include_str!("../ui/animated-text.argui"),
    ),
    (
        "@argui/ui/split-pane.argui",
        include_str!("../ui/split-pane.argui"),
    ),
    (
        "@argui/ui/checkbox.argui",
        include_str!("../ui/checkbox.argui"),
    ),
    (
        "@argui/ui/radio-group.argui",
        include_str!("../ui/radio-group.argui"),
    ),
    ("@argui/ui/tabs.argui", include_str!("../ui/tabs.argui")),
    ("@argui/ui/avatar.argui", include_str!("../ui/avatar.argui")),
    (
        "@argui/ui/breadcrumb.argui",
        include_str!("../ui/breadcrumb.argui"),
    ),
    (
        "@argui/ui/progress.argui",
        include_str!("../ui/progress.argui"),
    ),
    ("@argui/ui/slider.argui", include_str!("../ui/slider.argui")),
    (
        "@argui/ui/fancy-slider.argui",
        include_str!("../ui/fancy-slider.argui"),
    ),
    (
        "@argui/ui/sidebar-nav-item.argui",
        include_str!("../ui/sidebar-nav-item.argui"),
    ),
    ("@argui/ui/input.argui", include_str!("../ui/input.argui")),
    (
        "@argui/ui/text-area.argui",
        include_str!("../ui/text-area.argui"),
    ),
    (
        "@argui/ui/popover.argui",
        include_str!("../ui/popover.argui"),
    ),
    ("@argui/ui/dialog.argui", include_str!("../ui/dialog.argui")),
    ("@argui/ui/menu.argui", include_str!("../ui/menu.argui")),
    (
        "@argui/ui/menu-bar.argui",
        include_str!("../ui/menu-bar.argui"),
    ),
    (
        "@argui/ui/sub-menu.argui",
        include_str!("../ui/sub-menu.argui"),
    ),
    (
        "@argui/ui/menu-item.argui",
        include_str!("../ui/menu-item.argui"),
    ),
    ("@argui/ui/select.argui", include_str!("../ui/select.argui")),
    (
        "@argui/ui/combobox.argui",
        include_str!("../ui/combobox.argui"),
    ),
    (
        "@argui/ui/scroll-view.argui",
        include_str!("../ui/scroll-view.argui"),
    ),
    ("@argui/ui/card.argui", include_str!("../ui/card.argui")),
    ("@argui/ui/badge.argui", include_str!("../ui/badge.argui")),
    (
        "@argui/ui/separator.argui",
        include_str!("../ui/separator.argui"),
    ),
    ("@argui/ui/switch.argui", include_str!("../ui/switch.argui")),
    (
        "@argui/ui/list-view.argui",
        include_str!("../ui/list-view.argui"),
    ),
];
