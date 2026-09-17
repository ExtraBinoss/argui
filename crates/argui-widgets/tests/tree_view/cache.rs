use std::collections::BTreeSet;

use argui_core::{Color, ColorScheme};
use argui_paint::VectorId;
use argui_ui::{Element, ElementKind};
use argui_widgets::{TreeNode, TreeView, TreeViewCache, VList, shadcn};

fn row<'a>(element: &'a Element, key: &str) -> Option<&'a Element> {
    if element.key.as_deref() == Some(key) {
        return Some(element);
    }
    element.children.iter().find_map(|child| row(child, key))
}

fn vectors(element: &Element) -> usize {
    usize::from(matches!(element.kind, ElementKind::Vector { .. }))
        + element.children.iter().map(vectors).sum::<usize>()
}

fn opacity(element: &Element) -> f32 {
    element.layer.as_ref().map_or(1.0, |layer| layer.opacity)
}

#[test]
fn scrolling_reuses_rows_and_evicts_rows_outside_the_window() {
    let nodes = (0..200)
        .map(|index| TreeNode {
            key: format!("node-{index}"),
            label: format!("Row {index}"),
            depth: 0,
            icon: Some(VectorId::fresh()),
        })
        .collect::<Vec<_>>();
    let collapsed = BTreeSet::new();
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Dark);
    let mut cache = TreeViewCache::default();
    let mut tree = TreeView::new(
        &nodes,
        None,
        &collapsed,
        VList::new("tree", 28.0, 280.0, 0.0),
    );
    let first = tree.build_cached(theme, &mut cache);
    tree.list.offset = 28.0;
    let second = tree.build_cached(theme, &mut cache);
    let original = row(&first, "node-5").unwrap();
    let reused = row(&second, "node-5").unwrap();
    assert!(original.children[0].ptr_eq(&reused.children[0]));
    assert_eq!(vectors(reused), 1);
    assert!(row(&second, "node-100").is_none());
    tree.list.offset = 2800.0;
    assert!(row(&tree.build_cached(theme, &mut cache), "node-5").is_none());
    tree.list.offset = 0.0;
    let returned = tree.build_cached(theme, &mut cache);
    assert!(!original.children[0].ptr_eq(&row(&returned, "node-5").unwrap().children[0]));
}

#[test]
fn selection_collapse_data_theme_and_icons_invalidate_rows() {
    let mut nodes = vec![
        TreeNode {
            key: "parent".into(),
            label: "Parent".into(),
            depth: 0,
            icon: None,
        },
        TreeNode {
            key: "child".into(),
            label: "Child".into(),
            depth: 1,
            icon: None,
        },
    ];
    let mut collapsed = BTreeSet::new();
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Dark);
    let mut cache = TreeViewCache::default();
    let build = |nodes: &[TreeNode],
                 collapsed: &BTreeSet<String>,
                 selected,
                 icon,
                 theme: &_,
                 cache: &mut _| {
        TreeView::new(
            nodes,
            selected,
            collapsed,
            VList::new("tree", 28.0, 280.0, 0.0),
        )
        .disclosure(icon)
        .build_cached(theme, cache)
    };
    let first = build(&nodes, &collapsed, None, None, theme, &mut cache);
    let selected = build(&nodes, &collapsed, Some("child"), None, theme, &mut cache);
    assert_ne!(
        row(&first, "child").unwrap().paint,
        row(&selected, "child").unwrap().paint
    );
    collapsed.insert("parent".into());
    assert!(
        row(
            &build(&nodes, &collapsed, None, None, theme, &mut cache),
            "child"
        )
        .is_none()
    );
    collapsed.clear();
    nodes[1].label = "Renamed".into();
    let renamed = build(&nodes, &collapsed, None, None, theme, &mut cache);
    assert!(
        !row(&first, "child").unwrap().children[0]
            .ptr_eq(&row(&renamed, "child").unwrap().children[0])
    );
    let light = themes.resolve(ColorScheme::Light);
    let changed = build(
        &nodes,
        &collapsed,
        None,
        Some(VectorId::fresh()),
        light,
        &mut cache,
    );
    assert_eq!(vectors(&changed), 1);
    assert!(
        !row(&renamed, "parent").unwrap().children[0]
            .ptr_eq(&row(&changed, "parent").unwrap().children[0])
    );
    assert!(
        build(&[], &collapsed, None, None, theme, &mut cache)
            .children
            .len()
            <= 2
    );
}

#[test]
fn descendant_reveal_leaves_the_trigger_and_siblings_still() {
    let nodes = [
        TreeNode {
            key: "parent".into(),
            label: "Parent".into(),
            depth: 0,
            icon: None,
        },
        TreeNode {
            key: "child".into(),
            label: "Child".into(),
            depth: 1,
            icon: None,
        },
        TreeNode {
            key: "nested".into(),
            label: "Nested".into(),
            depth: 2,
            icon: None,
        },
        TreeNode {
            key: "sibling".into(),
            label: "Sibling".into(),
            depth: 0,
            icon: None,
        },
    ];
    let collapsed = BTreeSet::new();
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Light);
    let mut cache = TreeViewCache::default();
    let build = |progress, cache: &mut TreeViewCache| {
        TreeView::new(
            &nodes,
            None,
            &collapsed,
            VList::new("tree", 28.0, 280.0, 0.0),
        )
        .reveal_descendants("parent", progress)
        .build_cached(theme, cache)
    };

    let entering = build(0.0, &mut cache);
    assert_eq!(opacity(row(&entering, "parent").unwrap()), 1.0);
    assert_eq!(opacity(row(&entering, "sibling").unwrap()), 1.0);
    assert_eq!(opacity(row(&entering, "child").unwrap()), 0.0);
    assert_ne!(
        row(&entering, "child").unwrap().transform,
        argui_core::Transform2D::IDENTITY
    );

    let sweeping = build(0.45, &mut cache);
    assert!(opacity(row(&sweeping, "child").unwrap()) > opacity(row(&sweeping, "nested").unwrap()));

    let settled = build(1.0, &mut cache);
    assert_eq!(opacity(row(&settled, "child").unwrap()), 1.0);
    assert_eq!(
        row(&settled, "nested").unwrap().transform,
        argui_core::Transform2D::IDENTITY
    );
    assert!(
        row(&entering, "child").unwrap().children[0]
            .ptr_eq(&row(&settled, "child").unwrap().children[0])
    );
}

#[test]
fn active_row_stays_mounted_and_is_the_only_tab_stop() {
    let nodes: Vec<_> = (0..1000)
        .map(|i| TreeNode {
            key: i.to_string(),
            label: i.to_string(),
            depth: 0,
            icon: None,
        })
        .collect();
    let collapsed = BTreeSet::new();
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Light);
    let mut cache = TreeViewCache::default();
    for selected in [Some("999"), Some("500"), None, Some("deleted")] {
        let tree = TreeView::new(
            &nodes,
            selected,
            &collapsed,
            VList::new("tree", 28.0, 280.0, 5600.0),
        );
        for built in [tree.build(theme), tree.build_cached(theme, &mut cache)] {
            let ui = argui_ui::UiTree::new(built);
            assert!(ui.node_ids().len() < 250);
            let stops: Vec<_> = (0..ui.node_ids().len())
                .filter_map(|i| ui.element_at(i))
                .filter(|element| {
                    element
                        .interaction
                        .as_ref()
                        .is_some_and(|i| i.focus_policy == argui_ui::FocusPolicy::TabStop)
                })
                .map(|element| element.key.as_deref().unwrap())
                .collect();
            assert_eq!(
                stops,
                vec![selected.filter(|id| *id != "deleted").unwrap_or("0")]
            );
        }
    }
}

#[test]
fn focus_survives_virtual_scroll_and_tab_leaves_the_tree_once() {
    use argui_core::{Key, KeyInput, KeyState, Size};
    use argui_layout::LayoutEngine;
    use argui_text::TextEngine;
    use argui_ui::{FocusRequest, UiTree};
    let nodes: Vec<_> = (0..100)
        .map(|i| TreeNode {
            key: i.to_string(),
            label: i.to_string(),
            depth: 0,
            icon: None,
        })
        .collect();
    let collapsed = BTreeSet::new();
    let themes = shadcn(Color::WHITE);
    let theme = themes.resolve(ColorScheme::Light);
    let mut cache = TreeViewCache::default();
    let mut view = TreeView::new(
        &nodes,
        Some("50"),
        &collapsed,
        VList::new("tree", 28.0, 140.0, 1400.0),
    );
    let build = |view: &TreeView<'_>, cache: &mut TreeViewCache| {
        Element::column([
            view.build_cached(theme, cache),
            argui_widgets::Button::new("next", "Next", theme.button()).build(),
        ])
    };
    let mut ui = UiTree::new(build(&view, &mut cache));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let output = engine
        .compute(&mut ui, &mut text, Size::new(300.0, 300.0))
        .unwrap();
    ui.sync_focus(&output.hit_regions, Some(FocusRequest::Focus("50".into())));
    let focused = ui.focused_node().unwrap();
    view.list.offset = 0.0;
    ui.update(build(&view, &mut cache));
    let output = engine
        .compute(&mut ui, &mut text, Size::new(300.0, 300.0))
        .unwrap();
    ui.sync_focus(&output.hit_regions, None);
    assert_eq!(ui.focused_node(), Some(focused));
    ui.key_input(
        &KeyInput {
            key: Key::Tab,
            state: KeyState::Pressed,
            modifiers: Default::default(),
            repeat: false,
            text: None,
        },
        &output.hit_regions,
    );
    assert_eq!(ui.key(ui.focused_node().unwrap()), Some("next"));
}
