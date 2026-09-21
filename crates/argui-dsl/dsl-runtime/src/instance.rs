use std::{cell::RefCell, collections::HashMap, rc::Rc};

use argui_dsl_ir::{
    CallbackId, ComponentId, ExpressionId, IrStatement, IrType, LocalId, PropertyId,
};

use crate::{DslValue, RuntimeError};

/// Stable process-local live component-instance identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InstanceId(u64);

impl InstanceId {
    /// Creates a stable instance ID from its runtime representation.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the runtime representation.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Heterogeneous typed property stored only in the live DSL runtime.
#[derive(Clone, Debug, PartialEq)]
pub struct DynamicProperty {
    pub id: PropertyId,
    pub value_type: IrType,
    value: DslValue,
    revision: u64,
    modified: bool,
}

impl DynamicProperty {
    /// Creates a typed property at revision zero.
    #[must_use]
    pub fn new(id: PropertyId, value_type: IrType, value: DslValue) -> Self {
        Self {
            id,
            value_type,
            value,
            revision: 0,
            modified: false,
        }
    }

    /// Returns the current value.
    #[must_use]
    pub fn get(&self) -> &DslValue {
        &self.value
    }

    /// Returns the accepted-change revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Replaces a compatible value, marks it explicit, and reports whether it changed.
    ///
    /// # Errors
    ///
    /// Returns a type mismatch without changing the previous value.
    pub fn set(&mut self, value: DslValue) -> Result<bool, RuntimeError> {
        if !value.compatible_with(&self.value_type) {
            return Err(RuntimeError::TypeMismatch {
                expected: format!("{:?}", self.value_type),
                actual: value.type_name().into(),
            });
        }
        self.modified = true;
        if self.value == value {
            return Ok(false);
        }
        self.value = value;
        self.revision = self.revision.wrapping_add(1);
        Ok(true)
    }
}

pub(crate) type Callback = Rc<RefCell<Box<dyn FnMut(Vec<DslValue>) -> DslValue>>>;

/// Exact dependency revision set associated with one cached expression value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EvaluationStamp {
    pub properties: Vec<(PropertyId, u64)>,
    pub token_revision: u64,
}

/// One expression result reused while all captured inputs remain unchanged.
#[derive(Clone, Debug, PartialEq)]
struct CachedExpression {
    stamp: EvaluationStamp,
    value: DslValue,
}

/// Parent handler connected to one callback of a nested component instance.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EventRoute {
    pub parent: InstanceId,
    pub statements: Vec<IrStatement>,
    pub locals: HashMap<LocalId, DslValue>,
}

/// Reverse destination for one component property bound with `<=>`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PropertyLink {
    pub instance: InstanceId,
    pub property: PropertyId,
}

/// Stateful live instance preserved across compatible package generations.
#[derive(Clone)]
pub struct ComponentInstance {
    pub id: InstanceId,
    pub component: ComponentId,
    pub properties: HashMap<PropertyId, DynamicProperty>,
    pub(crate) callbacks: HashMap<CallbackId, Callback>,
    routes: HashMap<CallbackId, EventRoute>,
    links: HashMap<PropertyId, PropertyLink>,
    expression_cache: HashMap<ExpressionId, CachedExpression>,
}

impl std::fmt::Debug for ComponentInstance {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ComponentInstance")
            .field("id", &self.id)
            .field("component", &self.component)
            .field("properties", &self.properties)
            .field("callback_count", &self.callbacks.len())
            .finish()
    }
}

impl ComponentInstance {
    /// Creates an instance from prepared initial property values.
    #[must_use]
    pub fn new(
        id: InstanceId,
        component: ComponentId,
        properties: impl IntoIterator<Item = DynamicProperty>,
    ) -> Self {
        Self {
            id,
            component,
            properties: properties
                .into_iter()
                .map(|property| (property.id, property))
                .collect(),
            callbacks: HashMap::new(),
            routes: HashMap::new(),
            links: HashMap::new(),
            expression_cache: HashMap::new(),
        }
    }

    /// Invalidates memoized expressions after an external evaluation-context change.
    pub(crate) fn clear_expression_cache(&mut self) {
        self.expression_cache.clear();
    }

    /// Binds a typed-erased callback to a resolved callback ID.
    pub fn bind_callback(
        &mut self,
        id: CallbackId,
        callback: impl FnMut(Vec<DslValue>) -> DslValue + 'static,
    ) {
        self.callbacks
            .insert(id, Rc::new(RefCell::new(Box::new(callback))));
    }

    /// Invokes an externally bound callback.
    pub(crate) fn invoke_callback(
        &mut self,
        id: CallbackId,
        arguments: Vec<DslValue>,
    ) -> Option<DslValue> {
        self.callbacks
            .get(&id)
            .map(|callback| (callback.borrow_mut())(arguments))
    }

    /// Returns a clonable external callback handle without holding an instance borrow.
    pub(crate) fn callback(&self, id: CallbackId) -> Option<Callback> {
        self.callbacks.get(&id).cloned()
    }

    /// Replaces or clears the parent route for one nested callback.
    pub(crate) fn set_route(&mut self, id: CallbackId, route: Option<EventRoute>) {
        if let Some(route) = route {
            self.routes.insert(id, route);
        } else {
            self.routes.remove(&id);
        }
    }

    /// Returns a cloned callback route for re-entrant dispatch.
    pub(crate) fn route(&self, id: CallbackId) -> Option<EventRoute> {
        self.routes.get(&id).cloned()
    }

    /// Replaces or clears one reverse two-way property link.
    pub(crate) fn set_link(&mut self, id: PropertyId, link: Option<PropertyLink>) {
        if let Some(link) = link {
            self.links.insert(id, link);
        } else {
            self.links.remove(&id);
        }
    }

    /// Returns the reverse destination of one two-way property.
    pub(crate) fn link(&self, id: PropertyId) -> Option<PropertyLink> {
        self.links.get(&id).copied()
    }

    /// Returns a cached expression only when its exact dependency stamp matches.
    pub(crate) fn cached_expression(
        &self,
        id: ExpressionId,
        stamp: &EvaluationStamp,
    ) -> Option<DslValue> {
        self.expression_cache
            .get(&id)
            .filter(|cached| cached.stamp == *stamp)
            .map(|cached| cached.value.clone())
    }

    /// Stores a clean expression result under its exact dependency stamp.
    pub(crate) fn cache_expression(
        &mut self,
        id: ExpressionId,
        stamp: EvaluationStamp,
        value: DslValue,
    ) {
        self.expression_cache
            .insert(id, CachedExpression { stamp, value });
    }

    /// Migrates explicitly assigned compatible properties and callbacks into a new definition.
    ///
    /// Unmodified properties keep the replacement's freshly evaluated defaults.
    /// `replacement` is the freshly initialized instance for the next generation.
    pub(crate) fn migrate_into(self, replacement: &mut Self) {
        for (id, previous) in self.properties {
            if previous.modified
                && let Some(next) = replacement.properties.get_mut(&id)
                && previous.value.compatible_with(&next.value_type)
            {
                next.value = previous.value;
                next.revision = previous.revision;
                next.modified = true;
            }
        }
        replacement.callbacks = self.callbacks;
    }
}
