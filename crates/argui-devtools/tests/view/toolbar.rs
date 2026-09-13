use super::*;
use argui_core::{Color, ColorScheme};
use argui_runtime::WindowEnvironment;
use argui_ui::ElementKind;

fn find<'a>(root: &'a Element, key: &str) -> &'a Element {
    fn search<'a>(root: &'a Element, key: &str) -> Option<&'a Element> {
        if root.key.as_deref() == Some(key) {
            return Some(root);
        }
        root.children.iter().find_map(|child| search(child, key))
    }
    search(root, key).unwrap()
}

fn vector_colors(root: &Element) -> Vec<Color> {
    let mut colors = Vec::new();
    if let ElementKind::Vector { color, .. } = root.kind {
        colors.push(color);
    }
    colors.extend(root.children.iter().flat_map(vector_colors));
    colors
}

#[test]
fn toolbar_icons_follow_light_dark_and_picker_selection_colors() {
    for color_scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let host =
            argui_runtime::Entity::new(DevtoolsHost::new(StateShowcase::default()).open(true))
                .mount()
                .unwrap();
        let environment = WindowEnvironment {
            color_scheme,
            ..Default::default()
        };
        let themes = argui_widgets::shadcn(&environment);
        let theme = themes.resolve(color_scheme);
        let root = host.render(environment.clone()).unwrap();
        assert_eq!(
            vector_colors(find(&root, "__devtools-picker")),
            vec![theme.foreground]
        );
        assert_eq!(
            vector_colors(find(&root, "__devtools-dock")),
            vec![theme.foreground]
        );
        let search = UiTree::new(find(&root, "__devtools-search").clone());
        let placeholder = search
            .node_ids()
            .iter()
            .enumerate()
            .find_map(|(index, _)| match &search.element_at(index)?.kind {
                ElementKind::TextEditor {
                    placeholder_text, ..
                } => Some(placeholder_text.color),
                _ => None,
            })
            .unwrap();
        assert_eq!(placeholder, theme.muted_foreground);
        assert!(placeholder.contrast_ratio(theme.card) >= 4.5);
        let close = find(find(&root, "__devtools-panel"), "__devtools-toggle");
        let close_tree = UiTree::new(close.clone());
        let label_color = close_tree
            .node_ids()
            .iter()
            .enumerate()
            .find_map(|(index, _)| match &close_tree.element_at(index)?.kind {
                ElementKind::Text { content, style } if content.as_str() == "×" => {
                    Some(style.color)
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(label_color, theme.foreground);
        host.update(|tools, cx| {
            cx.notify();
            tools.update(&event("__devtools-picker"))
        })
        .unwrap();
        let root = host.render(environment).unwrap();
        assert_eq!(
            vector_colors(find(&root, "__devtools-picker")),
            vec![theme.primary_foreground]
        );
    }
}
