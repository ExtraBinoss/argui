use std::collections::HashMap;

use argui_dsl_ir::{
    CallbackId, FieldId, IrAnimation, IrExpression, IrExpressionKind, IrNode, IrState, IrType,
    IrValue, LocalId, PropertyId, PropertyTargetId, SiteId, SlotId,
};

use crate::{CompilerError, codegen::Context};

/// Resolved generated variable names available at one expression site.
#[derive(Clone, Default)]
pub(super) struct Scope {
    pub observation_owner: Option<String>,
    pub observed_identities: HashMap<SiteId, String>,
    pub return_type: Option<IrType>,
    pub properties: HashMap<PropertyId, String>,
    pub child_properties: HashMap<(SiteId, PropertyId), String>,
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

impl Scope {
    /// Returns the retained identity expression for observed `site` in this scope.
    pub(super) fn observation_identity(&self, site: SiteId) -> String {
        self.observed_identities
            .get(&site)
            .cloned()
            .unwrap_or_else(|| {
                format!(
                    "::argui::ui::RetainedIdentity::new({}, {})",
                    self.observation_owner.as_deref().unwrap_or("owner"),
                    site.raw()
                )
            })
    }

    /// Returns the generated variable a closure must capture to observe `site`.
    pub(super) fn observation_capture(&self, site: SiteId) -> String {
        self.observed_identities
            .get(&site)
            .or(self.observation_owner.as_ref())
            .cloned()
            .unwrap_or_else(|| "owner".into())
    }
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
                            .ok_or(CompilerError::Codegen(format!(
                                "property {} is outside the generated scope",
                                id.raw()
                            )))?
                    )
                }
            }
            IrExpressionKind::ChildPropertyRead { site, property } => format!(
                "{}.get()",
                scope
                    .child_properties
                    .get(&(*site, *property))
                    .ok_or(CompilerError::Codegen(format!(
                        "child property {} at site {} is outside the generated scope",
                        property.raw(),
                        site.raw()
                    )))?
            ),
            IrExpressionKind::ObservedRead {
                site, observation, ..
            } => {
                let state = format!("observer.get(&{})", scope.observation_identity(*site));
                match observation {
                    argui_dsl_ir::IrObservation::Hover => {
                        format!("{state}.states.contains(::argui::ui::VisualState::Hovered)")
                    }
                    argui_dsl_ir::IrObservation::Pressed => {
                        format!("{state}.states.contains(::argui::ui::VisualState::Pressed)")
                    }
                    argui_dsl_ir::IrObservation::Focused => {
                        format!("{state}.states.contains(::argui::ui::VisualState::Focused)")
                    }
                    argui_dsl_ir::IrObservation::FocusVisible => {
                        format!("{state}.states.contains(::argui::ui::VisualState::FocusVisible)")
                    }
                    argui_dsl_ir::IrObservation::PointerX => {
                        format!("{state}.pointer_position.map_or(0.0, |point| point.x)")
                    }
                    argui_dsl_ir::IrObservation::PointerY => {
                        format!("{state}.pointer_position.map_or(0.0, |point| point.y)")
                    }
                    argui_dsl_ir::IrObservation::PointerGlobalX => {
                        format!("{state}.pointer_global_position.map_or(0.0, |point| point.x)")
                    }
                    argui_dsl_ir::IrObservation::PointerGlobalY => {
                        format!("{state}.pointer_global_position.map_or(0.0, |point| point.y)")
                    }
                    argui_dsl_ir::IrObservation::PressedX => {
                        format!("{state}.pressed_position.map_or(0.0, |point| point.x)")
                    }
                    argui_dsl_ir::IrObservation::PressedY => {
                        format!("{state}.pressed_position.map_or(0.0, |point| point.y)")
                    }
                    argui_dsl_ir::IrObservation::ScrollX => {
                        format!("{state}.scroll.map_or(0.0, |scroll| scroll.offset.x)")
                    }
                    argui_dsl_ir::IrObservation::ScrollY => {
                        format!("{state}.scroll.map_or(0.0, |scroll| scroll.offset.y)")
                    }
                    argui_dsl_ir::IrObservation::ViewportWidth => {
                        format!("{state}.scroll.map_or(0.0, |scroll| scroll.viewport.width)")
                    }
                    argui_dsl_ir::IrObservation::ViewportHeight => {
                        format!("{state}.scroll.map_or(0.0, |scroll| scroll.viewport.height)")
                    }
                    argui_dsl_ir::IrObservation::ContentWidth => {
                        format!("{state}.scroll.map_or(0.0, |scroll| scroll.content.width)")
                    }
                    argui_dsl_ir::IrObservation::ContentHeight => {
                        format!("{state}.scroll.map_or(0.0, |scroll| scroll.content.height)")
                    }
                }
            }
            IrExpressionKind::LocalRead(id) => format!(
                "{}.clone()",
                scope.locals.get(id).ok_or(CompilerError::Codegen(format!(
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
                self.token_functions
                    .get(id)
                    .ok_or(CompilerError::Codegen(format!(
                        "theme token {} is unavailable",
                        id.raw()
                    )))?
            ),
            IrExpressionKind::Asset(id) => format!("asset_handle({})", id.raw()),
            IrExpressionKind::BuiltinCall {
                function: argui_dsl_ir::BuiltinFunction::Translate,
                arguments,
            } => {
                let key = arguments
                    .first()
                    .map(|argument| self.expression(argument, scope))
                    .transpose()?
                    .unwrap_or("String::new()".into());
                scope.translator.as_ref().map_or(key.clone(), |translator| {
                    format!("{{ let key = {key}; ({translator})(&key).unwrap_or(key) }}")
                })
            }
            IrExpressionKind::BuiltinCall {
                function: argui_dsl_ir::BuiltinFunction::Stringify,
                arguments,
            } => {
                let value = arguments
                    .first()
                    .map(|argument| self.expression(argument, scope))
                    .transpose()?
                    .unwrap_or("String::new()".into());
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
                function: argui_dsl_ir::BuiltinFunction::Lower,
                arguments,
            } => {
                let value = self.expression(&arguments[0], scope)?;
                format!("({value}).to_lowercase()")
            }
            IrExpressionKind::BuiltinCall {
                function: argui_dsl_ir::BuiltinFunction::Range,
                arguments,
            } => {
                let count = self.expression(&arguments[0], scope)?;
                format!("(0_i64..({count}).clamp(0, 100_000)).collect::<Vec<i64>>()")
            }
            IrExpressionKind::BuiltinCall {
                function: argui_dsl_ir::BuiltinFunction::Slice,
                arguments,
            } => {
                let value = self.expression(&arguments[0], scope)?;
                let start = self.expression(&arguments[1], scope)?;
                let count = self.expression(&arguments[2], scope)?;
                format!(
                    "({value}).chars().skip(({start}).max(0) as usize).take(({count}).max(0) as usize).collect::<String>()"
                )
            }
            IrExpressionKind::BuiltinCall {
                function: argui_dsl_ir::BuiltinFunction::Hsv,
                arguments,
            } => {
                let values = arguments
                    .iter()
                    .map(|argument| self.expression(argument, scope))
                    .collect::<Result<Vec<_>, _>>()?;
                format!(
                    "::argui::core::Color::hsva(({}) as f32, ({}) as f32, ({}) as f32, ({}) as f32)",
                    values[0], values[1], values[2], values[3]
                )
            }
            IrExpressionKind::BuiltinCall {
                function: argui_dsl_ir::BuiltinFunction::ColorHex,
                arguments,
            } => {
                let value = self.expression(&arguments[0], scope)?;
                format!("({value}).to_hex_rgba()")
            }
            IrExpressionKind::BuiltinCall {
                function:
                    function @ (argui_dsl_ir::BuiltinFunction::ColorRed
                    | argui_dsl_ir::BuiltinFunction::ColorGreen
                    | argui_dsl_ir::BuiltinFunction::ColorBlue),
                arguments,
            } => {
                let value = self.expression(&arguments[0], scope)?;
                let index = match function {
                    argui_dsl_ir::BuiltinFunction::ColorRed => 0,
                    argui_dsl_ir::BuiltinFunction::ColorGreen => 1,
                    _ => 2,
                };
                format!("i64::from(({value}).to_srgba8()[{index}])")
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
                let signature = self
                    .ir
                    .components
                    .iter()
                    .flat_map(|component| &component.callbacks)
                    .find(|candidate| candidate.id == *callback)
                    .ok_or_else(|| {
                        CompilerError::Codegen("callback signature is unavailable".into())
                    })?;
                let callback =
                    scope
                        .callbacks
                        .get(callback)
                        .ok_or(CompilerError::Codegen(format!(
                            "callback {} is outside the generated scope",
                            callback.raw()
                        )))?;
                let arguments = arguments
                    .iter()
                    .zip(&signature.parameters)
                    .map(|(argument, expected)| self.expression_as(argument, expected, scope))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(", ");
                format!(
                    "({callback}.borrow().as_ref().map(|handler| handler({arguments})).unwrap_or_default())"
                )
            }
            IrExpressionKind::Unary { operator, operand } => {
                if *operator == argui_dsl_ir::UnaryOperator::Negate
                    && operand.value_type == IrType::Int
                {
                    return Ok(format!(
                        "({}).checked_neg().expect(\"invalid integer arithmetic\")",
                        self.expression(operand, scope)?
                    ));
                }
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
                let mut left_code = self.expression(left, scope)?;
                let mut right_code = self.expression(right, scope)?;
                let float_like = |value: &IrType| {
                    matches!(
                        value,
                        IrType::Float | IrType::Length | IrType::Dimension | IrType::Percentage
                    )
                };
                if left.value_type == IrType::Int && float_like(&right.value_type) {
                    left_code = format!("({left_code} as f32)");
                }
                if right.value_type == IrType::Int && float_like(&left.value_type) {
                    right_code = format!("({right_code} as f32)");
                }
                if *operator == argui_dsl_ir::BinaryOperator::Add
                    && value.value_type == IrType::String
                {
                    format!(
                        "{{ let mut text = {left_code}; text.push_str(&({right_code})); text }}"
                    )
                } else if value.value_type == IrType::Int
                    && matches!(
                        operator,
                        argui_dsl_ir::BinaryOperator::Add
                            | argui_dsl_ir::BinaryOperator::Subtract
                            | argui_dsl_ir::BinaryOperator::Multiply
                            | argui_dsl_ir::BinaryOperator::Divide
                            | argui_dsl_ir::BinaryOperator::Remainder
                    )
                {
                    let method = match operator {
                        argui_dsl_ir::BinaryOperator::Add => "checked_add",
                        argui_dsl_ir::BinaryOperator::Subtract => "checked_sub",
                        argui_dsl_ir::BinaryOperator::Multiply => "checked_mul",
                        argui_dsl_ir::BinaryOperator::Divide => "checked_div",
                        _ => "checked_rem",
                    };
                    format!(
                        "(({left_code}).{method}({right_code}).expect(\"invalid integer arithmetic\"))"
                    )
                } else if matches!(
                    operator,
                    argui_dsl_ir::BinaryOperator::Divide | argui_dsl_ir::BinaryOperator::Remainder
                ) {
                    format!(
                        "({{ let left = {left_code}; let right = {right_code}; assert!(right != 0.0, \"zero divisor\"); left {} right }})",
                        binary_operator(*operator)
                    )
                } else {
                    format!("({left_code} {} {right_code})", binary_operator(*operator))
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
                    self.expression_as(then_value, &value.value_type, scope)?,
                    self.expression_as(else_value, &value.value_type, scope)?
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
                .ok_or(CompilerError::Codegen(format!(
                    "gradient argument {index} is missing"
                )))
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

    /// Resolves a stable field ID to its generated Rust name.
    fn field_name(&self, field: FieldId) -> Result<&str, CompilerError> {
        self.field_names
            .get(&field)
            .map(String::as_str)
            .ok_or(CompilerError::Codegen(format!(
                "unknown struct field {}",
                field.raw()
            )))
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
