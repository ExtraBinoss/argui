use std::collections::HashMap;

use argui_dsl_ir::{
    AssignmentOperator, CallbackId, IrAssignmentTarget, IrStatement, LocalId, PropertyId, TokenId,
};

use crate::{DslValue, EvaluationContext, InstanceId, LiveRuntime, RuntimeError};

impl LiveRuntime {
    /// Applies a native event payload, two-way updates, and its explicit handler block.
    pub(crate) fn deliver_native_event(
        &mut self,
        instance: InstanceId,
        statements: &[IrStatement],
        locals: HashMap<LocalId, DslValue>,
        updates: &[(PropertyId, argui_schema::ValueType)],
        event: &argui_ui::UiEvent,
    ) -> Result<(), RuntimeError> {
        for (property, value_type) in updates {
            let value = native_event_value(event, *value_type)?;
            self.set_property(instance, *property, value)?;
        }
        self.execute_statements(instance, statements, locals)?;
        Ok(())
    }

    /// Executes a restricted handler statement list on one mounted instance.
    pub(crate) fn execute_statements(
        &mut self,
        instance: InstanceId,
        statements: &[IrStatement],
        mut locals: HashMap<LocalId, DslValue>,
    ) -> Result<DslValue, RuntimeError> {
        for statement in statements {
            match statement {
                IrStatement::SetThemeMode(expression) => {
                    let value = self.evaluate_event(instance, expression.id, &locals)?;
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
                    self.evaluate_event(instance, expression.id, &locals)?;
                }
                IrStatement::Assignment {
                    target,
                    operator,
                    value,
                } => {
                    let value = self.evaluate_event(instance, value.id, &locals)?;
                    self.assign_event(instance, *target, *operator, value, &mut locals)?;
                }
                IrStatement::Return(value) => {
                    return value.as_ref().map_or(Ok(DslValue::Null), |value| {
                        self.evaluate_event(instance, value.id, &locals)
                    });
                }
            }
        }
        Ok(DslValue::Null)
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
fn native_event_value(
    event: &argui_ui::UiEvent,
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

/// Applies a restricted assignment operator without implicit type coercion.
fn assignment_value(
    operator: AssignmentOperator,
    current: DslValue,
    value: DslValue,
) -> Result<DslValue, RuntimeError> {
    use AssignmentOperator as Op;
    match (operator, current, value) {
        (Op::Set, _, value) => Ok(value),
        (Op::Add, DslValue::Int(left), DslValue::Int(right)) => Ok(DslValue::Int(left + right)),
        (Op::Subtract, DslValue::Int(left), DslValue::Int(right)) => {
            Ok(DslValue::Int(left - right))
        }
        (Op::Multiply, DslValue::Int(left), DslValue::Int(right)) => {
            Ok(DslValue::Int(left * right))
        }
        (Op::Divide, DslValue::Int(left), DslValue::Int(right)) if right != 0 => {
            Ok(DslValue::Int(left / right))
        }
        (Op::Add, DslValue::Float(left), DslValue::Float(right)) => {
            Ok(DslValue::Float(left + right))
        }
        (Op::Subtract, DslValue::Float(left), DslValue::Float(right)) => {
            Ok(DslValue::Float(left - right))
        }
        (Op::Multiply, DslValue::Float(left), DslValue::Float(right)) => {
            Ok(DslValue::Float(left * right))
        }
        (Op::Divide, DslValue::Float(left), DslValue::Float(right)) if right != 0.0 => {
            Ok(DslValue::Float(left / right))
        }
        (Op::Add, DslValue::String(left), DslValue::String(right)) => {
            Ok(DslValue::String(left + &right))
        }
        (_, left, right) => Err(RuntimeError::TypeMismatch {
            expected: "assignment-compatible operands".into(),
            actual: format!("{} and {}", left.type_name(), right.type_name()),
        }),
    }
}
