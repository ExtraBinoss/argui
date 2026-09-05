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
    let mut tree = TreeView {
        nodes: &nodes,
        selected: None,
        collapsed: &collapsed,
        list: VList::new("tree", 28.0, 280.0, 0.0),
        disclosure: None,
    };
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
        TreeView {
            nodes,
            collapsed,
            selected,
            disclosure: icon,
            list: VList::new("tree", 28.0, 280.0, 0.0),
        }
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
