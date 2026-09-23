//! Native event bindings and two-way property updates.

use std::fmt::Write;

use argui_dsl_ir::{EventTargetId, IrPropertyBinding, PropertyTargetId};

use super::{statement_capture_names, statement_has_host_effect};
use crate::{
    CompilerError,
    codegen::{Context, expression::Scope},
};

impl Context<'_> {
    /// Emits native event registration and two-way payload propagation.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_native_events(
        &self,
        output: &mut String,
        schema: &argui_schema::NativeSchema,
        properties: &[IrPropertyBinding],
        events: &[argui_dsl_ir::IrEventBinding],
        input: &str,
        pad: &str,
        scope: &Scope,
    ) -> Result<(), CompilerError> {
        for event_schema in &schema.events {
            let explicit = events
                .iter()
                .find(|event| event.target == EventTargetId::Native(event_schema.id));
            let updates = properties
                .iter()
                .filter(|binding| binding.two_way)
                .filter_map(|binding| {
                    let PropertyTargetId::Native(property) = binding.target else {
                        return None;
                    };
                    schema
                        .properties
                        .iter()
                        .find(|candidate| {
                            candidate.id == property
                                && candidate.change_event == Some(event_schema.id)
                        })
                        .map(|property_schema| (binding, property_schema.value_type))
                })
                .collect::<Vec<_>>();
            if explicit.is_none() && updates.is_empty() {
                continue;
            }
            writeln!(
                output,
                "{pad}if let Some(register) = handlers.as_deref_mut() {{"
            )
            .unwrap();
            let update_properties = updates
                .iter()
                .filter_map(|(binding, _)| match binding.value.kind {
                    argui_dsl_ir::IrExpressionKind::PropertyRead(property) => Some(property),
                    _ => None,
                })
                .collect::<Vec<_>>();
            let statements = explicit
                .map(|binding| binding.statements.as_slice())
                .unwrap_or_default();
            let (captures, observed_sites) =
                statement_capture_names(scope, statements, &update_properties);
            for site in observed_sites {
                writeln!(
                    output,
                    "{pad}    observer.watch(&{});",
                    scope.observation_identity(site)
                )
                .unwrap();
            }
            for name in captures {
                writeln!(output, "{pad}    let {name} = {name}.clone();").unwrap();
            }
            if statements.iter().any(statement_has_host_effect) {
                writeln!(output, "{pad}    let host_effects = host_effects.clone();").unwrap();
            }
            writeln!(
                output,
                "{pad}    let handler = register(Box::new(move |event| {{"
            )
            .unwrap();
            writeln!(output, "{pad}        let _ = event;").unwrap();
            let mut handler_scope = scope.clone();
            if let Some(explicit) = explicit
                && let Some(parameter) = explicit.parameters.first()
            {
                let value = format!("_event_payload_{}", parameter.raw());
                let extraction = match event_schema.payload {
                        Some(argui_schema::ValueType::String) =>
                            "match &event.kind { ::argui::ui::UiEventKind::TextChanged(value) | ::argui::ui::UiEventKind::Submitted(value) => value.clone(), _ => return }".to_string(),
                        Some(argui_schema::ValueType::Float)
                            if event_schema.id == argui_schema::builtin::DRAG_X =>
                            "match &event.kind { ::argui::ui::UiEventKind::Gesture(gesture) => match gesture.kind { ::argui::ui::GestureKind::Pan { total, .. } => total.x, _ => return }, _ => return }".to_string(),
                        Some(argui_schema::ValueType::Float)
                            if event_schema.id == argui_schema::builtin::DRAG_Y =>
                            "match &event.kind { ::argui::ui::UiEventKind::Gesture(gesture) => match gesture.kind { ::argui::ui::GestureKind::Pan { total, .. } => total.y, _ => return }, _ => return }".to_string(),
                        Some(argui_schema::ValueType::Float) =>
                            "match &event.kind { ::argui::ui::UiEventKind::Scrolled { offset, .. } => offset.y, _ => return }".to_string(),
                        other => {
                            return Err(CompilerError::Codegen(format!(
                                "native event payload {other:?} has no AOT conversion"
                            )));
                        }
                    };
                writeln!(output, "{pad}        let {value} = {extraction};").unwrap();
                handler_scope.locals.insert(*parameter, value);
                handler_scope.local_types.insert(
                    *parameter,
                    match event_schema.payload {
                        Some(argui_schema::ValueType::String) => argui_dsl_ir::IrType::String,
                        Some(argui_schema::ValueType::Float) => argui_dsl_ir::IrType::Float,
                        _ => argui_dsl_ir::IrType::Unknown,
                    },
                );
            }
            for (binding, value_type) in updates {
                self.emit_two_way_update(output, binding, value_type, event_schema.id, pad, scope)?;
            }
            if let Some(explicit) = explicit {
                for statement in &explicit.statements {
                    writeln!(
                        output,
                        "{pad}        {}",
                        self.statement(statement, &mut handler_scope)?
                    )
                    .unwrap();
                }
            }
            writeln!(output, "{pad}    }}));").unwrap();
            writeln!(output, "{pad}    {input} = {input}.event(::argui::schema::NativeEventValue::new(::argui::schema::EventId::from_raw({}), handler));", event_schema.id.raw()).unwrap();
            writeln!(output, "{pad}}}").unwrap();
        }
        Ok(())
    }

    /// Emits the statically typed update for one schema two-way event payload.
    fn emit_two_way_update(
        &self,
        output: &mut String,
        binding: &IrPropertyBinding,
        value_type: argui_schema::ValueType,
        event_id: argui_schema::EventId,
        pad: &str,
        scope: &Scope,
    ) -> Result<(), CompilerError> {
        let argui_dsl_ir::IrExpressionKind::PropertyRead(property) = binding.value.kind else {
            return Err(CompilerError::Codegen(
                "native two-way source is not a property".into(),
            ));
        };
        let property = scope.properties.get(&property).ok_or_else(|| {
            CompilerError::Codegen("native two-way source is outside component scope".into())
        })?;
        match value_type {
            argui_schema::ValueType::String if event_id == argui_schema::builtin::TEXT_EDIT => {
                writeln!(output, "{pad}        let ::argui::ui::UiEventKind::TextEdited(edit) = &event.kind else {{ return }}; {property}.mutate(|value| {{ let Some(previous) = value.get(edit.range.clone()) else {{ return false }}; let changed = previous != edit.replacement.as_str(); edit.apply_to(value).is_ok() && changed }});").unwrap();
            }
            argui_schema::ValueType::String => writeln!(output, "{pad}        let value = match &event.kind {{ ::argui::ui::UiEventKind::TextChanged(value) | ::argui::ui::UiEventKind::Submitted(value) => value.clone(), _ => return }}; {property}.set(value);").unwrap(),
            argui_schema::ValueType::Float => writeln!(output, "{pad}        let value = match &event.kind {{ ::argui::ui::UiEventKind::Scrolled {{ offset, .. }} => offset.y, _ => return }}; {property}.set(value);").unwrap(),
            unsupported => {
                return Err(CompilerError::Codegen(format!(
                    "native two-way event payload `{unsupported:?}` has no AOT conversion"
                )));
            }
        }
        Ok(())
    }
}
