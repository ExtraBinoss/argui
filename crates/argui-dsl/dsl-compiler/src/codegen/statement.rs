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
        scope: &Scope,
    ) -> Result<String, CompilerError> {
        let mut canonical = scope.clone();
        canonical.rendered_properties.clear();
        let scope = &canonical;
        match statement {
            IrStatement::PreventDefault => Ok("host_effects.borrow_mut().push(HostEffect::PreventDefault);".into()),
            IrStatement::StopPropagation => Ok("host_effects.borrow_mut().push(HostEffect::StopPropagation);".into()),
            IrStatement::FocusNext => Ok("host_effects.borrow_mut().push(HostEffect::Focus(::argui::ui::FocusRequest::Next));".into()),
            IrStatement::FocusPrevious => Ok("host_effects.borrow_mut().push(HostEffect::Focus(::argui::ui::FocusRequest::Previous));".into()),
            IrStatement::ScrollTo { site, x, y } => Ok(format!("host_effects.borrow_mut().push(HostEffect::Scroll(::argui::ui::ScrollRequest::offset(::argui::ui::RetainedIdentity::new(owner, {}), ::argui::core::Point::new(({}) as f32, ({}) as f32))));", site.raw(), self.expression(x, scope)?, self.expression(y, scope)?)),
            IrStatement::SetThemeMode(mode) => Ok(format!(
                "set_theme_mode({});",
                self.expression(mode, scope)?
            )),
            IrStatement::Expression(value) => Ok(format!("{};", self.expression(value, scope)?)),
            IrStatement::Return(None) => Ok("return;".into()),
            IrStatement::Return(Some(value)) => {
                let result = scope.return_type.as_ref().unwrap_or(&value.value_type);
                Ok(format!("return {};", self.expression_as(value, result, scope)?))
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
                let expected = self.ir.components.iter().flat_map(|component| &component.properties)
                    .find(|candidate| candidate.id == *property)
                    .map(|property| &property.value_type)
                    .ok_or_else(|| CompilerError::Codegen("assignment property has no type".into()))?;
                let property = scope.properties.get(property).ok_or(CompilerError::Codegen(
                    "assignment target is outside component scope".into(),
                ))?;
                let value = self.expression_as(value, expected, scope)?;
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
