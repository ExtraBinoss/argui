//! Unpainted pointer interaction primitive for declarative composition.

use argui_ui::{
    CursorIcon, Element, EventFilter, EventType, GestureCapture, GestureDelivery, GestureSet,
    HitTestStyle, Interaction, PanGesture, PointerEvents, StateScopeId, TapGesture, UserSelect,
};

use super::{
    CHILDREN, CLICK, CONTEXT_MENU, CommonProperty, DOUBLE_CLICK, DRAG_X, DRAG_Y, ENABLED,
    HAS_HOVER, MOUSE_CURSOR, MOUSE_GLOBAL_X, MOUSE_GLOBAL_Y, MOUSE_X, MOUSE_Y, MOVED,
    POINTER_CANCEL, POINTER_DOWN, POINTER_ENTER, POINTER_LEAVE, POINTER_MOVE, POINTER_UP, PRESSED,
    PRESSED_X, PRESSED_Y, WHEEL, apply_common, common_event, common_property, optional_bool,
};
use crate::{
    EventSchema, NativeElementInput, NativeSchema, ObservationKind, PropertySchema, SchemaError,
    SchemaRegistry, SchemaValue, SlotArity, SlotSchema, ValueType,
};

/// Scope consumed by visual descendants for pointer hover and press styles.
pub(super) const TOUCH_AREA_SCOPE: &str = "argui.touch-area";

/// Registers a paint-free hit region with read-only pointer state and events.
///
/// * `registry` — registry receiving the TouchArea contract and adapter.
///
/// # Errors
///
/// Returns a schema error if built-in identifiers or metadata conflict.
pub(super) fn register(registry: &mut SchemaRegistry) -> Result<(), SchemaError> {
    let schema = NativeSchema::new(
        super::TOUCH_AREA,
        "TouchArea",
        "Pointer hit region with engine-observed state and no prescribed visual style.",
    )
    .property(common_property(CommonProperty::Key))
    .property(common_property(CommonProperty::Width))
    .property(common_property(CommonProperty::Height))
    .property(common_property(CommonProperty::Position))
    .property(common_property(CommonProperty::Inset))
    .property(common_property(CommonProperty::Rotation))
    .property(common_property(CommonProperty::Opacity))
    .property(common_property(CommonProperty::BackdropFilter))
    .property(common_property(CommonProperty::DesktopBackdropTint))
    .property(common_property(CommonProperty::DesktopBackdropFallback))
    .property(common_property(CommonProperty::Visible))
    .property(
        PropertySchema::new(ENABLED, "enabled", ValueType::Bool, "Accept pointer input.")
            .default_value(SchemaValue::Bool(true)),
    )
    .property(PropertySchema::new(
        MOUSE_CURSOR,
        "mouseCursor",
        ValueType::String,
        "Cursor shown while hovered.",
    ))
    .property(
        PropertySchema::new(
            HAS_HOVER,
            "hasHover",
            ValueType::Bool,
            "Pointer is over the area.",
        )
        .observed(ObservationKind::Hover),
    )
    .property(
        PropertySchema::new(
            PRESSED,
            "pressed",
            ValueType::Bool,
            "Primary pointer is pressed.",
        )
        .observed(ObservationKind::Pressed),
    )
    .property(
        PropertySchema::new(
            MOUSE_X,
            "mouseX",
            ValueType::Dimension,
            "Current local pointer x coordinate.",
        )
        .observed(ObservationKind::PointerX),
    )
    .property(
        PropertySchema::new(
            MOUSE_Y,
            "mouseY",
            ValueType::Dimension,
            "Current local pointer y coordinate.",
        )
        .observed(ObservationKind::PointerY),
    )
    .property(
        PropertySchema::new(
            MOUSE_GLOBAL_X,
            "mouseGlobalX",
            ValueType::Dimension,
            "Current pointer x in window logical pixels, stable while dragging.",
        )
        .observed(ObservationKind::PointerGlobalX),
    )
    .property(
        PropertySchema::new(
            MOUSE_GLOBAL_Y,
            "mouseGlobalY",
            ValueType::Dimension,
            "Current pointer y in window logical pixels, stable while dragging.",
        )
        .observed(ObservationKind::PointerGlobalY),
    )
    .property(
        PropertySchema::new(
            PRESSED_X,
            "pressedX",
            ValueType::Dimension,
            "Local x coordinate of the most recent press.",
        )
        .observed(ObservationKind::PressedX),
    )
    .property(
        PropertySchema::new(
            PRESSED_Y,
            "pressedY",
            ValueType::Dimension,
            "Local y coordinate of the most recent press.",
        )
        .observed(ObservationKind::PressedY),
    )
    .event(common_event(CLICK, "click", EventType::Click))
    .event(common_event(
        CONTEXT_MENU,
        "contextMenu",
        EventType::ContextMenu,
    ))
    .event(common_event(
        DOUBLE_CLICK,
        "doubleClicked",
        EventType::Click,
    ))
    .event(common_event(
        POINTER_ENTER,
        "pointerEnter",
        EventType::PointerEnter,
    ))
    .event(common_event(
        POINTER_LEAVE,
        "pointerLeave",
        EventType::PointerLeave,
    ))
    .event(common_event(
        POINTER_DOWN,
        "pointerDown",
        EventType::PointerDown,
    ))
    .event(common_event(POINTER_UP, "pointerUp", EventType::PointerUp))
    .event(common_event(
        POINTER_MOVE,
        "pointerMove",
        EventType::PointerMove,
    ))
    .event(common_event(
        POINTER_CANCEL,
        "pointerCancel",
        EventType::PointerCancel,
    ))
    .event(common_event(MOVED, "moved", EventType::PointerMove))
    .event(
        EventSchema::new(
            DRAG_X,
            "dragX",
            EventType::Gesture,
            "Frame-coalesced horizontal pan displacement in logical pixels.",
        )
        .payload(ValueType::Float),
    )
    .event(
        EventSchema::new(
            DRAG_Y,
            "dragY",
            EventType::Gesture,
            "Frame-coalesced vertical pan displacement in logical pixels.",
        )
        .payload(ValueType::Float),
    )
    .event(common_event(WHEEL, "wheel", EventType::Wheel))
    .slot(SlotSchema {
        id: CHILDREN,
        name: "children".into(),
        arity: SlotArity::Many,
        documentation: "Visual descendants of the hit region.".into(),
    });
    registry.register(schema, |input: &NativeElementInput| {
        let cursor = match input.get(MOUSE_CURSOR) {
            Some(SchemaValue::String(name)) => parse_cursor(name)?,
            _ => CursorIcon::Auto,
        };
        let mut gestures = GestureSet::default().tap(TapGesture::default());
        if input
            .events
            .iter()
            .any(|event| event.id == DRAG_X || event.id == DRAG_Y)
        {
            gestures = gestures.pan(
                PanGesture::default()
                    .immediate()
                    .capture(GestureCapture::OnPress)
                    .delivery(GestureDelivery::FrameCoalesced),
            );
        }
        // Ordinary taps leave touch scrolling available to an ancestor viewport.
        // Pressed-pointer movement and explicit pans retain their drag capture.
        let captures_drag =
            gestures.captures_on_press() || input.events.iter().any(|event| event.id == MOVED);
        let interaction = Interaction::default()
            .enabled(optional_bool(input, ENABLED).unwrap_or(true))
            .cursor(cursor)
            .gestures(gestures)
            .capture_on_press(captures_drag);
        let mut element =
            apply_common(Element::container(input.children(CHILDREN).to_vec()), input)?
                .interaction(interaction)
                .user_select(UserSelect::None)
                .state_scope(StateScopeId::new(TOUCH_AREA_SCOPE))
                .hit_test(HitTestStyle::default().pointer_events(PointerEvents::BoxOnly));
        for event in &input.events {
            let event_type = match event.id {
                CLICK | DOUBLE_CLICK => EventType::Click,
                CONTEXT_MENU => EventType::ContextMenu,
                POINTER_ENTER => EventType::PointerEnter,
                POINTER_LEAVE => EventType::PointerLeave,
                POINTER_DOWN => EventType::PointerDown,
                POINTER_UP => EventType::PointerUp,
                POINTER_MOVE | MOVED => EventType::PointerMove,
                POINTER_CANCEL => EventType::PointerCancel,
                WHEEL => EventType::Wheel,
                DRAG_X | DRAG_Y => EventType::Gesture,
                _ => continue,
            };
            let filter = match event.id {
                DOUBLE_CLICK => EventFilter::DoubleClick,
                MOVED => EventFilter::PressedPointerMove,
                DRAG_X | DRAG_Y => EventFilter::Pan,
                _ => EventFilter::Any,
            };
            element = element.on(event.handler.direct_listener(event_type).filter(filter));
        }
        Ok(element)
    })
}

/// Parses a cursor name accepted by the engine's generic cursor enumeration.
///
/// * `name` — cursor spelling from a validated string property.
///
/// # Errors
///
/// Returns an adapter error when the name is not a supported cursor.
fn parse_cursor(name: &str) -> Result<CursorIcon, SchemaError> {
    let cursor = match name {
        "auto" => CursorIcon::Auto,
        "default" => CursorIcon::Default,
        "contextMenu" => CursorIcon::ContextMenu,
        "help" => CursorIcon::Help,
        "pointer" => CursorIcon::Pointer,
        "progress" => CursorIcon::Progress,
        "wait" => CursorIcon::Wait,
        "cell" => CursorIcon::Cell,
        "crosshair" => CursorIcon::Crosshair,
        "text" => CursorIcon::Text,
        "verticalText" => CursorIcon::VerticalText,
        "alias" => CursorIcon::Alias,
        "copy" => CursorIcon::Copy,
        "move" => CursorIcon::Move,
        "noDrop" => CursorIcon::NoDrop,
        "notAllowed" => CursorIcon::NotAllowed,
        "grab" => CursorIcon::Grab,
        "grabbing" => CursorIcon::Grabbing,
        "eResize" => CursorIcon::EResize,
        "nResize" => CursorIcon::NResize,
        "neResize" => CursorIcon::NeResize,
        "nwResize" => CursorIcon::NwResize,
        "sResize" => CursorIcon::SResize,
        "seResize" => CursorIcon::SeResize,
        "swResize" => CursorIcon::SwResize,
        "wResize" => CursorIcon::WResize,
        "ewResize" => CursorIcon::EwResize,
        "nsResize" => CursorIcon::NsResize,
        "neswResize" => CursorIcon::NeswResize,
        "nwseResize" => CursorIcon::NwseResize,
        "colResize" => CursorIcon::ColResize,
        "rowResize" => CursorIcon::RowResize,
        "allScroll" => CursorIcon::AllScroll,
        "zoomIn" => CursorIcon::ZoomIn,
        "zoomOut" => CursorIcon::ZoomOut,
        "dndAsk" => CursorIcon::DndAsk,
        "allResize" => CursorIcon::AllResize,
        _ => {
            return Err(SchemaError::Adapter(format!(
                "TouchArea does not support mouse_cursor `{name}`"
            )));
        }
    };
    Ok(cursor)
}
