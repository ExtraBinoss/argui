//! Unpainted pointer interaction primitive for declarative composition.

use argui_ui::{
    CursorIcon, Element, EventFilter, EventType, GestureCapture, GestureDelivery, GestureSet,
    HitTestStyle, Interaction, PanGesture, PointerEvents, TapGesture, UserSelect,
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
    .property(common_property(CommonProperty::X))
    .property(common_property(CommonProperty::Y))
    .property(common_property(CommonProperty::Rotation))
    .property(common_property(CommonProperty::Opacity))
    .property(common_property(CommonProperty::BackdropFilter))
    .property(common_property(CommonProperty::Visible))
    .property(
        PropertySchema::new(ENABLED, "enabled", ValueType::Bool, "Accept pointer input.")
            .default_value(SchemaValue::Bool(true)),
    )
    .property(PropertySchema::new(
        MOUSE_CURSOR,
        "mouse_cursor",
        ValueType::String,
        "Cursor shown while hovered.",
    ))
    .property(
        PropertySchema::new(
            HAS_HOVER,
            "has_hover",
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
            "mouse_x",
            ValueType::Dimension,
            "Current local pointer x coordinate.",
        )
        .observed(ObservationKind::PointerX),
    )
    .property(
        PropertySchema::new(
            MOUSE_Y,
            "mouse_y",
            ValueType::Dimension,
            "Current local pointer y coordinate.",
        )
        .observed(ObservationKind::PointerY),
    )
    .property(
        PropertySchema::new(
            MOUSE_GLOBAL_X,
            "mouse_global_x",
            ValueType::Dimension,
            "Current pointer x in window logical pixels, stable while dragging.",
        )
        .observed(ObservationKind::PointerGlobalX),
    )
    .property(
        PropertySchema::new(
            MOUSE_GLOBAL_Y,
            "mouse_global_y",
            ValueType::Dimension,
            "Current pointer y in window logical pixels, stable while dragging.",
        )
        .observed(ObservationKind::PointerGlobalY),
    )
    .property(
        PropertySchema::new(
            PRESSED_X,
            "pressed_x",
            ValueType::Dimension,
            "Local x coordinate of the most recent press.",
        )
        .observed(ObservationKind::PressedX),
    )
    .property(
        PropertySchema::new(
            PRESSED_Y,
            "pressed_y",
            ValueType::Dimension,
            "Local y coordinate of the most recent press.",
        )
        .observed(ObservationKind::PressedY),
    )
    .event(common_event(CLICK, "click", EventType::Click))
    .event(common_event(
        CONTEXT_MENU,
        "context_menu",
        EventType::ContextMenu,
    ))
    .event(common_event(
        DOUBLE_CLICK,
        "double_clicked",
        EventType::Click,
    ))
    .event(common_event(
        POINTER_ENTER,
        "pointer_enter",
        EventType::PointerEnter,
    ))
    .event(common_event(
        POINTER_LEAVE,
        "pointer_leave",
        EventType::PointerLeave,
    ))
    .event(common_event(
        POINTER_DOWN,
        "pointer_down",
        EventType::PointerDown,
    ))
    .event(common_event(POINTER_UP, "pointer_up", EventType::PointerUp))
    .event(common_event(
        POINTER_MOVE,
        "pointer_move",
        EventType::PointerMove,
    ))
    .event(common_event(
        POINTER_CANCEL,
        "pointer_cancel",
        EventType::PointerCancel,
    ))
    .event(common_event(MOVED, "moved", EventType::PointerMove))
    .event(
        EventSchema::new(
            DRAG_X,
            "drag_x",
            EventType::Gesture,
            "Frame-coalesced horizontal pan displacement in logical pixels.",
        )
        .payload(ValueType::Float),
    )
    .event(
        EventSchema::new(
            DRAG_Y,
            "drag_y",
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
        let interaction = Interaction::default()
            .enabled(optional_bool(input, ENABLED).unwrap_or(true))
            .cursor(cursor)
            .gestures(gestures)
            .capture_on_press(true);
        let mut element =
            apply_common(Element::container(input.children(CHILDREN).to_vec()), input)?
                .interaction(interaction)
                .user_select(UserSelect::None)
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
        "context_menu" => CursorIcon::ContextMenu,
        "help" => CursorIcon::Help,
        "pointer" => CursorIcon::Pointer,
        "progress" => CursorIcon::Progress,
        "wait" => CursorIcon::Wait,
        "cell" => CursorIcon::Cell,
        "crosshair" => CursorIcon::Crosshair,
        "text" => CursorIcon::Text,
        "vertical_text" => CursorIcon::VerticalText,
        "alias" => CursorIcon::Alias,
        "copy" => CursorIcon::Copy,
        "move" => CursorIcon::Move,
        "no_drop" => CursorIcon::NoDrop,
        "not_allowed" => CursorIcon::NotAllowed,
        "grab" => CursorIcon::Grab,
        "grabbing" => CursorIcon::Grabbing,
        "e_resize" => CursorIcon::EResize,
        "n_resize" => CursorIcon::NResize,
        "ne_resize" => CursorIcon::NeResize,
        "nw_resize" => CursorIcon::NwResize,
        "s_resize" => CursorIcon::SResize,
        "se_resize" => CursorIcon::SeResize,
        "sw_resize" => CursorIcon::SwResize,
        "w_resize" => CursorIcon::WResize,
        "ew_resize" => CursorIcon::EwResize,
        "ns_resize" => CursorIcon::NsResize,
        "nesw_resize" => CursorIcon::NeswResize,
        "nwse_resize" => CursorIcon::NwseResize,
        "col_resize" => CursorIcon::ColResize,
        "row_resize" => CursorIcon::RowResize,
        "all_scroll" => CursorIcon::AllScroll,
        "zoom_in" => CursorIcon::ZoomIn,
        "zoom_out" => CursorIcon::ZoomOut,
        "dnd_ask" => CursorIcon::DndAsk,
        "all_resize" => CursorIcon::AllResize,
        _ => {
            return Err(SchemaError::Adapter(format!(
                "TouchArea does not support mouse_cursor `{name}`"
            )));
        }
    };
    Ok(cursor)
}
