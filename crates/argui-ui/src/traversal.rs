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

pub(crate) fn nth_element(root: &Element, target: usize) -> Option<&Element> {
    fn visit<'a>(element: &'a Element, target: usize, cursor: &mut usize) -> Option<&'a Element> {
        if *cursor == target {
            return Some(element);
        }
        *cursor += 1;
        element
            .children
            .iter()
            .find_map(|child| visit(child, target, cursor))
    }

    visit(root, target, &mut 0)
}
