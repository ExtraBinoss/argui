use argui_core::Point;
use argui_ui::{CursorIcon, HitRegion, scrollbar_at};
use winit::{window::CursorIcon as WinitCursorIcon, window::Window};

use super::Application;

impl Application {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn refresh_cursor(&mut self, window: &Window) {
        let cursor = self.pointer.map_or(CursorIcon::Default, |point| {
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
        });
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

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use argui_core::{Affine2D, Point, Rect, Size};
    use argui_paint::ClipChain;
    use argui_ui::{CursorIcon, HitRegion};

    use super::{cursor_at, to_winit};

    fn region(cursor: CursorIcon) -> HitRegion {
        HitRegion {
            node: argui_ui::UiTree::new(argui_ui::Element::container([]))
                .node_id_at(0)
                .unwrap(),
            bounds: Rect::new(Point::default(), Size::new(40.0, 40.0)),
            transform: Affine2D::IDENTITY,
            clips: ClipChain::default(),
            shape: argui_ui::HitShape::Bounds,
            slop: argui_ui::HitTestStyle::default().slop,
            enabled: true,
            focusable: false,
            cursor,
            gestures: argui_ui::GestureSet::NONE,
            window_drag: None,
        }
    }

    #[test]
    fn topmost_hit_region_controls_the_cursor() {
        let regions = [region(CursorIcon::Pointer), region(CursorIcon::Text)];
        assert_eq!(
            cursor_at(&regions, Point::new(20.0, 20.0)),
            CursorIcon::Text
        );
        assert_eq!(
            cursor_at(&regions, Point::new(50.0, 50.0)),
            CursorIcon::Default
        );
        assert_eq!(
            cursor_at(&[region(CursorIcon::Auto)], Point::new(20.0, 20.0)),
            CursorIcon::Default
        );

        let mut disabled = region(CursorIcon::NotAllowed);
        disabled.enabled = false;
        assert_eq!(
            cursor_at(&[disabled], Point::new(20.0, 20.0)),
            CursorIcon::NotAllowed
        );
    }

    #[test]
    fn every_public_cursor_maps_to_winit() {
        let cursors = [
            CursorIcon::Auto,
            CursorIcon::Default,
            CursorIcon::ContextMenu,
            CursorIcon::Help,
            CursorIcon::Pointer,
            CursorIcon::Progress,
            CursorIcon::Wait,
            CursorIcon::Cell,
            CursorIcon::Crosshair,
            CursorIcon::Text,
            CursorIcon::VerticalText,
            CursorIcon::Alias,
            CursorIcon::Copy,
            CursorIcon::Move,
            CursorIcon::NoDrop,
            CursorIcon::NotAllowed,
            CursorIcon::Grab,
            CursorIcon::Grabbing,
            CursorIcon::EResize,
            CursorIcon::NResize,
            CursorIcon::NeResize,
            CursorIcon::NwResize,
            CursorIcon::SResize,
            CursorIcon::SeResize,
            CursorIcon::SwResize,
            CursorIcon::WResize,
            CursorIcon::EwResize,
            CursorIcon::NsResize,
            CursorIcon::NeswResize,
            CursorIcon::NwseResize,
            CursorIcon::ColResize,
            CursorIcon::RowResize,
            CursorIcon::AllScroll,
            CursorIcon::ZoomIn,
            CursorIcon::ZoomOut,
            CursorIcon::DndAsk,
            CursorIcon::AllResize,
        ];
        for cursor in cursors {
            black_box(to_winit(black_box(cursor)));
        }
    }
}
