use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::{Rc, Weak},
};

use crate::{
    ReactiveError, Subscription,
    graph::{self, Dependent, Node},
};

type Evaluator<T> = Box<dyn Fn() -> Result<T, ReactiveError>>;
type Observer<T> = Rc<dyn Fn(&T)>;

/// A lazily cached typed value whose dependencies are captured during evaluation.
pub struct Computed<T: Clone + PartialEq + 'static> {
    node: Rc<ComputedNode<T>>,
}

impl<T: Clone + PartialEq + 'static> Clone for Computed<T> {
    fn clone(&self) -> Self {
        Self {
            node: Rc::clone(&self.node),
        }
    }
}

struct ComputedNode<T: Clone + PartialEq + 'static> {
    id: u64,
    label: String,
    evaluator: Evaluator<T>,
    value: RefCell<Option<T>>,
    dirty: Cell<bool>,
    generation: Cell<u64>,
    revision: Cell<u64>,
    dependents: RefCell<Vec<Dependent>>,
    observers: RefCell<BTreeMap<u64, Observer<T>>>,
    next_observer: Cell<u64>,
    self_node: RefCell<Weak<dyn Node>>,
    notification_pending: Cell<bool>,
}

impl<T: Clone + PartialEq + 'static> Computed<T> {
    /// Creates an infallible computed value with a diagnostic label.
    ///
    /// * `label` — stable human-readable binding name used in cycle diagnostics.
    /// * `evaluator` — pure callback that reads properties and returns the derived value.
    #[must_use]
    pub fn new(label: impl Into<String>, evaluator: impl Fn() -> T + 'static) -> Self {
        Self::try_new(label, move || Ok(evaluator()))
    }

    /// Creates a fallible computed value with a diagnostic label.
    ///
    /// * `label` — stable human-readable binding name used in diagnostics.
    /// * `evaluator` — pure callback returning a typed value or evaluation error.
    #[must_use]
    pub fn try_new(
        label: impl Into<String>,
        evaluator: impl Fn() -> Result<T, ReactiveError> + 'static,
    ) -> Self {
        let node = Rc::new(ComputedNode {
            id: graph::next_node_id(),
            label: label.into(),
            evaluator: Box::new(evaluator),
            value: RefCell::new(None),
            dirty: Cell::new(true),
            generation: Cell::new(0),
            revision: Cell::new(0),
            dependents: RefCell::new(Vec::new()),
            observers: RefCell::new(BTreeMap::new()),
            next_observer: Cell::new(1),
            self_node: RefCell::new(Weak::<ComputedNode<T>>::new()),
            notification_pending: Cell::new(false),
        });
        let erased: Rc<dyn Node> = node.clone();
        *node.self_node.borrow_mut() = Rc::downgrade(&erased);
        Self { node }
    }

    /// Returns the cached value, reevaluating once when a dependency is dirty.
    ///
    /// # Errors
    ///
    /// Returns a deterministic cycle path or the evaluator's explicit error.
    pub fn get(&self) -> Result<T, ReactiveError> {
        let changed = self.node.evaluate_if_dirty()?;
        let source: Rc<dyn Node> = self.node.clone();
        if changed {
            self.node.publish_change();
            graph::schedule(source.clone());
        }
        graph::track(source);
        Ok(self
            .node
            .value
            .borrow()
            .as_ref()
            .expect("successful computed evaluation stores a value")
            .clone())
    }

    /// Returns the number of evaluations that changed the cached value.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.node.revision.get()
    }

    /// Registers an observer called when reevaluation changes the cached value.
    ///
    /// * `observer` — callback receiving the latest derived value.
    #[must_use]
    pub fn observe(&self, observer: impl Fn(&T) + 'static) -> Subscription {
        let id = self.node.next_observer.get();
        self.node.next_observer.set(id.wrapping_add(1).max(1));
        self.node
            .observers
            .borrow_mut()
            .insert(id, Rc::new(observer));
        let node: Weak<ComputedNode<T>> = Rc::downgrade(&self.node);
        Subscription::new(move || {
            if let Some(node) = node.upgrade() {
                node.observers.borrow_mut().remove(&id);
            }
        })
    }
}

impl<T: Clone + PartialEq + 'static> ComputedNode<T> {
    fn evaluate_if_dirty(&self) -> Result<bool, ReactiveError> {
        if !self.dirty.get() && self.value.borrow().is_some() {
            return Ok(false);
        }
        let generation = self.generation.get().wrapping_add(1);
        self.generation.set(generation);
        let dependent = self.self_node.borrow().clone();
        let result = graph::enter_evaluation(self.id, &self.label, || {
            graph::collect(dependent, generation, || (self.evaluator)())
        })??;
        let changed = self.value.borrow().as_ref() != Some(&result);
        if changed {
            *self.value.borrow_mut() = Some(result);
            self.revision.set(self.revision.get().wrapping_add(1));
            self.notification_pending.set(true);
        }
        self.dirty.set(false);
        Ok(changed)
    }

    fn publish_change(&self) {
        graph::invalidate(&self.dependents);
    }

    fn notify(&self) {
        if !self.notification_pending.replace(false) {
            return;
        }
        let value = self.value.borrow().as_ref().cloned();
        let observers = self
            .observers
            .borrow()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        if let Some(value) = value {
            for observer in observers {
                observer(&value);
            }
        }
    }
}

impl<T: Clone + PartialEq + 'static> Node for ComputedNode<T> {
    fn id(&self) -> u64 {
        self.id
    }

    fn add_dependent(&self, dependent: Dependent) {
        self.dependents.borrow_mut().push(dependent);
    }

    fn mark_dirty(&self, generation: u64) -> bool {
        if generation != self.generation.get() {
            return false;
        }
        self.dirty.set(true);
        true
    }

    fn flush(&self) {
        if let Ok(changed) = self.evaluate_if_dirty()
            && changed
        {
            self.publish_change();
        }
        self.notify();
    }
}
