use std::collections::{BTreeMap, HashMap};

use argui_schema::{
    EventId, NativeElementInput, NativeEventValue, NativeSlotValue, NativeTypeId, PropertyId,
    SchemaError, SchemaRegistry, SchemaValue, builtin,
};
use argui_ui::{
    Element, EventHandler, EventHandlerId, EventOwnerId, RetainedIdentity, UiEvent, VirtualList,
};

use crate::{CallbackDelivery, CallbackId, HostId, Operation};

mod descendant;
mod validation;
mod virtual_list;

/// A rejected batch leaves both the host graph and its visible root unchanged.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum HostError {
    #[error("invalid host identity {0:?}")]
    InvalidId(HostId),
    #[error("host identity {0:?} is already live or has a stale generation")]
    StaleId(HostId),
    #[error("unknown host identity {0:?}")]
    UnknownId(HostId),
    #[error("native type {0:?} has no schema")]
    UnknownType(NativeTypeId),
    #[error("node {0:?} does not accept ordinary children")]
    NoChildren(HostId),
    #[error("node {0:?} is not a child of {1:?}")]
    InvalidBefore(HostId, HostId),
    #[error("inserting {0:?} would create a cycle")]
    Cycle(HostId),
    #[error("the root node must be detached: {0:?}")]
    AttachedRoot(HostId),
    #[error("event handler identities are exhausted")]
    HandlerExhausted,
    #[error(transparent)]
    Schema(#[from] SchemaError),
}

/// Visible effect and number of nodes changed by an accepted transaction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommitResult {
    pub visible_changed: bool,
    pub changed_nodes: usize,
}

#[derive(Clone)]
struct HostNode {
    native_type: NativeTypeId,
    properties: BTreeMap<PropertyId, SchemaValue>,
    listeners: BTreeMap<EventId, CallbackId>,
    children: Vec<HostId>,
    parent: Option<HostId>,
    handler_owner: usize,
    element: Option<Element>,
    virtual_list: Option<VirtualList>,
    dirty: bool,
}

/// Owns a validated presentation graph feeding the runtime's canonical tree.
pub struct Host {
    registry: SchemaRegistry,
    nodes: HashMap<HostId, HostNode>,
    slots: HashMap<u32, HostId>,
    generations: HashMap<u32, u32>,
    owners: HashMap<usize, HostId>,
    next_owner: usize,
    root: Option<HostId>,
    element: Option<Element>,
}

impl Host {
    /// Creates an empty host using `registry` as its only primitive contract.
    #[must_use]
    pub fn new(registry: SchemaRegistry) -> Self {
        Self {
            registry,
            nodes: HashMap::new(),
            slots: HashMap::new(),
            generations: HashMap::new(),
            owners: HashMap::new(),
            next_owner: 1,
            root: None,
            element: None,
        }
    }

    /// Creates a host with the built-in native primitives.
    ///
    /// # Errors
    ///
    /// Returns a schema error if registration of the built-ins fails.
    pub fn with_builtins() -> Result<Self, SchemaError> {
        Ok(Self::new(builtin::registry()?))
    }

    /// Returns the validated root for the runtime's [`argui_ui::UiTree`].
    #[must_use]
    pub fn root_element(&self) -> Option<Element> {
        self.element.clone()
    }

    /// Returns the current presentation root identity, if one exists.
    #[must_use]
    pub const fn root_id(&self) -> Option<HostId> {
        self.root
    }

    /// Returns the canonical schema fingerprint used by presentation adapters.
    #[must_use]
    pub fn abi_hash(&self) -> u64 {
        self.registry.abi_hash()
    }

    /// Returns the canonical native schema registry for contract export.
    #[must_use]
    pub const fn registry(&self) -> &SchemaRegistry {
        &self.registry
    }

    /// Validates and applies `operations` as one atomic native tree update.
    ///
    /// The entire batch is checked before changing the visible root. Detached
    /// nodes may be constructed ahead of a later `SetRoot` operation.
    ///
    /// # Errors
    ///
    /// Returns a host or schema error for an invalid identity, structure, or
    /// native property; no state changes in that case.
    pub fn commit(&mut self, operations: &[Operation]) -> Result<CommitResult, HostError> {
        let (changes, slots, generations, owners, next_owner, root, element) = {
            let mut stage = Stage::new(self);
            for operation in operations {
                stage.apply(operation)?;
            }
            let changed_ids = stage
                .changes
                .iter()
                .filter_map(|(id, node)| node.as_ref().map(|_| *id))
                .collect::<Vec<_>>();
            for id in changed_ids {
                stage.materialize(id)?;
            }
            let element = stage.root.map(|id| stage.materialize(id)).transpose()?;
            (
                stage.changes,
                stage.slots,
                stage.generations,
                stage.owners,
                stage.next_owner,
                stage.root,
                element,
            )
        };
        let changed_nodes = changes.len();
        let playback_nodes = changes
            .iter()
            .filter_map(|(id, node)| node.as_ref().map(|_| *id))
            .collect::<Vec<_>>();
        let visible_changed = match (&self.element, &element) {
            (Some(previous), Some(current)) => !previous.ptr_eq(current),
            (None, None) => false,
            _ => true,
        };
        for (id, node) in changes {
            match node {
                Some(node) => {
                    self.nodes.insert(id, node);
                }
                None => {
                    self.nodes.remove(&id);
                }
            }
        }
        for (slot, id) in slots {
            match id {
                Some(id) => {
                    self.slots.insert(slot, id);
                }
                None => {
                    self.slots.remove(&slot);
                }
            }
        }
        self.generations.extend(generations);
        for (owner, id) in owners {
            match id {
                Some(id) => {
                    self.owners.insert(owner, id);
                }
                None => {
                    self.owners.remove(&owner);
                }
            }
        }
        self.next_owner = next_owner;
        self.root = root;
        self.element = element;
        for id in playback_nodes {
            self.sync_loop_playback(id);
        }
        Ok(CommitResult {
            visible_changed,
            changed_nodes,
        })
    }

    /// Applies accepted playback controls to the mounted native motions of `id`.
    fn sync_loop_playback(&self, id: HostId) {
        let Some(node) = self
            .nodes
            .get(&id)
            .filter(|node| node.native_type == builtin::RECTANGLE)
        else {
            return;
        };
        let playing = !matches!(
            node.properties.get(&builtin::LOOP_PLAYING),
            Some(SchemaValue::Bool(false))
        );
        if let Some(element) = &node.element {
            for binding in &element.bindings {
                if playing {
                    binding.track().resume();
                } else {
                    binding.track().pause();
                }
            }
        }
    }

    /// Resolves a native event to a still-mounted JavaScript callback.
    ///
    /// `event` is a delivery emitted by the current [`argui_ui::UiTree`]. Stale events
    /// from removed nodes or replaced listeners return `None`.
    #[must_use]
    pub fn callback_for(&self, event: &UiEvent) -> Option<CallbackDelivery> {
        let handler = event.current_handler()?;
        let node_id = *self.owners.get(&handler.owner().0)?;
        let node = self.nodes.get(&node_id)?;
        let callback = CallbackId(handler.slot());
        // Adapters can bind one declared callback to several native event kinds.
        // The UiTree delivery identifies the handler; only its live registration
        // must be checked here.
        node.listeners
            .iter()
            .any(|(_, registered)| *registered == callback)
            .then_some(CallbackDelivery {
                node: node_id,
                callback,
            })
    }
}

struct Stage<'a> {
    host: &'a Host,
    changes: HashMap<HostId, Option<HostNode>>,
    slots: HashMap<u32, Option<HostId>>,
    generations: HashMap<u32, u32>,
    owners: HashMap<usize, Option<HostId>>,
    next_owner: usize,
    root: Option<HostId>,
}

impl<'a> Stage<'a> {
    fn new(host: &'a Host) -> Self {
        Self {
            host,
            changes: HashMap::new(),
            slots: HashMap::new(),
            generations: HashMap::new(),
            owners: HashMap::new(),
            next_owner: host.next_owner,
            root: host.root,
        }
    }

    fn get(&self, id: HostId) -> Result<&HostNode, HostError> {
        self.changes
            .get(&id)
            .map_or_else(|| self.host.nodes.get(&id), Option::as_ref)
            .ok_or(HostError::UnknownId(id))
    }

    fn get_mut(&mut self, id: HostId) -> Result<&mut HostNode, HostError> {
        if !self.changes.contains_key(&id) {
            self.changes.insert(id, Some(self.get(id)?.clone()));
        }
        self.changes
            .get_mut(&id)
            .and_then(Option::as_mut)
            .ok_or(HostError::UnknownId(id))
    }

    /// Invalidates `id` and its ancestors until reaching one already invalidated.
    ///
    /// Earlier operations in the same transaction have already invalidated
    /// every ancestor of a dirty node, so repeated properties need no walk.
    /// Returns an error only if the node or an ancestor is absent.
    fn mark_dirty(&mut self, mut id: HostId) -> Result<(), HostError> {
        loop {
            let node = self.get_mut(id)?;
            if node.dirty {
                return Ok(());
            }
            node.dirty = true;
            node.element = None;
            match node.parent {
                Some(parent) => id = parent,
                None => return Ok(()),
            }
        }
    }

    fn apply(&mut self, operation: &Operation) -> Result<(), HostError> {
        match operation {
            Operation::Create { id, native_type } => self.create(*id, *native_type),
            Operation::SetProperty {
                id,
                property,
                value,
            } => {
                let node = self.get_mut(*id)?;
                match value {
                    Some(value) => {
                        node.properties.insert(*property, value.clone());
                    }
                    None => {
                        node.properties.remove(property);
                    }
                }
                self.mark_dirty(*id)
            }
            Operation::SetListener {
                id,
                event,
                callback,
            } => {
                let node = self.get_mut(*id)?;
                match callback {
                    Some(callback) => {
                        node.listeners.insert(*event, *callback);
                    }
                    None => {
                        node.listeners.remove(event);
                    }
                }
                self.mark_dirty(*id)
            }
            Operation::Insert {
                parent,
                child,
                before,
            } => self.insert(*parent, *child, *before),
            Operation::Remove { id } => self.remove(*id),
            Operation::SetRoot { id } => {
                if let Some(id) = id
                    && self.get(*id)?.parent.is_some()
                {
                    return Err(HostError::AttachedRoot(*id));
                }
                self.root = *id;
                Ok(())
            }
        }
    }

    fn create(&mut self, id: HostId, native_type: NativeTypeId) -> Result<(), HostError> {
        if id.slot() == 0 || id.generation() == 0 {
            return Err(HostError::InvalidId(id));
        }
        let live = self
            .slots
            .get(&id.slot())
            .copied()
            .unwrap_or_else(|| self.host.slots.get(&id.slot()).copied());
        let previous = self
            .generations
            .get(&id.slot())
            .copied()
            .or_else(|| self.host.generations.get(&id.slot()).copied())
            .unwrap_or(0);
        if live.is_some() || id.generation() <= previous {
            return Err(HostError::StaleId(id));
        }
        if self.host.registry.schema(native_type).is_none() {
            return Err(HostError::UnknownType(native_type));
        }
        let owner = self.next_owner;
        self.next_owner = self
            .next_owner
            .checked_add(1)
            .ok_or(HostError::HandlerExhausted)?;
        self.changes.insert(
            id,
            Some(HostNode {
                native_type,
                properties: BTreeMap::new(),
                listeners: BTreeMap::new(),
                children: Vec::new(),
                parent: None,
                handler_owner: owner,
                element: None,
                virtual_list: None,
                dirty: true,
            }),
        );
        self.slots.insert(id.slot(), Some(id));
        self.generations.insert(id.slot(), id.generation());
        self.owners.insert(owner, Some(id));
        Ok(())
    }

    fn insert(
        &mut self,
        parent: HostId,
        child: HostId,
        before: Option<HostId>,
    ) -> Result<(), HostError> {
        let schema = self
            .host
            .registry
            .schema(self.get(parent)?.native_type)
            .ok_or(HostError::UnknownId(parent))?;
        if !schema.slots.iter().any(|slot| slot.id == builtin::CHILDREN) {
            return Err(HostError::NoChildren(parent));
        }
        self.get(child)?;
        let mut cursor = Some(parent);
        while let Some(id) = cursor {
            if id == child {
                return Err(HostError::Cycle(child));
            }
            cursor = self.get(id)?.parent;
        }
        if let Some(before) = before
            && (before == child || self.get(before)?.parent != Some(parent))
        {
            return Err(HostError::InvalidBefore(before, parent));
        }
        if let Some(old_parent) = self.get(child)?.parent {
            self.get_mut(old_parent)?.children.retain(|id| *id != child);
            self.mark_dirty(old_parent)?;
        } else if self.root == Some(child) {
            self.root = None;
        }
        let children = &mut self.get_mut(parent)?.children;
        let index = before
            .and_then(|id| children.iter().position(|candidate| *candidate == id))
            .unwrap_or(children.len());
        children.insert(index, child);
        self.get_mut(child)?.parent = Some(parent);
        self.mark_dirty(parent)
    }

    fn remove(&mut self, id: HostId) -> Result<(), HostError> {
        let parent = self.get(id)?.parent;
        if let Some(parent) = parent {
            self.get_mut(parent)?.children.retain(|child| *child != id);
            self.mark_dirty(parent)?;
        }
        if self.root == Some(id) {
            self.root = None;
        }
        let mut pending = vec![id];
        while let Some(current) = pending.pop() {
            let node = self.get(current)?.clone();
            pending.extend(node.children);
            self.changes.insert(current, None);
            self.slots.insert(current.slot(), None);
            self.owners.insert(node.handler_owner, None);
        }
        Ok(())
    }

    /// Builds the element for `id` from its current properties and descendants.
    ///
    /// Reuses an existing rectangle only when its materialized children retain
    /// the same identities, so a descendant edit reaches the visible root.
    ///
    /// Returns the built or retained element. Errors if `id` or a descendant is
    /// unknown, or if its native schema rejects the current values.
    fn materialize(&mut self, id: HostId) -> Result<Element, HostError> {
        let node = self.get(id)?.clone();
        if !node.dirty
            && let Some(element) = node.element
        {
            return Ok(element);
        }
        let children = node
            .children
            .iter()
            .map(|child| {
                let child_node = self.get(*child)?;
                if child_node.native_type == builtin::TEXT
                    && matches!(child_node.properties.get(&builtin::TEXT_VALUE), Some(SchemaValue::String(value)) if value.is_empty())
                {
                    Ok(None)
                } else {
                    self.materialize(*child).map(Some)
                }
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let mut input = NativeElementInput::new();
        let reset_virtual_measurements = self.host.nodes.get(&id).is_some_and(|previous| {
            previous.properties.get(&builtin::VIRTUAL_DATA_VERSION)
                != node.properties.get(&builtin::VIRTUAL_DATA_VERSION)
        });
        let virtual_list = virtual_list::retained(&node, reset_virtual_measurements);
        input.virtual_list = virtual_list.clone();
        for (property, value) in &node.properties {
            input.properties.push((*property, value.clone()));
        }
        for (event, callback) in &node.listeners {
            input.events.push(NativeEventValue::new(
                *event,
                EventHandler::from_identity(EventHandlerId::new(
                    EventOwnerId(node.handler_owner),
                    callback.0,
                )),
            ));
        }
        if !children.is_empty() {
            input
                .slots
                .push(NativeSlotValue::new(builtin::CHILDREN, children));
        }
        let constructed = self.host.registry.construct(node.native_type, &input)?;
        let reusable = self.host.nodes.get(&id).filter(|previous| {
            node.native_type == builtin::RECTANGLE
                && previous.element.is_some()
                && previous.native_type == node.native_type
                && previous.parent == node.parent
                && previous.children == node.children
                && previous.listeners == node.listeners
                && previous
                    .element
                    .as_ref()
                    .is_some_and(|element| descendant::same_children(element, &constructed))
                && previous
                    .properties
                    .iter()
                    .filter(|(property, _)| **property != builtin::LOOP_PLAYING)
                    .eq(node
                        .properties
                        .iter()
                        .filter(|(property, _)| **property != builtin::LOOP_PLAYING))
        });
        let element = if let Some(previous) = reusable {
            previous
                .element
                .clone()
                .expect("filtered element is present")
        } else {
            let mut element = constructed.retained_identity(RetainedIdentity::new(
                0,
                (u64::from(id.slot()) << 32) | u64::from(id.generation()),
            ));
            if element.key.is_none() {
                element = element.keyed(format!("host:{}:{}", id.slot(), id.generation()));
            }
            element
        };
        let entry = self.get_mut(id)?;
        entry.element = Some(element.clone());
        entry.virtual_list = virtual_list;
        entry.dirty = false;
        Ok(element)
    }
}
