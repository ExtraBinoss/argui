use argui_core::{Color, ColorScheme};
use argui_paint::{BorderWidths, CornerRadii};
use argui_ui::{AlignSelf, Element, Orientation, Role, UiTree, WritingDirection};
use argui_widgets::{Button, ButtonGroup, ButtonGroupSeparator, ButtonGroupText, shadcn};

#[test]
fn groups_preserve_independent_button_targets_and_names_in_both_axes() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Light);
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let mut group = ButtonGroup::new(
            "actions",
            "Actions",
            [
                Button::new("save", "Save", theme.button()).build(),
                Button::new("cancel", "Cancel", theme.outline_button()).build(),
            ],
        );
        group.orientation = orientation;
        let tree = UiTree::new(group.build());
        let semantic = tree.semantic_tree(&[], 1.0);
        assert_eq!(
            semantic
                .nodes
                .iter()
                .filter(|node| node.semantics.role == Role::Button)
                .count(),
            2
        );
        assert!(
            semantic
                .nodes
                .iter()
                .any(|node| node.semantics.label.as_deref() == Some("Actions"))
        );
    }
}

#[test]
fn adjacent_controls_share_borders_and_only_outer_corners_stay_rounded() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Light);
    let controls = || {
        ["one", "two", "three"].map(|label| {
            Button::new(label, label, theme.outline_button())
                .without_tooltip()
                .build()
        })
    };

    let horizontal = ButtonGroup::new("horizontal", "Horizontal", controls()).build();
    assert_eq!(horizontal.style.align_self, Some(AlignSelf::START));
    assert_eq!(
        horizontal.children[0].paint.quad.radii,
        CornerRadii {
            top_left: 7.0,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: 7.0,
        }
    );
    assert_eq!(
        horizontal.children[1].paint.quad.radii,
        CornerRadii::all(0.0)
    );
    assert_eq!(
        horizontal.children[2].paint.quad.radii,
        CornerRadii {
            top_left: 0.0,
            top_right: 7.0,
            bottom_right: 7.0,
            bottom_left: 0.0,
        }
    );
    assert_eq!(
        horizontal.children[1]
            .paint
            .quad
            .border
            .as_ref()
            .unwrap()
            .widths,
        BorderWidths {
            left: 0.0,
            right: 1.0,
            top: 1.0,
            bottom: 1.0,
        }
    );

    let vertical = ButtonGroup::new("vertical", "Vertical", controls())
        .orientation(Orientation::Vertical)
        .build();
    assert_eq!(vertical.children[1].paint.quad.radii, CornerRadii::all(0.0));
    assert_eq!(
        vertical.children[1]
            .paint
            .quad
            .border
            .as_ref()
            .unwrap()
            .widths,
        BorderWidths {
            left: 1.0,
            right: 1.0,
            top: 0.0,
            bottom: 1.0,
        }
    );
}

#[test]
fn rtl_groups_join_the_physical_edges_selected_by_visual_order() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Light);
    let group = ButtonGroup::new(
        "rtl-actions",
        "RTL actions",
        ["one", "two", "three"].map(|label| {
            Button::new(label, label, theme.outline_button())
                .without_tooltip()
                .build()
        }),
    )
    .rtl(true)
    .build();

    assert_eq!(group.style.writing_direction, WritingDirection::Rtl);
    assert_eq!(
        group.children[0].paint.quad.radii,
        CornerRadii {
            top_left: 0.0,
            top_right: 7.0,
            bottom_right: 7.0,
            bottom_left: 0.0,
        }
    );
    assert_eq!(group.children[1].paint.quad.radii, CornerRadii::all(0.0));
    assert_eq!(
        group.children[2].paint.quad.radii,
        CornerRadii {
            top_left: 7.0,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: 7.0,
        }
    );
    assert_eq!(
        group.children[1].paint.quad.border.as_ref().unwrap().widths,
        BorderWidths {
            left: 1.0,
            right: 0.0,
            top: 1.0,
            bottom: 1.0,
        }
    );
}

#[test]
fn joined_edges_reach_control_surfaces_inside_overlay_wrappers() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Dark);
    let wrapped = Element::container([Button::new("menu", "Menu", theme.outline_button()).build()]);
    let group = ButtonGroup::new(
        "split",
        "Split",
        [
            Button::new("action", "Action", theme.outline_button()).build(),
            wrapped,
        ],
    )
    .build();
    let trigger = &group.children[1].children[0];

    assert_eq!(
        trigger.paint.quad.radii,
        CornerRadii {
            top_left: 0.0,
            top_right: 7.0,
            bottom_right: 7.0,
            bottom_left: 0.0,
        }
    );
    assert_eq!(
        trigger.paint.quad.border.as_ref().unwrap().widths,
        BorderWidths {
            left: 0.0,
            right: 1.0,
            top: 1.0,
            bottom: 1.0,
        }
    );
}

#[test]
fn separator_and_text_segments_compose_without_stealing_button_semantics() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Light);
    let group = ButtonGroup::new(
        "clipboard",
        "Clipboard",
        [
            ButtonGroupText::new("mode", "Edit").build(theme),
            Button::new("copy", "Copy", theme.secondary_button()).build(),
            ButtonGroupSeparator::new("divider").build(theme),
            Button::new("paste", "Paste", theme.secondary_button()).build(),
        ],
    )
    .build();
    let tree = UiTree::new(group);
    let semantic = tree.semantic_tree(&[], 1.0);

    assert_eq!(
        semantic
            .nodes
            .iter()
            .filter(|node| node.semantics.role == Role::Button)
            .count(),
        2
    );
    assert!(
        semantic
            .nodes
            .iter()
            .any(|node| node.semantics.label.as_deref() == Some("Clipboard"))
    );
}
