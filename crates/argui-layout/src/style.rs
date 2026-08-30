use argui_ui::{LayoutStyle, Overflow, ScrollbarGutter};
use taffy::{Style, geometry::Point};

pub(crate) fn taffy_style(style: &LayoutStyle) -> Style {
    Style {
        display: style.display,
        box_sizing: style.box_sizing,
        direction: style.writing_direction,
        overflow: Point {
            x: overflow(style.overflow.x),
            y: overflow(style.overflow.y),
        },
        scrollbar_width: match style.scrollbar_gutter {
            ScrollbarGutter::Auto => 0.0,
            ScrollbarGutter::Stable => style.scrollbar_width.max(0.0),
        },
        position: style.position,
        inset: style.inset,
        size: style.size,
        min_size: style.min_size,
        max_size: style.max_size,
        aspect_ratio: style.aspect_ratio,
        margin: style.margin,
        padding: style.padding,
        border: style.border,
        align_items: style.align_items,
        align_self: style.align_self,
        justify_items: style.justify_items,
        justify_self: style.justify_self,
        align_content: style.align_content,
        justify_content: style.justify_content,
        gap: style.gap,
        flex_direction: style.flex_direction,
        flex_wrap: style.flex_wrap,
        flex_basis: style.flex_basis,
        flex_grow: style.flex_grow,
        flex_shrink: style.flex_shrink,
        grid_template_rows: style.grid_template_rows.clone(),
        grid_template_columns: style.grid_template_columns.clone(),
        grid_auto_rows: style.grid_auto_rows.clone(),
        grid_auto_columns: style.grid_auto_columns.clone(),
        grid_auto_flow: style.grid_auto_flow,
        grid_template_areas: style.grid_template_areas.clone(),
        grid_template_column_names: style.grid_template_column_names.clone(),
        grid_template_row_names: style.grid_template_row_names.clone(),
        grid_row: style.grid_row.clone(),
        grid_column: style.grid_column.clone(),
        ..Style::default()
    }
}

const fn overflow(value: Overflow) -> taffy::Overflow {
    match value {
        Overflow::Visible => taffy::Overflow::Visible,
        Overflow::Clip => taffy::Overflow::Clip,
        Overflow::Hidden => taffy::Overflow::Hidden,
        Overflow::Auto | Overflow::Scroll => taffy::Overflow::Scroll,
    }
}
