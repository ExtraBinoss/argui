use crate::Element;

pub(crate) fn flattened(root: &Element) -> Vec<&Element> {
    fn visit<'a>(element: &'a Element, output: &mut Vec<&'a Element>) {
        output.push(element);
        for child in &element.children {
            visit(child, output);
        }
    }

    let mut output = Vec::new();
    visit(root, &mut output);
    output
}

/// Returns preorder elements with their original indices and visible ancestors.
///
/// A hidden ancestor excludes its entire subtree while indices still match the
/// full presentation tree used by layout and animation bindings.
///
/// * `root` — root of the presentation tree.
///
/// Returns visible elements paired with their full-tree preorder indices.
pub(crate) fn flattened_visible(root: &Element) -> Vec<(usize, &Element)> {
    /// Visits `element` and descendants, incrementing `index` for every node.
    ///
    /// `visible` records ancestor visibility; `output` receives only visible
    /// elements and their original preorder indices.
    fn visit<'a>(
        element: &'a Element,
        visible: bool,
        index: &mut usize,
        output: &mut Vec<(usize, &'a Element)>,
    ) {
        let current = *index;
        *index += 1;
        let visible = visible && element.style.display != crate::Display::None;
        if visible {
            output.push((current, element));
        }
        for child in &element.children {
            visit(child, visible, index, output);
        }
    }

    let mut output = Vec::new();
    visit(root, true, &mut 0, &mut output);
    output
}
