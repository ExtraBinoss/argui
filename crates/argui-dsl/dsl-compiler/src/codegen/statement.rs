//! Typed Rust emission for handler statements.

use crate::{
    CompilerError,
    codegen::{Context, expression::Scope},
};
use argui_dsl_ir::{AssignmentOperator, IrAssignmentTarget, IrStatement, IrType};

impl Context<'_> {
    /// Emits a restricted handler statement against typed property handles.
    pub(super) fn statement(
        &self,
        statement: &IrStatement,
        scope: &mut Scope,
    ) -> Result<String, CompilerError> {
        let mut canonical = scope.clone();
        canonical.rendered_properties.clear();
        let read_scope = &canonical;
        match statement {
            IrStatement::Let { local, value } => {
                let initial = self.expression(value, read_scope)?;
                let name = format!("local_{}", local.raw());
                scope.locals.insert(*local, name.clone());
                scope.local_types.insert(*local, value.value_type.clone());
                Ok(format!("let mut {name} = {initial};"))
            }
            IrStatement::If { condition, then_body, else_body } => {
                let condition = self.expression(condition, read_scope)?;
                let mut then_scope = scope.clone();
                let then_body = then_body.iter().map(|statement| self.statement(statement, &mut then_scope)).collect::<Result<Vec<_>, _>>()?.join(" ");
                let mut else_scope = scope.clone();
                let else_body = else_body.iter().map(|statement| self.statement(statement, &mut else_scope)).collect::<Result<Vec<_>, _>>()?.join(" ");
                Ok(format!("if {condition} {{ {then_body} }} else {{ {else_body} }}"))
            }
            IrStatement::PreventDefault => Ok("host_effects.borrow_mut().push(HostEffect::PreventDefault);".into()),
            IrStatement::StopPropagation => Ok("host_effects.borrow_mut().push(HostEffect::StopPropagation);".into()),
            IrStatement::FocusNext => Ok("host_effects.borrow_mut().push(HostEffect::Focus(::argui::ui::FocusRequest::Next));".into()),
            IrStatement::FocusPrevious => Ok("host_effects.borrow_mut().push(HostEffect::Focus(::argui::ui::FocusRequest::Previous));".into()),
            IrStatement::ScrollTo { site, x, y } => Ok(format!("host_effects.borrow_mut().push(HostEffect::Scroll(::argui::ui::ScrollRequest::offset(::argui::ui::RetainedIdentity::new(owner, {}), ::argui::core::Point::new(({}) as f32, ({}) as f32))));", site.raw(), self.expression(x, read_scope)?, self.expression(y, read_scope)?)),
            IrStatement::SetThemeMode(mode) => Ok(format!(
                "set_theme_mode({});",
                self.expression(mode, read_scope)?
            )),
            IrStatement::Expression(value) => Ok(format!("{};", self.expression(value, read_scope)?)),
            IrStatement::Return(None) => Ok("return;".into()),
            IrStatement::Return(Some(value)) => {
                let result = scope.return_type.as_ref().unwrap_or(&value.value_type);
                Ok(format!("return {};", self.expression_as(value, result, read_scope)?))
            }
            IrStatement::Assignment {
                target,
                operator,
                value,
            } => {
                if let IrAssignmentTarget::Local(local) = target {
                    let name = read_scope.locals.get(local).ok_or_else(|| CompilerError::Codegen(format!("local {} is outside handler scope", local.raw())))?;
                    let expected = read_scope.local_types.get(local).ok_or_else(|| CompilerError::Codegen(format!("local {} has no type", local.raw())))?;
                    let value = self.expression_as(value, expected, read_scope)?;
                    let update = match operator {
                        AssignmentOperator::Set => format!("{name} = {value};"),
                        AssignmentOperator::Add if *expected == IrType::String => format!("{name}.push_str(&({value}));"),
                        operator if *expected == IrType::Int => {
                            let method = match operator {
                                AssignmentOperator::Add => "checked_add",
                                AssignmentOperator::Subtract => "checked_sub",
                                AssignmentOperator::Multiply => "checked_mul",
                                _ => "checked_div",
                            };
                            format!("{name} = {name}.{method}({value}).expect(\"invalid integer arithmetic\");")
                        }
                        operator => {
                            let symbol = match operator {
                                AssignmentOperator::Add => "+=", AssignmentOperator::Subtract => "-=",
                                AssignmentOperator::Multiply => "*=", AssignmentOperator::Divide => "/=",
                                AssignmentOperator::Set => unreachable!(),
                            };
                            if *operator == AssignmentOperator::Divide {
                                format!("{{ let value = {value}; assert!(value != 0.0, \"zero divisor\"); {name} {symbol} value; }}")
                            } else { format!("{name} {symbol} {value};") }
                        }
                    };
                    return Ok(update);
                }
                let IrAssignmentTarget::Property(property) = target else { unreachable!() };
                let expected = self.ir.components.iter().flat_map(|component| &component.properties)
                    .find(|candidate| candidate.id == *property)
                    .map(|property| &property.value_type)
                    .ok_or_else(|| CompilerError::Codegen("assignment property has no type".into()))?;
                let property = scope.properties.get(property).ok_or(CompilerError::Codegen(
                    "assignment target is outside component scope".into(),
                ))?;
                let value = self.expression_as(value, expected, read_scope)?;
                if *operator == AssignmentOperator::Set {
                    return Ok(format!("{property}.set({value});"));
                }
                let operation = match operator {
                    AssignmentOperator::Add if *expected == IrType::String => "current.push_str(&value)".into(),
                    operator if *expected == IrType::Int => {
                        let method = match operator {
                            AssignmentOperator::Add => "checked_add",
                            AssignmentOperator::Subtract => "checked_sub",
                            AssignmentOperator::Multiply => "checked_mul",
                            _ => "checked_div",
                        };
                        format!("*current = current.{method}(value).expect(\"invalid integer arithmetic\")")
                    }
                    AssignmentOperator::Divide => "assert!(value != 0.0, \"zero divisor\"); *current /= value".into(),
                    operator => {
                        let operator = match operator {
                            AssignmentOperator::Add => "+=",
                            AssignmentOperator::Subtract => "-=",
                            AssignmentOperator::Multiply => "*=",
                            _ => "/=",
                        };
                        format!("*current {operator} value")
                    }
                };
                Ok(format!("{{ let value = {value}; {property}.update(|current| {{ {operation}; }}); }}"))
            }
        }
    }
}
