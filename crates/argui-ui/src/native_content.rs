use std::{any::Any, fmt, rc::Rc};

/// Opaque retained native content. The renderer does not interpret its payload.
/// Identity is scoped to the payload type and must remain unique within that type.
#[derive(Clone)]
pub struct NativeContent {
    id: u64,
    payload: Rc<dyn Any>,
}

impl NativeContent {
    /// Retains an opaque native payload under an application-provided identity.
    ///
    /// * `id` — stable identity, unique among payloads of the same type.
    /// * `payload` — value retained for native integration code.
    #[must_use]
    pub fn new<T: Any>(id: u64, payload: T) -> Self {
        Self {
            id,
            payload: Rc::new(payload),
        }
    }
    /// Returns a reference to the payload when it has type `T`.
    #[must_use]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.payload.downcast_ref()
    }
    /// Returns this payload's application-provided identity.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }
}

impl PartialEq for NativeContent {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.payload.as_ref().type_id() == other.payload.as_ref().type_id()
    }
}

impl fmt::Debug for NativeContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeContent")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl crate::Element {
    /// Attaches opaque native content to this element.
    #[must_use]
    pub fn native_content(mut self, content: NativeContent) -> Self {
        self.native_content = Some(content);
        self
    }
}
