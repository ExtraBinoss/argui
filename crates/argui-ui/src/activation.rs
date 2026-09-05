use argui_core::{KeyInput, Modifiers, Point, PointerEvent, PointerId};

#[derive(Clone, Debug, PartialEq)]
pub enum ActivationSource {
    Pointer(PointerEvent),
    Keyboard(KeyInput),
    Accessibility,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClickEvent {
    pub source: ActivationSource,
    pub count: u8,
}

impl ClickEvent {
    #[must_use]
    pub const fn pointer(event: PointerEvent, count: u8) -> Self {
        Self {
            source: ActivationSource::Pointer(event),
            count,
        }
    }

    #[must_use]
    pub const fn keyboard(input: KeyInput) -> Self {
        Self {
            source: ActivationSource::Keyboard(input),
            count: 0,
        }
    }

    #[must_use]
    pub const fn accessibility() -> Self {
        Self {
            source: ActivationSource::Accessibility,
            count: 0,
        }
    }

    #[must_use]
    pub const fn pointer_id(&self) -> Option<PointerId> {
        match &self.source {
            ActivationSource::Pointer(event) => Some(event.id),
            ActivationSource::Keyboard(_) | ActivationSource::Accessibility => None,
        }
    }

    #[must_use]
    pub const fn position(&self) -> Option<Point> {
        match &self.source {
            ActivationSource::Pointer(event) => Some(event.position),
            ActivationSource::Keyboard(_) | ActivationSource::Accessibility => None,
        }
    }

    #[must_use]
    pub const fn modifiers(&self) -> Modifiers {
        match &self.source {
            ActivationSource::Pointer(event) => event.modifiers,
            ActivationSource::Keyboard(input) => input.modifiers,
            ActivationSource::Accessibility => Modifiers {
                shift: false,
                control: false,
                alt: false,
                super_key: false,
            },
        }
    }
}
