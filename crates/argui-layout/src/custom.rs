use crate::engine::NodeMap;
use argui_core::Size;
use argui_ui::{CustomMeasurement, ElementKind};

#[derive(Clone, Debug, PartialEq)]
pub struct CustomElementStats {
    pub node: argui_ui::NodeId,
    pub type_name: &'static str,
    pub phases: argui_ui::CustomPhaseStats,
}

pub(crate) fn stats(root: Option<&NodeMap>) -> Vec<CustomElementStats> {
    fn collect(map: &NodeMap, output: &mut Vec<CustomElementStats>) {
        if let (ElementKind::Custom(description), Some(state)) =
            (&map.element.kind, &map.custom_state)
        {
            output.push(CustomElementStats {
                node: map.node,
                type_name: description.type_name(),
                phases: state.phase_stats(),
            });
        }
        for child in &map.children {
            collect(child, output);
        }
    }
    let mut output = Vec::new();
    if let Some(root) = root {
        collect(root, &mut output);
    }
    output
}

pub(crate) fn validate(element: &argui_ui::Element) -> Result<(), crate::LayoutError> {
    if element.layout_boundary
        && (!matches!(element.kind, ElementKind::Container) || element.children.len() != 1)
    {
        return Err(crate::LayoutError::InvalidBoundary(
            "expected a container with one content child",
        ));
    }
    for child in &element.children {
        if (matches!(child.kind, ElementKind::Custom(_))
            || matches!(element.kind, ElementKind::Custom(_)))
            && let Some(key) = &child.key
            && element
                .children
                .iter()
                .filter(|other| other.key.as_ref() == Some(key))
                .count()
                > 1
        {
            return Err(crate::LayoutError::Custom(format!(
                "duplicate custom element key: {key}"
            )));
        }
        validate(child)?;
    }
    Ok(())
}

pub(crate) fn install(
    tree: &mut crate::layout_tree::LayoutTree,
    id: taffy::NodeId,
    kind: &ElementKind,
    state: Option<std::rc::Rc<argui_ui::CustomState>>,
) -> Result<Option<std::rc::Rc<argui_ui::CustomState>>, crate::LayoutError> {
    if let ElementKind::Custom(custom) = kind {
        let state = state.unwrap_or_else(|| std::rc::Rc::new(custom.create_state()));
        tree.set_custom(id, Some((custom.clone(), state.clone())))?;
        Ok(Some(state))
    } else {
        Ok(None)
    }
}

pub(crate) fn validate_measure(
    custom: &argui_ui::CustomDescription,
    measured: CustomMeasurement,
) -> Result<CustomMeasurement, String> {
    let Size { width, height } = measured.size;
    if !width.is_finite()
        || !height.is_finite()
        || width < 0.0
        || height < 0.0
        || measured
            .baseline
            .is_some_and(|value| !value.is_finite() || value < 0.0 || value > height)
    {
        return Err(format!(
            "{} returned invalid measurement {measured:?}",
            custom.type_name()
        ));
    }
    Ok(measured)
}

pub(crate) fn paint(
    map: &NodeMap,
    element: &argui_ui::Element,
    node: crate::LayoutNode,
    output: &mut crate::LayoutOutput,
    transform: argui_core::Affine2D,
    clips: &argui_paint::ClipChain,
    style: argui_paint::QuadStyle,
) {
    if let (ElementKind::Custom(custom), Some(state)) = (&element.kind, &map.custom_state) {
        custom.paint(
            state,
            &mut argui_ui::CustomPaintContext {
                bounds: node.bounds,
                transform,
                clips,
                display_list: &mut output.display_list,
                object: argui_paint::RenderObjectId::new(
                    argui_paint::ProfileDomain::Ui,
                    node.node.get(),
                ),
                opacity: style.opacity,
                radii: style.radii,
            },
        );
    }
}
