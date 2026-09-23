//! Presentation-scoped reuse of unchanged native element subtrees.

use std::collections::HashMap;

use argui_ui::{Element, RetainedIdentity};

use crate::{NativeElementInput, NativeTypeId, SchemaError, SchemaRegistry};

struct Entry {
    native: NativeTypeId,
    input: NativeElementInput,
    interactive: bool,
    element: Element,
    generation: u64,
}

/// Reuses native subtrees while inputs, children and handler identities match.
/// Own one cache per presentation and clear it when replacing the native registry.
/// Completed render passes retain only mounted sites, bounding cache lifetime.
#[derive(Default)]
pub struct NativeElementCache {
    entries: HashMap<RetainedIdentity, Entry>,
    generation: u64,
}

impl NativeElementCache {
    /// Returns an empty presentation cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Begins a render pass; constructed or reused sites stay mounted until its end.
    pub fn begin_render(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// Releases inputs, handlers and subtrees absent from this completed render pass.
    pub fn end_render(&mut self) {
        self.entries
            .retain(|_, entry| entry.generation == self.generation);
    }

    /// Drops all retained elements, for example when a native registry is replaced.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns the number of retained source sites.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether no native elements are retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Reuses or builds `native` at `identity` using `registry` and owned `input`.
    /// `interactive` adds hit testing for observed style states. Returns an element
    /// sharing its retained subtree when every input matches the previous render.
    ///
    /// # Errors
    /// Returns schema validation or adapter errors without replacing a valid entry.
    pub fn construct(
        &mut self,
        registry: &SchemaRegistry,
        native: NativeTypeId,
        identity: RetainedIdentity,
        input: NativeElementInput,
        interactive: bool,
    ) -> Result<Element, SchemaError> {
        if let Some(entry) = self.entries.get_mut(&identity)
            && entry.native == native
            && entry.interactive == interactive
            && entry.input == input
        {
            entry.generation = self.generation;
            return Ok(entry.element.clone());
        }
        let mut element = registry.construct(native, &input)?;
        if interactive {
            element.interaction.get_or_insert_with(Default::default);
        }
        let element = element.retained_identity(identity.clone());
        self.entries.insert(
            identity,
            Entry {
                native,
                input,
                interactive,
                element: element.clone(),
                generation: self.generation,
            },
        );
        Ok(element)
    }
}
