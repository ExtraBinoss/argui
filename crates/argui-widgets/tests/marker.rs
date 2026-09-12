use argui_core::{Color, ColorScheme};
use argui_ui::{Element, LiveRegion, UiTree};
use argui_widgets::{Marker, MarkerVariant, shadcn};

#[test]
fn status_markers_announce_the_note_once_and_hide_decorative_icons() {
    for variant in [
        MarkerVariant::Inline,
        MarkerVariant::Border,
        MarkerVariant::Separator,
    ] {
        for live in [LiveRegion::Off, LiveRegion::Polite] {
            let mut marker = Marker::new("status", "Synced");
            marker.variant = variant;
            marker.live = live;
            marker.icon = Some(Element::text("check"));
            let tree = UiTree::new(marker.build(shadcn(Color::BLACK).resolve(ColorScheme::Light)));
            let semantic = tree.semantic_tree(&[], 1.0);
            assert_eq!(
                semantic
                    .nodes
                    .iter()
                    .filter(|node| node.semantics.label.as_deref() == Some("Synced"))
                    .count(),
                1
            );
            assert_eq!(
                semantic
                    .nodes
                    .iter()
                    .filter(|node| node.semantics.live == LiveRegion::Polite)
                    .count(),
                usize::from(live == LiveRegion::Polite)
            );
        }
    }
}
