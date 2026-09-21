use std::collections::HashMap;

use argui_dsl_ir::{
    AssignmentOperator, CallbackId, FieldId, IrAnimation, IrAssignmentTarget, IrExpression,
    IrExpressionKind, IrNode, IrState, IrStatement, IrType, IrValue, LocalId, PropertyId,
    PropertyTargetId, SiteId, SlotId,
};

use crate::{CompilerError, codegen::Context};

/// Resolved generated variable names available at one expression site.
#[derive(Clone, Default)]
pub(super) struct Scope {
    pub properties: HashMap<PropertyId, String>,
    pub rendered_properties: HashMap<PropertyId, String>,
    pub callbacks: HashMap<CallbackId, String>,
    pub locals: HashMap<LocalId, String>,
    pub slots: HashMap<SlotId, String>,
    pub translator: Option<String>,
    pub animations: HashMap<(SiteId, PropertyTargetId), IrAnimation>,
    pub states: HashMap<SiteId, Vec<IrState>>,
    pub component_states: Vec<IrState>,
    pub template: Option<TemplateSlot>,
}

/// Lazy caller-authored row recipe substituted into a template component.
#[derive(Clone)]
pub(super) struct TemplateSlot {
    pub slot: SlotId,
    pub repeater: IrNode,
    pub caller: Box<Scope>,
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
            IrExpressionKind::PropertyRead(id) => {
                if let Some(value) = scope.rendered_properties.get(id) {
                    value.clone()
                } else {
                    format!(
                        "{}.get()",
                        scope
                            .properties
                            .get(id)
                            .ok_or_else(|| CompilerError::Codegen(format!(
                                "property {} is outside the generated scope",
                                id.raw()
                            )))?
                    )
                }
            }
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
            IrExpressionKind::Asset(id) => format!("asset_handle({})", id.raw()),
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
            IrExpressionKind::BuiltinCall {
                function: argui_dsl_ir::BuiltinFunction::Stringify,
                arguments,
            } => {
                let value = arguments.first().map_or_else(
                    || Ok("String::new()".into()),
                    |argument| self.expression(argument, scope),
                )?;
                format!("({value}).to_string()")
            }
            IrExpressionKind::BuiltinCall {
                function: argui_dsl_ir::BuiltinFunction::Solid,
                arguments,
            } => {
                let color = self.expression(&arguments[0], scope)?;
                format!("::argui::paint::Fill::Solid({color})")
            }
            IrExpressionKind::BuiltinCall {
                function: argui_dsl_ir::BuiltinFunction::Contains,
                arguments,
            } => {
                let text = self.expression(&arguments[0], scope)?;
                let fragment = self.expression(&arguments[1], scope)?;
                format!("({text}).contains(&({fragment}))")
            }
            IrExpressionKind::BuiltinCall {
                function:
                    function @ (argui_dsl_ir::BuiltinFunction::LinearGradient
                    | argui_dsl_ir::BuiltinFunction::RadialGradient
                    | argui_dsl_ir::BuiltinFunction::ConicGradient),
                arguments,
            } => self.gradient_expression(*function, arguments, scope)?,
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
            } => {
                let left = self.expression(left, scope)?;
                let right = self.expression(right, scope)?;
                if *operator == argui_dsl_ir::BinaryOperator::Add
                    && value.value_type == IrType::String
                {
                    format!("{{ let mut text = {left}; text.push_str(&({right})); text }}")
                } else {
                    format!("({left} {} {right})", binary_operator(*operator))
                }
            }
            IrExpressionKind::Conditional {
                condition,
                then_value,
                else_value,
            } => {
                let condition = self.expression(condition, scope)?;
                let condition = condition
                    .strip_prefix('(')
                    .and_then(|inner| inner.strip_suffix(')'))
                    .unwrap_or(&condition);
                format!(
                    "if {condition} {{ {} }} else {{ {} }}",
                    self.expression(then_value, scope)?,
                    self.expression(else_value, scope)?
                )
            }
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

    /// Emits one validated arbitrary-stop gradient constructor for AOT code.
    ///
    /// * `function` — linear, radial, or conic gradient kind.
    /// * `arguments` — typed color, offset, geometry, and color-space expressions.
    /// * `scope` — resolved local and property bindings.
    ///
    /// # Errors
    ///
    /// Returns a code-generation error if the lowered call has missing arguments.
    fn gradient_expression(
        &self,
        function: argui_dsl_ir::BuiltinFunction,
        arguments: &[IrExpression],
        scope: &Scope,
    ) -> Result<String, CompilerError> {
        let argument = |index: usize| {
            arguments
                .get(index)
                .ok_or_else(|| {
                    CompilerError::Codegen(format!("gradient argument {index} is missing"))
                })
                .and_then(|expression| self.expression(expression, scope))
        };
        let colors = argument(0)?;
        let offsets = argument(1)?;
        let call = match function {
            argui_dsl_ir::BuiltinFunction::LinearGradient => {
                let angle = argument(2)?;
                let space = argument(3)?;
                format!(
                    "::argui::paint::Fill::linear_gradient(&({colors}), &({offsets}), ({angle}) as f32, &({space}))"
                )
            }
            argui_dsl_ir::BuiltinFunction::RadialGradient => {
                let x = argument(2)?;
                let y = argument(3)?;
                let radius_x = argument(4)?;
                let radius_y = argument(5)?;
                let space = argument(6)?;
                format!(
                    "::argui::paint::Fill::radial_gradient(&({colors}), &({offsets}), ::argui::core::Point::new(({x}) as f32, ({y}) as f32), ::argui::core::Point::new(({radius_x}) as f32, ({radius_y}) as f32), &({space}))"
                )
            }
            argui_dsl_ir::BuiltinFunction::ConicGradient => {
                let x = argument(2)?;
                let y = argument(3)?;
                let angle = argument(4)?;
                let space = argument(5)?;
                format!(
                    "::argui::paint::Fill::conic_gradient(&({colors}), &({offsets}), ::argui::core::Point::new(({x}) as f32, ({y}) as f32), ({angle}) as f32, &({space}))"
                )
            }
            _ => return Err(CompilerError::Codegen("not a gradient function".into())),
        };
        Ok(format!(
            "{call}.expect(\"invalid authored gradient stops or color space\")"
        ))
    }

    /// Wraps a generated expression using its statically resolved DSL type.
    ///
    /// * `value_type` — type of the expression after animation sampling.
    /// * `expression` — Rust expression producing the typed value.
    ///
    /// # Errors
    ///
    /// Returns when the type cannot cross the native schema boundary.
    pub(super) fn schema_value_expression(
        value_type: &IrType,
        expression: &str,
    ) -> Result<String, CompilerError> {
        let constructor = match value_type {
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
                    "::argui::schema::SchemaValue::Dimension(::argui::ui::percent(({expression}) / 100.0_f32))"
                ));
            }
            IrType::Insets => "Insets",
            IrType::Radii => "Radii",
            IrType::Border => "Border",
            IrType::Shadow => "Shadow",
            IrType::Transform => "Transform",
            IrType::Asset => "Asset",
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
        let mut canonical = scope.clone();
        canonical.rendered_properties.clear();
        let scope = &canonical;
        match statement {
            IrStatement::SetThemeMode(mode) => Ok(format!(
                "set_theme_mode({});",
                self.expression(mode, scope)?
            )),
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
