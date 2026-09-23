use std::collections::HashMap;

use argui_dsl_ir::{
    AssignmentOperator, CallbackId, IrAssignmentTarget, IrStatement, LocalId, PropertyId, SiteId,
    TokenId,
};

use crate::{DslValue, EvaluationContext, InstanceId, LiveRuntime, RuntimeError};

impl LiveRuntime {
    /// Applies a native event payload, optional named argument, two-way updates, and its handler.
    ///
    /// `instance` owns `statements`; `locals` contains lexical values;
    /// `event_id` selects the schema payload conversion;
    /// `parameter` and `payload_type` describe the optional event argument;
    /// `updates` forwards edited native values; `event` supplies the payload.
    ///
    /// # Errors
    ///
    /// Returns schema, evaluation, or assignment errors from the event body.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn deliver_native_event(
        &mut self,
        instance: InstanceId,
        event_id: argui_schema::EventId,
        statements: &[IrStatement],
        mut locals: HashMap<LocalId, DslValue>,
        parameter: Option<LocalId>,
        payload_type: Option<argui_schema::ValueType>,
        updates: &[(PropertyId, argui_schema::ValueType)],
        event: &argui_ui::UiEvent,
    ) -> Result<(), RuntimeError> {
        self.pending_prevent_default = false;
        self.pending_stop_propagation = false;
        if let Some(parameter) = parameter {
            let payload_type = payload_type.ok_or_else(|| {
                RuntimeError::Schema(
                    "event handler binds a payload absent from the native schema".into(),
                )
            })?;
            locals.insert(
                parameter,
                native_event_value(event, event_id, payload_type)?,
            );
        }
        for (property, value_type) in updates {
            if event_id == argui_schema::builtin::TEXT_EDIT {
                let argui_ui::UiEventKind::TextEdited(edit) = &event.kind else {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "native text edit".into(),
                        actual: format!("{:?}", event.kind.event_type()),
                    });
                };
                self.apply_text_edit(instance, *property, edit)?;
            } else {
                let value = native_event_value(event, event_id, *value_type)?;
                self.set_property(instance, *property, value)?;
            }
        }
        self.execute_statements(instance, statements, locals)?;
        if self.pending_prevent_default {
            let _ = event.prevent_default();
        }
        if self.pending_stop_propagation {
            event.stop_propagation();
        }
        self.pending_prevent_default = false;
        self.pending_stop_propagation = false;
        Ok(())
    }

    /// Executes a restricted handler statement list on one mounted instance.
    pub(crate) fn execute_statements(
        &mut self,
        instance: InstanceId,
        statements: &[IrStatement],
        mut locals: HashMap<LocalId, DslValue>,
    ) -> Result<DslValue, RuntimeError> {
        Ok(self
            .execute_block(instance, statements, &mut locals)?
            .unwrap_or(DslValue::Null))
    }

    /// Executes one lexical handler block and propagates an explicit return.
    /// Mutations to outer locals remain visible; declarations are scoped by the caller.
    fn execute_block(
        &mut self,
        instance: InstanceId,
        statements: &[IrStatement],
        locals: &mut HashMap<LocalId, DslValue>,
    ) -> Result<Option<DslValue>, RuntimeError> {
        for statement in statements {
            match statement {
                IrStatement::Let { local, value } => {
                    let value = self.evaluate_event(instance, value.id, locals)?;
                    locals.insert(*local, value);
                }
                IrStatement::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    let condition = self.evaluate_event(instance, condition.id, locals)?;
                    let DslValue::Bool(condition) = condition else {
                        return Err(RuntimeError::InvalidBytecode(
                            "handler condition is not bool".into(),
                        ));
                    };
                    let mut branch_locals = locals.clone();
                    let branch = if condition { then_body } else { else_body };
                    if let Some(value) = self.execute_block(instance, branch, &mut branch_locals)? {
                        return Ok(Some(value));
                    }
                    for (id, value) in locals.iter_mut() {
                        if let Some(updated) = branch_locals.get(id) {
                            *value = updated.clone();
                        }
                    }
                }
                IrStatement::PreventDefault => self.pending_prevent_default = true,
                IrStatement::StopPropagation => self.pending_stop_propagation = true,
                IrStatement::FocusNext => {
                    self.pending_focus = Some(argui_ui::FocusRequest::Next);
                }
                IrStatement::FocusPrevious => {
                    self.pending_focus = Some(argui_ui::FocusRequest::Previous);
                }
                IrStatement::ScrollTo { site, x, y } => {
                    let x = self.evaluate_event(instance, x.id, locals)?;
                    let y = self.evaluate_event(instance, y.id, locals)?;
                    let (DslValue::Float(x), DslValue::Float(y)) = (x, y) else {
                        return Err(RuntimeError::TypeMismatch {
                            expected: "two length coordinates".into(),
                            actual: "non-length scroll offset".into(),
                        });
                    };
                    let identity = argui_ui::RetainedIdentity::new(instance.raw(), site.raw());
                    self.pending_scroll = Some(argui_ui::ScrollRequest::offset(
                        identity,
                        argui_core::Point::new(x as f32, y as f32),
                    ));
                }
                IrStatement::SetThemeMode(expression) => {
                    let value = self.evaluate_event(instance, expression.id, locals)?;
                    let DslValue::String(name) = value else {
                        return Err(RuntimeError::TypeMismatch {
                            expected: "string theme mode".into(),
                            actual: format!("{value:?}"),
                        });
                    };
                    self.set_theme_mode(argui_dsl_ir::ThemeModeId::named(&name))
                        .map_err(|error| match error {
                            RuntimeError::InvalidBytecode(message)
                                if message.starts_with("unknown theme mode ") =>
                            {
                                RuntimeError::InvalidBytecode(format!(
                                    "unknown theme mode `{name}`"
                                ))
                            }
                            other => other,
                        })?;
                }
                IrStatement::Expression(expression) => {
                    self.evaluate_event(instance, expression.id, locals)?;
                }
                IrStatement::Assignment {
                    target,
                    operator,
                    value,
                } => {
                    let value = self.evaluate_event(instance, value.id, locals)?;
                    self.assign_event(instance, *target, *operator, value, locals)?;
                }
                IrStatement::Return(value) => {
                    return value.as_ref().map_or(Ok(Some(DslValue::Null)), |value| {
                        self.evaluate_event(instance, value.id, locals).map(Some)
                    });
                }
            }
        }
        Ok(None)
    }

    /// Applies one event assignment to a property or mutable handler local.
    fn assign_event(
        &mut self,
        instance: InstanceId,
        target: IrAssignmentTarget,
        operator: AssignmentOperator,
        value: DslValue,
        locals: &mut HashMap<LocalId, DslValue>,
    ) -> Result<(), RuntimeError> {
        match target {
            IrAssignmentTarget::Property(property) => {
                let current = self
                    .instances
                    .get(&instance)
                    .and_then(|mounted| mounted.properties.get(&property))
                    .map(|property| property.get().clone())
                    .ok_or(RuntimeError::MissingProperty(property.raw()))?;
                let value = assignment_value(operator, current, value)?;
                self.set_property(instance, property, value)?;
            }
            IrAssignmentTarget::Local(local) => {
                let current = locals.get(&local).cloned().ok_or_else(|| {
                    RuntimeError::InvalidBytecode(format!("local {} is unavailable", local.raw()))
                })?;
                let value = assignment_value(operator, current, value)?;
                locals.insert(local, value);
            }
        }
        Ok(())
    }

    /// Evaluates one handler expression with re-entrant callback routing.
    fn evaluate_event(
        &mut self,
        instance: InstanceId,
        expression: argui_dsl_ir::ExpressionId,
        locals: &HashMap<LocalId, DslValue>,
    ) -> Result<DslValue, RuntimeError> {
        let program = self
            .package
            .program(expression)
            .cloned()
            .ok_or(RuntimeError::MissingExpression(expression.raw()))?;
        let mut context = EventValueContext {
            runtime: self,
            instance,
            locals,
        };
        program.evaluate(&mut context)
    }
}

/// Converts a schema-declared engine event payload into a live DSL value.
///
/// `event` carries the native payload, `event_id` selects its schema event, and
/// `expected` is the declared value type. Returns the string or numeric value.
///
/// # Errors
///
/// Returns a type mismatch when the event kind or payload does not match the schema.
fn native_event_value(
    event: &argui_ui::UiEvent,
    event_id: argui_schema::EventId,
    expected: argui_schema::ValueType,
) -> Result<DslValue, RuntimeError> {
    match (&event.kind, expected) {
        (argui_ui::UiEventKind::TextChanged(value), argui_schema::ValueType::String)
        | (argui_ui::UiEventKind::Submitted(value), argui_schema::ValueType::String) => {
            Ok(DslValue::String(value.clone()))
        }
        (argui_ui::UiEventKind::Scrolled { offset, .. }, argui_schema::ValueType::Float) => {
            Ok(DslValue::Float(f64::from(offset.y)))
        }
        (argui_ui::UiEventKind::Gesture(gesture), argui_schema::ValueType::Float) => {
            let argui_ui::GestureKind::Pan { total, .. } = gesture.kind else {
                return Err(RuntimeError::TypeMismatch {
                    expected: "pan displacement".into(),
                    actual: format!("{:?}", gesture.kind),
                });
            };
            let delta = if event_id == argui_schema::builtin::DRAG_X {
                total.x
            } else if event_id == argui_schema::builtin::DRAG_Y {
                total.y
            } else {
                return Err(RuntimeError::TypeMismatch {
                    expected: "native drag event".into(),
                    actual: format!("event {}", event_id.raw()),
                });
            };
            Ok(DslValue::Float(f64::from(delta)))
        }
        (kind, expected) => Err(RuntimeError::TypeMismatch {
            expected: format!("native event payload {expected:?}"),
            actual: format!("{:?}", kind.event_type()),
        }),
    }
}

/// Evaluates handler bytecode with access to routed runtime callbacks.
struct EventValueContext<'a> {
    runtime: &'a mut LiveRuntime,
    instance: InstanceId,
    locals: &'a HashMap<LocalId, DslValue>,
}

impl EvaluationContext for EventValueContext<'_> {
    fn property(&self, id: PropertyId) -> Option<DslValue> {
        self.runtime
            .instances
            .get(&self.instance)?
            .properties
            .get(&id)
            .map(|property| property.get().clone())
    }

    fn observed(
        &self,
        site: argui_dsl_ir::SiteId,
        observation: argui_dsl_ir::IrObservation,
    ) -> Option<DslValue> {
        self.runtime
            .instances
            .get(&self.instance)?
            .observations
            .get(&site)
            .map(|value| crate::observation::value(*value, observation))
    }

    fn child_property(&self, site: SiteId, property: PropertyId) -> Option<DslValue> {
        self.runtime
            .instances
            .get(&self.instance)?
            .child_outputs
            .get(&(site, property))
            .cloned()
    }

    fn local(&self, id: LocalId) -> Option<DslValue> {
        self.locals.get(&id).cloned()
    }

    fn token(&self, id: TokenId) -> Option<DslValue> {
        self.runtime.tokens.get(&id).cloned()
    }

    fn callback(&mut self, id: CallbackId, arguments: Vec<DslValue>) -> Option<DslValue> {
        self.runtime
            .invoke_callback(self.instance, id, arguments)
            .ok()
    }

    fn translate(&self, id: &str) -> Option<String> {
        self.runtime
            .translator
            .as_deref()
            .and_then(|translator| translator(id))
    }
}

/// Applies `operator` to `current` and `value`, sharing expression arithmetic.
/// Returns the assigned value or an incompatible-operand/overflow error.
fn assignment_value(
    operator: AssignmentOperator,
    current: DslValue,
    value: DslValue,
) -> Result<DslValue, RuntimeError> {
    use argui_dsl_ir::BinaryOperator;
    let operator = match operator {
        AssignmentOperator::Set => return Ok(value),
        AssignmentOperator::Add => BinaryOperator::Add,
        AssignmentOperator::Subtract => BinaryOperator::Subtract,
        AssignmentOperator::Multiply => BinaryOperator::Multiply,
        AssignmentOperator::Divide => {
            if matches!(value, DslValue::Float(number) if number == 0.0)
                || value == DslValue::Int(0)
            {
                return Err(RuntimeError::TypeMismatch {
                    expected: "assignment-compatible operands".into(),
                    actual: format!("{} and {}", current.type_name(), value.type_name()),
                });
            }
            BinaryOperator::Divide
        }
    };
    crate::bytecode::binary_value(operator, current, value)
}
