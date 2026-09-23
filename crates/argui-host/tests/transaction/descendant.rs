use super::*;
use argui_ui::ElementKind;

#[test]
fn changed_text_inside_unchanged_rectangle_reaches_visible_root() {
    let mut host = Host::with_builtins().unwrap();
    let mut tree = None;
    commit(
        &mut host,
        &mut tree,
        &[
            create(1, builtin::RECTANGLE),
            create(2, builtin::TEXT),
            Operation::SetProperty {
                id: id(2),
                property: builtin::TEXT_VALUE,
                value: Some(SchemaValue::String("Vulkan".into())),
            },
            insert(1, 2),
            Operation::SetRoot { id: Some(id(1)) },
        ],
    )
    .unwrap();

    let update = commit(
        &mut host,
        &mut tree,
        &[Operation::SetProperty {
            id: id(2),
            property: builtin::TEXT_VALUE,
            value: Some(SchemaValue::String("WebGPU".into())),
        }],
    )
    .unwrap();
    assert_ne!(update, TreeUpdate::None);
    let root = host.root_element().unwrap();
    assert!(matches!(
        &root.children[0].kind,
        ElementKind::Text { content, .. } if content.as_str() == "WebGPU"
    ));
}
