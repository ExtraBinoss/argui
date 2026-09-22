//! Conversion of engine interaction snapshots into typed DSL scalar values.

use argui_dsl_ir::IrObservation;
use argui_runtime::ObservedInteraction;
use argui_ui::VisualState;

use crate::DslValue;

/// Reads one observable native output from a retained interaction snapshot.
///
/// * `state` — host-provided hover, press, focus, and local and window pointer coordinates.
/// * `kind` — checked output operation carried by typed IR.
///
/// Returns a bool for visual state and a pixel length scalar for coordinates.
pub(crate) fn value(state: ObservedInteraction, kind: IrObservation) -> DslValue {
    match kind {
        IrObservation::Hover => DslValue::Bool(state.states.contains(VisualState::Hovered)),
        IrObservation::Pressed => DslValue::Bool(state.states.contains(VisualState::Pressed)),
        IrObservation::Focused => DslValue::Bool(state.states.contains(VisualState::Focused)),
        IrObservation::FocusVisible => {
            DslValue::Bool(state.states.contains(VisualState::FocusVisible))
        }
        IrObservation::PointerX => DslValue::Float(f64::from(
            state.pointer_position.map_or(0.0, |position| position.x),
        )),
        IrObservation::PointerY => DslValue::Float(f64::from(
            state.pointer_position.map_or(0.0, |position| position.y),
        )),
        IrObservation::PointerGlobalX => DslValue::Float(f64::from(
            state
                .pointer_global_position
                .map_or(0.0, |position| position.x),
        )),
        IrObservation::PointerGlobalY => DslValue::Float(f64::from(
            state
                .pointer_global_position
                .map_or(0.0, |position| position.y),
        )),
        IrObservation::PressedX => DslValue::Float(f64::from(
            state.pressed_position.map_or(0.0, |position| position.x),
        )),
        IrObservation::PressedY => DslValue::Float(f64::from(
            state.pressed_position.map_or(0.0, |position| position.y),
        )),
        IrObservation::ScrollX => DslValue::Float(f64::from(
            state.scroll.map_or(0.0, |scroll| scroll.offset.x),
        )),
        IrObservation::ScrollY => DslValue::Float(f64::from(
            state.scroll.map_or(0.0, |scroll| scroll.offset.y),
        )),
        IrObservation::ViewportWidth => DslValue::Float(f64::from(
            state.scroll.map_or(0.0, |scroll| scroll.viewport.width),
        )),
        IrObservation::ViewportHeight => DslValue::Float(f64::from(
            state.scroll.map_or(0.0, |scroll| scroll.viewport.height),
        )),
        IrObservation::ContentWidth => DslValue::Float(f64::from(
            state.scroll.map_or(0.0, |scroll| scroll.content.width),
        )),
        IrObservation::ContentHeight => DslValue::Float(f64::from(
            state.scroll.map_or(0.0, |scroll| scroll.content.height),
        )),
    }
}
