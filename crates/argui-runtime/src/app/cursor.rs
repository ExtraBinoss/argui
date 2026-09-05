use argui_core::Point;
use argui_ui::{CursorIcon, HitRegion, scrollbar_at};
use winit::{window::CursorIcon as WinitCursorIcon, window::Window};

use super::Application;

impl Application {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn refresh_cursor(&mut self, window: &Window) {
        let captured = self.ui_tree.as_ref().zip(self.ui_layout.as_ref())
            .and_then(|(tree, layout)| tree.captured_cursor(argui_core::PointerId::MOUSE, &layout.hit_regions));
        let cursor = captured.unwrap_or_else(|| self.pointer.map_or(CursorIcon::Default, |point| {
            let Some(layout) = &self.ui_layout else {
                return CursorIcon::Default;
            };
            if scrollbar_at(point, &layout.scroll_regions, &layout.hit_regions).is_some() {
                CursorIcon::Default
            } else if super::text_selection::text_region_at(&layout.text_regions, point) {
                CursorIcon::Text
            } else {
                cursor_at(&layout.hit_regions, point)
            }
        }));
        if cursor != self.last_cursor {
            window.set_cursor(to_winit(cursor));
            self.last_cursor = cursor;
        }
    }
}

fn cursor_at(regions: &[HitRegion], point: Point) -> CursorIcon {
    match regions.iter().rev().find(|region| region.contains(point)) {
        Some(HitRegion {
            cursor: CursorIcon::Auto,
            ..
        })
        | None => CursorIcon::Default,
        Some(region) => region.cursor,
    }
}

fn to_winit(cursor: CursorIcon) -> WinitCursorIcon {
    match cursor {
        CursorIcon::Auto | CursorIcon::Default => WinitCursorIcon::Default,
        CursorIcon::ContextMenu => WinitCursorIcon::ContextMenu,
        CursorIcon::Help => WinitCursorIcon::Help,
        CursorIcon::Pointer => WinitCursorIcon::Pointer,
        CursorIcon::Progress => WinitCursorIcon::Progress,
        CursorIcon::Wait => WinitCursorIcon::Wait,
        CursorIcon::Cell => WinitCursorIcon::Cell,
        CursorIcon::Crosshair => WinitCursorIcon::Crosshair,
        CursorIcon::Text => WinitCursorIcon::Text,
        CursorIcon::VerticalText => WinitCursorIcon::VerticalText,
        CursorIcon::Alias => WinitCursorIcon::Alias,
        CursorIcon::Copy => WinitCursorIcon::Copy,
        CursorIcon::Move => WinitCursorIcon::Move,
        CursorIcon::NoDrop => WinitCursorIcon::NoDrop,
        CursorIcon::NotAllowed => WinitCursorIcon::NotAllowed,
        CursorIcon::Grab => WinitCursorIcon::Grab,
        CursorIcon::Grabbing => WinitCursorIcon::Grabbing,
        CursorIcon::EResize => WinitCursorIcon::EResize,
        CursorIcon::NResize => WinitCursorIcon::NResize,
        CursorIcon::NeResize => WinitCursorIcon::NeResize,
        CursorIcon::NwResize => WinitCursorIcon::NwResize,
        CursorIcon::SResize => WinitCursorIcon::SResize,
        CursorIcon::SeResize => WinitCursorIcon::SeResize,
        CursorIcon::SwResize => WinitCursorIcon::SwResize,
        CursorIcon::WResize => WinitCursorIcon::WResize,
        CursorIcon::EwResize => WinitCursorIcon::EwResize,
        CursorIcon::NsResize => WinitCursorIcon::NsResize,
        CursorIcon::NeswResize => WinitCursorIcon::NeswResize,
        CursorIcon::NwseResize => WinitCursorIcon::NwseResize,
        CursorIcon::ColResize => WinitCursorIcon::ColResize,
        CursorIcon::RowResize => WinitCursorIcon::RowResize,
        CursorIcon::AllScroll => WinitCursorIcon::AllScroll,
        CursorIcon::ZoomIn => WinitCursorIcon::ZoomIn,
        CursorIcon::ZoomOut => WinitCursorIcon::ZoomOut,
        CursorIcon::DndAsk => WinitCursorIcon::DndAsk,
        CursorIcon::AllResize => WinitCursorIcon::AllResize,
    }
}
