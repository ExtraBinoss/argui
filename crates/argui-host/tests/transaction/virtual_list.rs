use argui_host::{Host, HostId, Operation};
use argui_schema::{SchemaValue, builtin};

fn set(property: argui_schema::PropertyId, value: SchemaValue) -> Operation {
    Operation::SetProperty {
        id: HostId::new(1, 1),
        property,
        value: Some(value),
    }
}

#[test]
fn virtual_measurements_survive_host_commits_and_atomic_count_edits() {
    let mut host = Host::with_builtins().unwrap();
    host.commit(&[
        Operation::Create {
            id: HostId::new(1, 1),
            native_type: builtin::VIRTUAL_WINDOW,
        },
        set(builtin::KEY, SchemaValue::String("list".into())),
        set(builtin::ROW_HEIGHT, SchemaValue::Float(30.0)),
        set(builtin::VARIABLE_HEIGHT, SchemaValue::Bool(true)),
        set(builtin::VIRTUAL_HORIZONTAL, SchemaValue::Bool(true)),
        set(builtin::VIRTUAL_VIEWPORT_WIDTH, SchemaValue::Float(120.0)),
        set(builtin::ITEM_COUNT, SchemaValue::Int(100)),
        set(builtin::WINDOW_START, SchemaValue::Int(0)),
        Operation::SetRoot {
            id: Some(HostId::new(1, 1)),
        },
    ])
    .unwrap();

    let mut before = host
        .root_element()
        .unwrap()
        .virtual_viewport()
        .unwrap()
        .list
        .clone();
    before.measure(3, 75.0, 0.0);
    assert_eq!(before.item_extent(3), Some(75.0));

    host.commit(&[set(builtin::SCROLL_OFFSET, SchemaValue::Float(10.0))])
        .unwrap();
    let after = host
        .root_element()
        .unwrap()
        .virtual_viewport()
        .unwrap()
        .list
        .clone();
    assert_eq!(after.item_extent(3), Some(75.0));
    assert!(after.is_measured(3));
    assert!(after.is_horizontal());

    let rejected = host.commit(&[
        set(builtin::ITEM_COUNT, SchemaValue::Int(120)),
        set(builtin::ROW_HEIGHT, SchemaValue::Float(0.0)),
    ]);
    assert!(rejected.is_err());
    assert_eq!(
        host.root_element()
            .unwrap()
            .virtual_viewport()
            .unwrap()
            .list
            .item_count(),
        100
    );
    assert_eq!(after.item_count(), 100);

    host.commit(&[
        set(builtin::ITEM_COUNT, SchemaValue::Int(120)),
        set(builtin::ROW_HEIGHT, SchemaValue::Float(30.0)),
    ])
    .unwrap();
    let appended = host
        .root_element()
        .unwrap()
        .virtual_viewport()
        .unwrap()
        .list
        .clone();
    assert_eq!(appended.item_count(), 120);
    assert_eq!(appended.item_extent(3), Some(75.0));
    assert_eq!(appended.item_extent(119), Some(30.0));
    assert_eq!(after.item_count(), 100);

    host.commit(&[set(builtin::VIRTUAL_DATA_VERSION, SchemaValue::Int(1))])
        .unwrap();
    let reordered = host
        .root_element()
        .unwrap()
        .virtual_viewport()
        .unwrap()
        .list
        .clone();
    assert_eq!(reordered.item_extent(3), Some(30.0));
    assert!(!reordered.is_measured(3));
    assert_eq!(appended.item_extent(3), Some(75.0));
}
