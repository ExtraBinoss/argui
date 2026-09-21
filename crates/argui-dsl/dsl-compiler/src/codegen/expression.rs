use std::collections::HashMap;

use argui_dsl_ir::{
    AssignmentOperator, CallbackId, FieldId, IrAssignmentTarget, IrExpression, IrExpressionKind,
    IrStatement, IrType, IrValue, LocalId, PropertyId, SlotId,
};

use crate::{CompilerError, codegen::Context};

/// Resolved generated variable names available at one expression site.
#[derive(Clone, Default)]
pub(super) struct Scope {
    pub properties: HashMap<PropertyId, String>,
    pub callbacks: HashMap<CallbackId, String>,
    pub locals: HashMap<LocalId, String>,
    pub slots: HashMap<SlotId, String>,
    pub translator: Option<String>,
}

impl Context<'_> {
    /// Emits one typed IR expression without runtime name lookup.
    pub(super) fn expression(
        &self,
        value: &IrExpression,
        scope: &Scope,
    ) -> Result<String, CompilerError> {
        Ok(match &value.kind {
            IrExpressionKind::Constant(value) => constant(value),
            IrExpressionKind::PropertyRead(id) => format!(
                "{}.get()",
                scope
                    .properties
                    .get(id)
                    .ok_or_else(|| CompilerError::Codegen(format!(
                        "property {} is outside the generated scope",
                        id.raw()
                    )))?
            ),
            IrExpressionKind::LocalRead(id) => format!(
                "{}.clone()",
                scope
                    .locals
                    .get(id)
                    .ok_or_else(|| CompilerError::Codegen(format!(
                        "local {} is outside the generated scope",
                        id.raw()
                    )))?
            ),
            IrExpressionKind::FieldRead { base, field } => format!(
                "({}).{}.clone()",
                self.expression(base, scope)?,
                self.field_name(*field)?
            ),
            IrExpressionKind::TokenRead(id) => format!(
                "{}()",
                self.token_functions.get(id).ok_or_else(|| {
                    CompilerError::Codegen(format!("theme token {} is unavailable", id.raw()))
                })?
            ),
            IrExpressionKind::Asset(id) => format!("ASSET_{}", id.raw()),
            IrExpressionKind::BuiltinCall {
                function: argui_dsl_ir::BuiltinFunction::Translate,
                arguments,
            } => {
                let key = arguments.first().map_or_else(
                    || Ok("String::new()".into()),
                    |argument| self.expression(argument, scope),
                )?;
                scope.translator.as_ref().map_or(key.clone(), |translator| {
                    format!("{{ let key = {key}; ({translator})(&key).unwrap_or(key) }}")
                })
            }
            IrExpressionKind::CallbackCall {
                callback,
                arguments,
            } => {
                let callback = scope.callbacks.get(callback).ok_or_else(|| {
                    CompilerError::Codegen(format!(
                        "callback {} is outside the generated scope",
                        callback.raw()
                    ))
                })?;
                let arguments = arguments
                    .iter()
                    .map(|argument| self.expression(argument, scope))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                format!(
                    "({callback}.borrow().as_ref().map(|handler| handler({arguments})).unwrap_or_default())"
                )
            }
            IrExpressionKind::Unary { operator, operand } => {
                let operator = match operator {
                    argui_dsl_ir::UnaryOperator::Not => "!",
                    argui_dsl_ir::UnaryOperator::Negate => "-",
                    argui_dsl_ir::UnaryOperator::Positive => "",
                };
                let operand = self.expression(operand, scope)?;
                if operator.is_empty() {
                    operand
                } else {
                    format!("{operator}({operand})")
                }
            }
            IrExpressionKind::Binary {
                operator,
                left,
                right,
            } => format!(
                "({} {} {})",
                self.expression(left, scope)?,
                binary_operator(*operator),
                self.expression(right, scope)?
            ),
            IrExpressionKind::Conditional {
                condition,
                then_value,
                else_value,
            } => format!(
                "(if {} {{ {} }} else {{ {} }})",
                self.expression(condition, scope)?,
                self.expression(then_value, scope)?,
                self.expression(else_value, scope)?
            ),
            IrExpressionKind::Array(values) => format!(
                "vec![{}]",
                values
                    .iter()
                    .map(|value| self.expression(value, scope))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ")
            ),
        })
    }

    /// Wraps a generated Rust value in the canonical native-schema value enum.
    pub(super) fn schema_value(
        &self,
        value: &IrExpression,
        scope: &Scope,
    ) -> Result<String, CompilerError> {
        let expression = self.expression(value, scope)?;
        let constructor = match &value.value_type {
            IrType::Bool => "Bool",
            IrType::Int => "Int",
            IrType::Float => "Float",
            IrType::String | IrType::FontFamily | IrType::FontWeight => "String",
            IrType::Color => "Color",
            IrType::Brush => "Brush",
            IrType::Dimension => "Dimension",
            IrType::Length | IrType::FontSize | IrType::LineHeight => {
                return Ok(format!(
                    "::argui::schema::SchemaValue::Dimension(::argui::ui::length({expression}))"
                ));
            }
            IrType::Percentage => {
                return Ok(format!(
                    "::argui::schema::SchemaValue::Dimension(::argui::ui::percent({expression}))"
                ));
            }
            IrType::Insets => "Insets",
            IrType::Radii => "Radii",
            IrType::Border => "Border",
            IrType::Shadow => "Shadow",
            IrType::Transform => "Transform",
            unsupported => {
                return Err(CompilerError::Codegen(format!(
                    "`{unsupported:?}` cannot be assigned to a native property"
                )));
            }
        };
        Ok(format!(
            "::argui::schema::SchemaValue::{constructor}({expression})"
        ))
    }

    /// Emits a restricted handler statement against typed property handles.
    pub(super) fn statement(
        &self,
        statement: &IrStatement,
        scope: &Scope,
    ) -> Result<String, CompilerError> {
        match statement {
            IrStatement::Expression(value) => Ok(format!("{};", self.expression(value, scope)?)),
            IrStatement::Return(None) => Ok("return;".into()),
            IrStatement::Return(Some(value)) => {
                Ok(format!("return {};", self.expression(value, scope)?))
            }
            IrStatement::Assignment {
                target,
                operator,
                value,
            } => {
                let IrAssignmentTarget::Property(property) = target else {
                    return Err(CompilerError::Codegen(
                        "mutable event locals are not part of the restricted handler ABI".into(),
                    ));
                };
                let property = scope.properties.get(property).ok_or_else(|| {
                    CompilerError::Codegen("assignment target is outside component scope".into())
                })?;
                let value = self.expression(value, scope)?;
                Ok(match operator {
                    AssignmentOperator::Set => format!("{property}.set({value});"),
                    AssignmentOperator::Add => {
                        format!("{property}.update(|current| *current += {value});")
                    }
                    AssignmentOperator::Subtract => {
                        format!("{property}.update(|current| *current -= {value});")
                    }
                    AssignmentOperator::Multiply => {
                        format!("{property}.update(|current| *current *= {value});")
                    }
                    AssignmentOperator::Divide => {
                        format!("{property}.update(|current| *current /= {value});")
                    }
                })
            }
        }
    }

    /// Resolves a stable field ID to its generated Rust name.
    fn field_name(&self, field: FieldId) -> Result<&str, CompilerError> {
        self.field_names
            .get(&field)
            .map(String::as_str)
            .ok_or_else(|| CompilerError::Codegen(format!("unknown struct field {}", field.raw())))
    }
}

/// Emits a typed constant value.
fn constant(value: &IrValue) -> String {
    match value {
        IrValue::Null => "None".into(),
        IrValue::Bool(value) => value.to_string(),
        IrValue::Int(value) => format!("{value}_i64"),
        IrValue::Float(value) => format!("{value:?}_f32"),
        IrValue::String(value) => format!("String::from(\"{}\")", value.escape_default()),
        IrValue::Color(value) => {
            let [red, green, blue, alpha] = value.to_be_bytes();
            format!("::argui::core::Color::from_srgba8({red}, {green}, {blue}, {alpha})")
        }
    }
}

/// Returns the direct Rust operator for one IR binary operator.
fn binary_operator(operator: argui_dsl_ir::BinaryOperator) -> &'static str {
    match operator {
        argui_dsl_ir::BinaryOperator::Add => "+",
        argui_dsl_ir::BinaryOperator::Subtract => "-",
        argui_dsl_ir::BinaryOperator::Multiply => "*",
        argui_dsl_ir::BinaryOperator::Divide => "/",
        argui_dsl_ir::BinaryOperator::Remainder => "%",
        argui_dsl_ir::BinaryOperator::Equal => "==",
        argui_dsl_ir::BinaryOperator::NotEqual => "!=",
        argui_dsl_ir::BinaryOperator::Less => "<",
        argui_dsl_ir::BinaryOperator::LessEqual => "<=",
        argui_dsl_ir::BinaryOperator::Greater => ">",
        argui_dsl_ir::BinaryOperator::GreaterEqual => ">=",
        argui_dsl_ir::BinaryOperator::And => "&&",
        argui_dsl_ir::BinaryOperator::Or => "||",
    }
}
