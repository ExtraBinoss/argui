//! References to retained child component outputs within one component.

use std::collections::HashSet;

use crate::{IrComponent, IrExpression, IrExpressionKind, IrNode, IrStatement, SiteId};

impl IrComponent {
    /// Returns child sites whose public properties are read by this component.
    ///
    /// The result includes reads in property defaults, visual bindings, handlers,
    /// states, and animations. A backend can prepare only these retained children
    /// before evaluating expressions that may read their outputs.
    #[must_use]
    pub fn referenced_child_sites(&self) -> HashSet<SiteId> {
        let mut sites = HashSet::new();
        for property in &self.properties {
            if let Some(default) = &property.default {
                collect_expression(default, &mut sites);
            }
        }
        for node in &self.body {
            collect_node(node, &mut sites);
        }
        for state in &self.states {
            collect_expression(&state.condition, &mut sites);
            for (_, value) in &state.assignments {
                collect_expression(value, &mut sites);
            }
        }
        for animation in &self.animations {
            for parameter in &animation.parameters {
                collect_expression(&parameter.value, &mut sites);
            }
            for keyframe in &animation.keyframes {
                collect_expression(&keyframe.value, &mut sites);
            }
        }
        sites
    }
}

/// Collects child reads in one visual node and its descendants.
fn collect_node(node: &IrNode, sites: &mut HashSet<SiteId>) {
    match node {
        IrNode::Element {
            properties,
            effect,
            events,
            children,
            ..
        } => {
            for property in properties {
                collect_expression(&property.value, sites);
            }
            if let Some(effect) = effect {
                for parameter in &effect.parameters {
                    collect_expression(&parameter.value, sites);
                }
            }
            for event in events {
                for statement in &event.statements {
                    collect_statement(statement, sites);
                }
            }
            for child in children {
                collect_node(child, sites);
            }
        }
        IrNode::Repeater {
            model, key, body, ..
        } => {
            collect_expression(model, sites);
            collect_expression(key, sites);
            for child in body {
                collect_node(child, sites);
            }
        }
        IrNode::Conditional {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_expression(condition, sites);
            for child in then_body.iter().chain(else_body) {
                collect_node(child, sites);
            }
        }
        IrNode::Slot { fallback: body, .. } | IrNode::SlotContent { body, .. } => {
            for child in body {
                collect_node(child, sites);
            }
        }
    }
}

/// Collects child reads in a restricted event handler statement.
fn collect_statement(statement: &IrStatement, sites: &mut HashSet<SiteId>) {
    match statement {
        IrStatement::Expression(value)
        | IrStatement::SetThemeMode(value)
        | IrStatement::Assignment { value, .. } => collect_expression(value, sites),
        IrStatement::ScrollTo { x, y, .. } => {
            collect_expression(x, sites);
            collect_expression(y, sites);
        }
        IrStatement::Return(Some(value)) => collect_expression(value, sites),
        IrStatement::Return(None)
        | IrStatement::FocusNext
        | IrStatement::FocusPrevious
        | IrStatement::PreventDefault
        | IrStatement::StopPropagation => {}
    }
}

/// Collects child reads within one expression tree.
fn collect_expression(expression: &IrExpression, sites: &mut HashSet<SiteId>) {
    match &expression.kind {
        IrExpressionKind::ChildPropertyRead { site, .. } => {
            sites.insert(*site);
        }
        IrExpressionKind::FieldRead { base, .. }
        | IrExpressionKind::Unary { operand: base, .. } => collect_expression(base, sites),
        IrExpressionKind::BuiltinCall { arguments, .. }
        | IrExpressionKind::CallbackCall { arguments, .. }
        | IrExpressionKind::Array(arguments) => {
            for argument in arguments {
                collect_expression(argument, sites);
            }
        }
        IrExpressionKind::Binary { left, right, .. } => {
            collect_expression(left, sites);
            collect_expression(right, sites);
        }
        IrExpressionKind::Conditional {
            condition,
            then_value,
            else_value,
        } => {
            collect_expression(condition, sites);
            collect_expression(then_value, sites);
            collect_expression(else_value, sites);
        }
        _ => {}
    }
}
