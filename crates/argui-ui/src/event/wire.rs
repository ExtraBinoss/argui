//! Stable event kind names shared by native and Web delivery.

impl super::EventType {
    /// Returns the stable lower camel case name used by native and Web event payloads.
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Action => "action",
            Self::PointerEnter => "pointerEnter",
            Self::PointerLeave => "pointerLeave",
            Self::PointerMove => "pointerMove",
            Self::PointerDown => "pointerDown",
            Self::PointerOutside => "pointerOutside",
            Self::Dismiss => "dismiss",
            Self::PointerUp => "pointerUp",
            Self::PointerCancel => "pointerCancel",
            Self::Click => "click",
            Self::ContextMenu => "contextMenu",
            Self::GotPointerCapture => "gotPointerCapture",
            Self::LostPointerCapture => "lostPointerCapture",
            Self::Key => "key",
            Self::Wheel => "wheel",
            Self::Scroll => "scroll",
            Self::VirtualMeasure => "measure",
            Self::VirtualWindow => "window",
            Self::Focus => "focus",
            Self::Blur => "blur",
            Self::Input => "input",
            Self::TextEdit => "edit",
            Self::Submit => "submit",
            Self::Gesture => "gesture",
            Self::SemanticAction => "semanticAction",
            Self::SelectionChange => "selectionChange",
        }
    }
}
