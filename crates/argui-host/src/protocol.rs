use argui_schema::{EventId, NativeTypeId, PropertyId, SchemaValue};

/// Stable identity of a mounted presentation node, including its reuse generation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HostId {
    slot: u32,
    generation: u32,
}

impl HostId {
    /// Creates an ID for `slot` and its monotonically increasing `generation`.
    #[must_use]
    pub const fn new(slot: u32, generation: u32) -> Self {
        Self { slot, generation }
    }

    /// Returns the presentation-owned slot number.
    #[must_use]
    pub const fn slot(self) -> u32 {
        self.slot
    }

    /// Returns the slot's current reuse generation.
    #[must_use]
    pub const fn generation(self) -> u32 {
        self.generation
    }
}

/// Opaque callback identity owned by the JavaScript adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CallbackId(pub u32);

/// One validated operation in a native tree transaction.
#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    Create {
        id: HostId,
        native_type: NativeTypeId,
    },
    SetProperty {
        id: HostId,
        property: PropertyId,
        value: Option<SchemaValue>,
    },
    SetListener {
        id: HostId,
        event: EventId,
        callback: Option<CallbackId>,
    },
    Insert {
        parent: HostId,
        child: HostId,
        before: Option<HostId>,
    },
    Remove {
        id: HostId,
    },
    SetRoot {
        id: Option<HostId>,
    },
}

/// Live callback destination for a native event delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CallbackDelivery {
    pub node: HostId,
    pub callback: CallbackId,
}
