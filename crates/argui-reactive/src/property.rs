use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::{Rc, Weak},
};

use crate::{
    Subscription,
    graph::{self, Dependent, Node},
};

type Observer<T> = Rc<dyn Fn(&T)>;

/// A typed observable value with revision and dependency tracking.
pub struct Property<T: Clone + PartialEq + 'static> {
    pub(crate) node: Rc<PropertyNode<T>>,
}

impl<T: Clone + PartialEq + 'static> Clone for Property<T> {
    fn clone(&self) -> Self {
        Self {
            node: Rc::clone(&self.node),
        }
    }
}

pub(crate) struct PropertyNode<T: Clone + PartialEq + 'static> {
    id: u64,
    value: RefCell<T>,
    revision: Cell<u64>,
    dependents: RefCell<Vec<Dependent>>,
    observers: RefCell<BTreeMap<u64, Observer<T>>>,
    next_observer: Cell<u64>,
}

impl<T: Clone + PartialEq + 'static> Property<T> {
    /// Creates a typed property with `value` as revision zero.
    ///
    /// * `value` — initial property value.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self {
            node: Rc::new(PropertyNode {
                id: graph::next_node_id(),
                value: RefCell::new(value),
                revision: Cell::new(0),
                dependents: RefCell::new(Vec::new()),
                observers: RefCell::new(BTreeMap::new()),
                next_observer: Cell::new(1),
            }),
        }
    }

    /// Returns the current value and registers a dependency when read by a binding.
    #[must_use]
    pub fn get(&self) -> T {
        let source: Rc<dyn Node> = self.node.clone();
        graph::track(source);
        self.node.value.borrow().clone()
    }

    /// Reads the current value without cloning it and registers a dependency.
    ///
    /// `read` receives a shared borrow valid only for the callback; its return
    /// value is forwarded to the caller. This is useful for large model arrays
    /// whose visible slice is much smaller than the complete collection.
    ///
    /// # Panics
    ///
    /// Panics if `read` attempts to mutate this property while it is borrowed.
    pub fn with<R>(&self, read: impl FnOnce(&T) -> R) -> R {
        let source: Rc<dyn Node> = self.node.clone();
        graph::track(source);
        read(&self.node.value.borrow())
    }

    /// Replaces the value when it changed and returns whether a write occurred.
    ///
    /// * `value` — next typed value.
    pub fn set(&self, value: T) -> bool {
        if *self.node.value.borrow() == value {
            return false;
        }
        *self.node.value.borrow_mut() = value;
        self.node
            .revision
            .set(self.node.revision.get().wrapping_add(1));
        graph::invalidate(&self.node.dependents);
        let pending: Rc<dyn Node> = self.node.clone();
        graph::schedule(pending);
        true
    }

    /// Mutates a cloned value and commits it only when the result changed.
    ///
    /// * `update` — operation that mutates the candidate value.
    pub fn update(&self, update: impl FnOnce(&mut T)) -> bool {
        let mut value = self.get();
        update(&mut value);
        self.set(value)
    }

    /// Mutates the stored value directly and invalidates dependents when changed.
    ///
    /// * `edit` — mutates the value and returns whether it changed. It must leave
    ///   the value unchanged when returning `false`.
    ///
    /// Returns whether the value changed.
    ///
    /// # Panics
    ///
    /// Panics if the property is already borrowed.
    pub fn mutate(&self, edit: impl FnOnce(&mut T) -> bool) -> bool {
        let changed = {
            let mut value = self.node.value.borrow_mut();
            edit(&mut value)
        };
        if changed {
            self.node
                .revision
                .set(self.node.revision.get().wrapping_add(1));
            graph::invalidate(&self.node.dependents);
            let pending: Rc<dyn Node> = self.node.clone();
            graph::schedule(pending);
        }
        changed
    }

    /// Returns the number of accepted value changes.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.node.revision.get()
    }

    /// Registers an observer called after each committed notification batch.
    ///
    /// The callback is not called for the initial value. Retain the returned
    /// subscription for as long as observation should remain active.
    ///
    /// * `observer` — callback receiving the latest property value.
    #[must_use]
    pub fn observe(&self, observer: impl Fn(&T) + 'static) -> Subscription {
        let id = self.node.next_observer.get();
        self.node.next_observer.set(id.wrapping_add(1).max(1));
        self.node
            .observers
            .borrow_mut()
            .insert(id, Rc::new(observer));
        let node: Weak<PropertyNode<T>> = Rc::downgrade(&self.node);
        Subscription::new(move || {
            if let Some(node) = node.upgrade() {
                node.observers.borrow_mut().remove(&id);
            }
        })
    }
}

impl<T: Clone + PartialEq + 'static> Node for PropertyNode<T> {
    fn id(&self) -> u64 {
        self.id
    }

    fn add_dependent(&self, dependent: Dependent) {
        self.dependents.borrow_mut().push(dependent);
    }

    fn mark_dirty(&self, _generation: u64) -> bool {
        false
    }

    fn flush(&self) {
        let value = self.value.borrow().clone();
        let observers = self
            .observers
            .borrow()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for observer in observers {
            observer(&value);
        }
    }
}
