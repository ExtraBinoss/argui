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
