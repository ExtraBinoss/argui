use argui_animation::Time;
use argui_host::{Host, HostError, HostId, Operation};
use argui_schema::{SchemaValue, builtin};
use argui_ui::{PropertyBinding, TreeUpdate, UiTree};
use web_time::Instant;

#[path = "transaction/descendant.rs"]
mod descendant;

/// Builds a first-generation presentation ID for a test slot.
fn id(slot: u32) -> HostId {
    HostId::new(slot, 1)
}

/// Builds a primitive creation operation for one test slot.
fn create(slot: u32, native_type: argui_schema::NativeTypeId) -> Operation {
    Operation::Create {
        id: id(slot),
        native_type,
    }
}

/// Appends one child to a test parent.
fn insert(parent: u32, child: u32) -> Operation {
    Operation::Insert {
        parent: id(parent),
        child: id(child),
        before: None,
    }
}

/// Applies a batch to the host and its single canonical UI tree.
fn commit(
    host: &mut Host,
    tree: &mut Option<UiTree>,
    operations: &[Operation],
) -> Result<TreeUpdate, HostError> {
    host.commit(operations)?;
    Ok(match (tree.as_mut(), host.root_element()) {
        (Some(tree), Some(root)) => tree.update(root),
        (None, Some(root)) => {
            *tree = Some(UiTree::new(root));
            TreeUpdate::Layout
        }
        (Some(_), None) => {
            *tree = None;
            TreeUpdate::Layout
        }
        (None, None) => TreeUpdate::None,
    })
}

#[test]
fn loop_playback_preserves_phase_and_rejected_batches_cannot_pause_it() {
    let mut host = Host::with_builtins().unwrap();
    let mut tree = None;
    commit(
        &mut host,
        &mut tree,
        &[
            create(1, builtin::RECTANGLE),
            Operation::SetProperty {
                id: id(1),
                property: builtin::ROTATION_LOOP_MS,
                value: Some(SchemaValue::Float(1000.0)),
            },
            Operation::SetRoot { id: Some(id(1)) },
        ],
    )
    .unwrap();
    let PropertyBinding::Transform(binding) = &host.root_element().unwrap().bindings[0] else {
        panic!("rotation loop must bind a transform");
    };
    let motion = binding.motion.clone();
    let identity = motion.identity();
    let tree = tree.as_mut().unwrap();
    tree.advance_animations(Time::from_nanos(1));
    tree.advance_animations(Time::from_nanos(250_000_001));
    let before_pause = motion.value().rotation;
    assert!(before_pause > 1.0);

    let rejected = host.commit(&[
        Operation::SetProperty {
            id: id(1),
            property: builtin::LOOP_PLAYING,
            value: Some(SchemaValue::Bool(false)),
        },
        Operation::SetProperty {
            id: id(1),
            property: builtin::LOOP_MS,
            value: Some(SchemaValue::Float(0.0)),
        },
    ]);
    assert!(matches!(rejected, Err(HostError::Schema(_))));
    assert!(motion.is_active());
    assert!(tree.wants_animation_frame());

    let pause = [Operation::SetProperty {
        id: id(1),
        property: builtin::LOOP_PLAYING,
        value: Some(SchemaValue::Bool(false)),
    }];
    host.commit(&pause).unwrap();
    assert_eq!(tree.update(host.root_element().unwrap()), TreeUpdate::None);
    assert_eq!(
        host.root_element().unwrap().bindings[0].track().identity(),
        identity
    );
    assert!(!motion.is_active());
    assert!(!tree.wants_animation_frame());
    tree.advance_animations(Time::from_nanos(1_250_000_001));
    assert_eq!(motion.value().rotation, before_pause);

    let resume = [Operation::SetProperty {
        id: id(1),
        property: builtin::LOOP_PLAYING,
        value: Some(SchemaValue::Bool(true)),
    }];
    host.commit(&resume).unwrap();
    assert_eq!(tree.update(host.root_element().unwrap()), TreeUpdate::None);
    assert_eq!(
        host.root_element().unwrap().bindings[0].track().identity(),
        identity
    );
    assert!(tree.wants_animation_frame());
    tree.advance_animations(Time::from_nanos(1_250_000_001));
    assert_eq!(motion.value().rotation, before_pause);
    tree.advance_animations(Time::from_nanos(1_500_000_001));
    assert!(motion.value().rotation > before_pause + 1.0);
}

#[test]
fn invalid_batch_keeps_the_visible_tree_and_host_graph() {
    let mut host = Host::with_builtins().unwrap();
    let mut tree = None;
    commit(
        &mut host,
        &mut tree,
        &[
            create(1, builtin::RECTANGLE),
            Operation::SetRoot { id: Some(id(1)) },
        ],
    )
    .unwrap();
    let revision = tree.as_ref().unwrap().revision();
    let result = host.commit(&[
        create(2, builtin::ROW),
        Operation::SetProperty {
            id: id(1),
            property: builtin::BACKGROUND,
            value: Some(SchemaValue::Color(argui_ui::Color::WHITE)),
        },
        Operation::SetRoot { id: Some(id(2)) },
    ]);
    assert!(matches!(result, Err(HostError::Schema(_))));
    assert_eq!(host.root_id(), Some(id(1)));
    assert_eq!(tree.as_ref().unwrap().revision(), revision);
    assert_eq!(
        tree.as_ref().unwrap().root().key.as_deref(),
        Some("host:1:1")
    );
    assert!(matches!(
        host.commit(&[insert(1, 2)]),
        Err(HostError::UnknownId(_))
    ));
}

#[test]
fn a_changed_leaf_reuses_large_siblings() {
    let mut host = Host::with_builtins().unwrap();
    let mut tree = None;
    let mut operations = vec![
        create(1, builtin::COLUMN),
        create(2, builtin::ROW),
        create(3, builtin::TEXT),
        Operation::SetProperty {
            id: id(3),
            property: builtin::TEXT_VALUE,
            value: Some(SchemaValue::String("old".into())),
        },
    ];
    for slot in 4..1004 {
        operations.extend([
            create(slot, builtin::TEXT),
            Operation::SetProperty {
                id: id(slot),
                property: builtin::CONTENT,
                value: Some(SchemaValue::String(format!("item {slot}"))),
            },
            insert(2, slot),
        ]);
    }
    operations.extend([
        insert(1, 2),
        insert(1, 3),
        Operation::SetRoot { id: Some(id(1)) },
    ]);
    commit(&mut host, &mut tree, &operations).unwrap();
    let update = commit(
        &mut host,
        &mut tree,
        &[Operation::SetProperty {
            id: id(3),
            property: builtin::TEXT_VALUE,
            value: Some(SchemaValue::String("changed".into())),
        }],
    )
    .unwrap();
    assert_eq!(update, TreeUpdate::Layout);
    assert!(tree.as_ref().unwrap().update_stats().visited <= 3);
    assert_eq!(tree.as_ref().unwrap().update_stats().shared_subtrees, 1);
}

#[test]
fn moving_a_node_between_parents_keeps_its_native_identity() {
    let mut host = Host::with_builtins().unwrap();
    let mut tree = None;
    commit(
        &mut host,
        &mut tree,
        &[
            create(1, builtin::COLUMN),
            create(2, builtin::ROW),
            create(3, builtin::ROW),
            create(4, builtin::TEXT),
            Operation::SetProperty {
                id: id(4),
                property: builtin::CONTENT,
                value: Some(SchemaValue::String("moving".into())),
            },
            insert(2, 4),
            insert(1, 2),
            insert(1, 3),
            Operation::SetRoot { id: Some(id(1)) },
        ],
    )
    .unwrap();
    let native_id = tree.as_ref().unwrap().node_id_at(2).unwrap();
    commit(&mut host, &mut tree, &[insert(3, 4)]).unwrap();
    assert_eq!(tree.as_ref().unwrap().node_id_at(3), Some(native_id));
}

#[test]
fn removed_slots_need_a_new_generation_and_cycles_are_rejected() {
    let mut host = Host::with_builtins().unwrap();
    host.commit(&[
        create(1, builtin::COLUMN),
        create(2, builtin::ROW),
        insert(1, 2),
        Operation::SetRoot { id: Some(id(1)) },
    ])
    .unwrap();
    assert_eq!(host.commit(&[insert(2, 1)]), Err(HostError::Cycle(id(1))));
    host.commit(&[Operation::Remove { id: id(2) }]).unwrap();
    assert_eq!(
        host.commit(&[create(2, builtin::TEXT)]),
        Err(HostError::StaleId(id(2)))
    );
    host.commit(&[Operation::Create {
        id: HostId::new(2, 2),
        native_type: builtin::ROW,
    }])
    .unwrap();
}

#[test]
fn invalid_ids_types_children_and_root_attachments_leave_host_unchanged() {
    let mut host = Host::with_builtins().unwrap();
    assert_eq!(
        host.commit(&[create(0, builtin::TEXT)]),
        Err(HostError::InvalidId(id(0)))
    );
    assert_eq!(
        host.commit(&[Operation::Create {
            id: HostId::new(2, 0),
            native_type: builtin::TEXT,
        }]),
        Err(HostError::InvalidId(HostId::new(2, 0)))
    );
    assert!(matches!(
        host.commit(&[create(1, argui_schema::NativeTypeId::from_raw(9999))]),
        Err(HostError::UnknownType(_))
    ));
    assert!(matches!(
        host.commit(&[
            create(1, builtin::TEXT),
            create(2, builtin::ROW),
            insert(1, 2)
        ]),
        Err(HostError::NoChildren(_))
    ));
    assert_eq!(host.root_id(), None);
    assert!(host.root_element().is_none());

    host.commit(&[
        create(1, builtin::COLUMN),
        create(2, builtin::ROW),
        insert(1, 2),
        Operation::SetRoot { id: Some(id(1)) },
    ])
    .unwrap();
    assert_eq!(
        host.commit(&[Operation::SetRoot { id: Some(id(2)) }]),
        Err(HostError::AttachedRoot(id(2)))
    );
    assert_eq!(
        host.commit(&[Operation::Insert {
            parent: id(1),
            child: id(2),
            before: Some(id(2))
        }]),
        Err(HostError::InvalidBefore(id(2), id(1)))
    );
    assert_eq!(host.root_id(), Some(id(1)));
}

#[test]
fn detached_edits_do_not_change_visible_root_and_subtree_removal_releases_slots() {
    let mut host = Host::with_builtins().unwrap();
    host.commit(&[
        create(1, builtin::COLUMN),
        Operation::SetRoot { id: Some(id(1)) },
    ])
    .unwrap();
    let detached = host
        .commit(&[
            create(2, builtin::TEXT),
            Operation::SetProperty {
                id: id(2),
                property: builtin::CONTENT,
                value: Some(SchemaValue::String("temporary".into())),
            },
        ])
        .unwrap();
    assert!(!detached.visible_changed);
    host.commit(&[insert(1, 2)]).unwrap();
    assert_eq!(host.root_element().unwrap().children.len(), 1);
    let removed = host.commit(&[Operation::Remove { id: id(1) }]).unwrap();
    assert!(removed.visible_changed);
    assert!(host.root_element().is_none());
    assert!(matches!(
        host.commit(&[Operation::Remove { id: id(2) }]),
        Err(HostError::UnknownId(_))
    ));
    assert!(matches!(
        host.commit(&[create(2, builtin::TEXT)]),
        Err(HostError::StaleId(_))
    ));
    host.commit(&[
        Operation::Create {
            id: HostId::new(2, 2),
            native_type: builtin::TEXT,
        },
        Operation::SetProperty {
            id: HostId::new(2, 2),
            property: builtin::CONTENT,
            value: Some(SchemaValue::String("reused slot".into())),
        },
    ])
    .unwrap();
}

#[test]
fn before_insertion_and_listener_replacement_are_atomic() {
    let mut host = Host::with_builtins().unwrap();
    host.commit(&[
        create(1, builtin::COLUMN),
        create(2, builtin::FOCUS_SCOPE),
        create(3, builtin::FOCUS_SCOPE),
        insert(1, 2),
        Operation::Insert {
            parent: id(1),
            child: id(3),
            before: Some(id(2)),
        },
        Operation::SetListener {
            id: id(2),
            event: builtin::CLICK,
            callback: Some(argui_host::CallbackId(7)),
        },
        Operation::SetRoot { id: Some(id(1)) },
    ])
    .unwrap();
    let children = host.root_element().unwrap().children.clone();
    assert_eq!(children[0].key.as_deref(), Some("host:3:1"));
    assert_eq!(children[1].key.as_deref(), Some("host:2:1"));
    host.commit(&[Operation::SetListener {
        id: id(2),
        event: builtin::CLICK,
        callback: None,
    }])
    .unwrap();
    assert_eq!(host.root_element().unwrap().children.len(), 2);
}

#[test]
fn empty_text_anchor_is_absent_from_ui_tree_and_reappears_in_order() {
    let mut host = Host::with_builtins().unwrap();
    host.commit(&[
        create(1, builtin::COLUMN),
        create(2, builtin::TEXT),
        create(3, builtin::TEXT),
        Operation::SetProperty {
            id: id(2),
            property: builtin::TEXT_VALUE,
            value: Some(SchemaValue::String(String::new())),
        },
        Operation::SetProperty {
            id: id(3),
            property: builtin::TEXT_VALUE,
            value: Some(SchemaValue::String("after".into())),
        },
        insert(1, 2),
        insert(1, 3),
        Operation::SetRoot { id: Some(id(1)) },
    ])
    .unwrap();
    let children = &host.root_element().unwrap().children;
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].key.as_deref(), Some("host:3:1"));

    host.commit(&[Operation::SetProperty {
        id: id(2),
        property: builtin::TEXT_VALUE,
        value: Some(SchemaValue::String("before".into())),
    }])
    .unwrap();
    let children = &host.root_element().unwrap().children;
    assert_eq!(children.len(), 2);
    assert_eq!(children[0].key.as_deref(), Some("host:2:1"));
    assert_eq!(children[1].key.as_deref(), Some("host:3:1"));

    host.commit(&[Operation::SetProperty {
        id: id(2),
        property: builtin::TEXT_VALUE,
        value: Some(SchemaValue::String(String::new())),
    }])
    .unwrap();
    let children = &host.root_element().unwrap().children;
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].key.as_deref(), Some("host:3:1"));
}

#[test]
fn isolated_update_in_ten_thousand_siblings_preserves_identity_and_reports_cost() {
    const COUNT: u32 = 10_000;
    let root = HostId::new(1, 1);
    let mut batch = vec![Operation::Create {
        id: root,
        native_type: builtin::COLUMN,
    }];
    for index in 0..COUNT {
        let id = HostId::new(index + 2, 1);
        batch.push(Operation::Create {
            id,
            native_type: builtin::TEXT,
        });
        batch.push(Operation::SetProperty {
            id,
            property: builtin::CONTENT,
            value: Some(SchemaValue::String(index.to_string())),
        });
        batch.push(Operation::Insert {
            parent: root,
            child: id,
            before: None,
        });
    }
    batch.push(Operation::SetRoot { id: Some(root) });
    let mut host = Host::with_builtins().expect("native schema");
    let started = Instant::now();
    host.commit(&batch).expect("large initial mount");
    let initial_host_time = started.elapsed();
    let started = Instant::now();
    let mut tree = UiTree::new(host.root_element().expect("visible root"));
    let initial_tree_time = started.elapsed();
    let last = HostId::new(COUNT + 1, 1);
    let first = HostId::new(2, 1);
    let old_id = tree.node_id_at(COUNT as usize).expect("last text node");
    let started = Instant::now();
    let changed = host
        .commit(&[Operation::SetProperty {
            id: last,
            property: builtin::CONTENT,
            value: Some(SchemaValue::String("changed".into())),
        }])
        .expect("isolated text update");
    let host_time = started.elapsed();
    let started = Instant::now();
    tree.update(host.root_element().expect("visible root"));
    let tree_time = started.elapsed();
    let stats = tree.update_stats();
    eprintln!(
        "HOST_10K initial_host_us={} initial_tree_us={} property_host_us={} property_tree_us={} changed_nodes={} visited={} shared_subtrees={}",
        initial_host_time.as_micros(),
        initial_tree_time.as_micros(),
        host_time.as_micros(),
        tree_time.as_micros(),
        changed.changed_nodes,
        stats.visited,
        stats.shared_subtrees
    );
    assert_eq!(
        changed.changed_nodes, 2,
        "only the root and changed text are materialized"
    );
    assert_eq!(tree.node_id_at(COUNT as usize), Some(old_id));
    assert!(stats.shared_subtrees >= COUNT as usize - 1);

    let started = Instant::now();
    let reordered = host
        .commit(&[Operation::Insert {
            parent: root,
            child: last,
            before: Some(first),
        }])
        .expect("reorder one text node");
    let reorder_host_time = started.elapsed();
    let started = Instant::now();
    tree.update(host.root_element().expect("visible root"));
    let reorder_tree_time = started.elapsed();
    eprintln!(
        "HOST_10K reorder_host_us={} reorder_tree_us={} changed_nodes={} visited={}",
        reorder_host_time.as_micros(),
        reorder_tree_time.as_micros(),
        reordered.changed_nodes,
        tree.update_stats().visited
    );
    assert_eq!(tree.node_id_at(1), Some(old_id));
    assert!(
        tree.update_stats().visited <= 2,
        "a keyed move is classified at its parent"
    );
}

#[test]
fn large_navigation_batch_commits_atomically() {
    let mut host = Host::with_builtins().unwrap();
    let mut tree = None;
    commit(
        &mut host,
        &mut tree,
        &[
            create(1, builtin::COLUMN),
            Operation::SetRoot { id: Some(id(1)) },
        ],
    )
    .unwrap();
    let mut operations = Vec::new();
    for slot in 2..203 {
        operations.push(create(slot, builtin::TEXT));
        for value in ["first", "second", "final"] {
            operations.push(Operation::SetProperty {
                id: id(slot),
                property: builtin::CONTENT,
                value: Some(SchemaValue::String(value.into())),
            });
        }
        operations.push(insert(1, slot));
    }
    let started = Instant::now();
    let result = host.commit(&operations).unwrap();
    let host_time = started.elapsed();
    let started = Instant::now();
    let update = tree.as_mut().unwrap().update(host.root_element().unwrap());
    let tree_time = started.elapsed();
    eprintln!(
        "host-large-batch operations={} host_ms={:.3} tree_ms={:.3}",
        operations.len(),
        host_time.as_secs_f64() * 1000.0,
        tree_time.as_secs_f64() * 1000.0
    );
    assert_eq!(result.changed_nodes, 202);
    assert!(result.visible_changed);
    assert_eq!(update, TreeUpdate::Layout);
}
