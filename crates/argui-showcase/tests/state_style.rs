use argui_showcase::StateShowcase;
use argui_ui::Element;

#[test]
fn examples_follow_images_and_include_a_real_scrollbar() {
    let view = StateShowcase::default().view(argui_runtime::WindowEnvironment::default());
    assert!(node_index(&view, "visual-primitives") < node_index(&view, "state-style-examples"));

    for key in [
        "state-example-spring",
        "state-example-directional",
        "state-example-layout",
        "state-example-gradient",
    ] {
        assert!(element_by_key(&view, key).unwrap().has_state_animation());
    }

    let scroll = element_by_key(&view, "state-example-scrollbar").unwrap();
    assert!(
        scroll
            .scroll
            .as_ref()
            .and_then(|config| config.scrollbar.as_ref())
            .is_some()
    );
    assert_eq!(scroll.children.len(), 10);
    assert!(scroll.children.iter().all(Element::has_state_animation));
}

fn node_index(root: &Element, key: &str) -> usize {
    fn visit(element: &Element, key: &str, index: &mut usize) -> Option<usize> {
        let current = *index;
        *index += 1;
        if element.key.as_deref() == Some(key) {
            return Some(current);
        }
        element
            .children
            .iter()
            .find_map(|child| visit(child, key, index))
    }
    visit(root, key, &mut 0).expect("showcase key exists")
}

fn element_by_key<'a>(element: &'a Element, key: &str) -> Option<&'a Element> {
    if element.key.as_deref() == Some(key) {
        return Some(element);
    }
    element
        .children
        .iter()
        .find_map(|child| element_by_key(child, key))
}
