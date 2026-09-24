use super::{create, id, insert};
use argui_host::{Host, HostError, HostId, Operation};
use argui_schema::{SchemaValue, builtin};

#[test]
fn dry_run_builds_the_result_without_publishing_or_mutating_the_host() {
    let mut host = Host::with_builtins().unwrap();
    let root = HostId::new(91, 1);
    let batch = [
        Operation::Create {
            id: root,
            native_type: builtin::TEXT,
        },
        Operation::SetProperty {
            id: root,
            property: builtin::TEXT_VALUE,
            value: Some(SchemaValue::String("ready".into())),
        },
        Operation::SetRoot { id: Some(root) },
    ];
    host.validate(&batch).unwrap();
    assert_eq!(host.root_id(), None);
    assert!(host.root_element().is_none());
    host.commit(&batch).unwrap();
    assert_eq!(host.root_id(), Some(root));

    let invalid = [Operation::SetProperty {
        id: root,
        property: builtin::TEXT_VALUE,
        value: Some(SchemaValue::Bool(true)),
    }];
    assert!(host.validate(&invalid).is_err());
    assert_eq!(host.root_id(), Some(root));
    host.commit(&[Operation::SetProperty {
        id: root,
        property: builtin::TEXT_VALUE,
        value: Some(SchemaValue::String("still ready".into())),
    }])
    .unwrap();
}

#[test]
fn duplicate_live_id_and_root_reparenting_are_transactional() {
    let mut host = Host::with_builtins().unwrap();
    host.commit(&[
        create(1, builtin::COLUMN),
        create(2, builtin::RECTANGLE),
        Operation::SetRoot { id: Some(id(2)) },
    ])
    .unwrap();
    assert_eq!(
        host.commit(&[create(2, builtin::RECTANGLE)]),
        Err(HostError::StaleId(id(2)))
    );
    assert_eq!(host.root_id(), Some(id(2)));
    host.commit(&[insert(1, 2), Operation::SetRoot { id: Some(id(1)) }])
        .unwrap();
    assert_eq!(host.root_id(), Some(id(1)));
    assert_eq!(host.root_element().unwrap().children.len(), 1);
    host.commit(&[Operation::SetRoot { id: None }]).unwrap();
    assert_eq!(host.root_id(), None);
}
