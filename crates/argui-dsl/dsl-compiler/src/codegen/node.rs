use std::fmt::Write;

use argui_dsl_ir::{IrElementTarget, IrNode, IrType, PropertyTargetId};

use crate::{
    CompilerError,
    codegen::{Context, expression::Scope},
};

mod component;
mod effect;
mod events;
mod motion;
mod state;
mod virtual_list;

use virtual_list::VirtualListNode;

impl Context<'_> {
    /// Emits a sequence of IR nodes into a generated element vector.
    pub(super) fn emit_nodes(
        &self,
        output: &mut String,
        nodes: &[IrNode],
        destination: &str,
        depth: usize,
        scope: &Scope,
        repeater_key: Option<&argui_dsl_ir::IrExpression>,
    ) -> Result<(), CompilerError> {
        for node in nodes {
            self.emit_node(output, node, destination, depth, scope, repeater_key)?;
        }
        Ok(())
    }

    /// Emits one element, repeater, conditional, or slot reference.
    fn emit_node(
        &self,
        output: &mut String,
        node: &IrNode,
        destination: &str,
        depth: usize,
        scope: &Scope,
        repeater_key: Option<&argui_dsl_ir::IrExpression>,
    ) -> Result<(), CompilerError> {
        let pad = "    ".repeat(depth);
        match node {
            IrNode::Element {
                site,
                target,
                properties,
                events,
                children,
                source_id,
                effect,
                ..
            } => match target {
                IrElementTarget::Native(native) => {
                    let native_schema = self.schema.schema(*native).ok_or_else(|| {
                        CompilerError::Codegen(format!("native {} has no schema", native.raw()))
                    })?;
                    let child_slot = native_schema.slots.first();
                    if child_slot.is_none() && !children.is_empty() {
                        return Err(CompilerError::Codegen(format!(
                            "native `{}` does not accept visual children",
                            native_schema.name
                        )));
                    }
                    let motion_identity = format!("identity_{}_{}", site.raw(), depth);
                    let identity = self.identity(site.raw(), scope, repeater_key)?;
                    writeln!(output, "{pad}let {motion_identity} = {identity};").unwrap();
                    let children_name = format!("children_{}_{}", site.raw(), depth);
                    if native_schema.virtual_window {
                        self.emit_virtual_children(
                            output,
                            VirtualListNode {
                                children,
                                properties,
                                identity: &motion_identity,
                            },
                            &children_name,
                            depth,
                            scope,
                        )?;
                    } else if child_slot.is_some() {
                        let mutability = if children.is_empty() { "" } else { "mut " };
                        writeln!(output, "{pad}let {mutability}{children_name}: Vec<::argui::ui::Element> = Vec::new();").unwrap();
                        self.emit_nodes(
                            output,
                            children,
                            &children_name,
                            depth,
                            scope,
                            repeater_key,
                        )?;
                    }
                    let input = format!("input_{}_{}", site.raw(), depth);
                    writeln!(
                        output,
                        "{pad}let mut {input} = ::argui::schema::NativeElementInput::new();"
                    )
                    .unwrap();
                    if let Some(key) = native_schema
                        .properties
                        .iter()
                        .find(|property| property.name.as_str() == "key")
                        .filter(|key| {
                            !properties
                                .iter()
                                .any(|binding| binding.target == PropertyTargetId::Native(key.id))
                        })
                    {
                        let value = source_id.as_ref().map_or_else(
                            || format!("format!(\"dsl:{{}}:{}\", owner)", site.raw()),
                            |source_id| format!("String::from(\"{}\")", source_id.escape_default()),
                        );
                        writeln!(output, "{pad}{input} = {input}.property(::argui::schema::PropertyId::from_raw({}), ::argui::schema::SchemaValue::String({value}));", key.id.raw()).unwrap();
                    }
                    for property in properties {
                        if native_schema.virtual_window
                            && property.target
                                == PropertyTargetId::Native(argui_schema::builtin::VIEWPORT_HEIGHT)
                        {
                            continue;
                        }
                        let PropertyTargetId::Native(property_id) = property.target else {
                            return Err(CompilerError::Codegen(
                                "component property reached a native element".into(),
                            ));
                        };
                        let base = self.expression(&property.value, scope)?;
                        let expected = native_schema
                            .properties
                            .iter()
                            .find(|schema| schema.id == property_id)
                            .ok_or_else(|| {
                                CompilerError::Codegen(format!(
                                    "native property {} is not in schema",
                                    property_id.raw()
                                ))
                            })?;
                        let selected = self.select_state_value(
                            *site,
                            property.target,
                            base,
                            &property.value.value_type,
                            &IrType::from_schema(expected.value_type),
                            scope,
                        )?;
                        let sampled = self.animated_expression(
                            *site,
                            property.target,
                            &selected,
                            scope,
                            &motion_identity,
                        )?;
                        let value_type = scope
                            .animations
                            .get(&(*site, property.target))
                            .map_or(&selected.value_type, |animation| &animation.value_type);
                        let value = Self::schema_value_expression(value_type, &sampled)?;
                        writeln!(output, "{pad}{input} = {input}.property(::argui::schema::PropertyId::from_raw({}), {value});", property_id.raw()).unwrap();
                    }
                    self.emit_unbound_native_animations(
                        output,
                        native_schema,
                        *site,
                        properties,
                        &input,
                        &pad,
                        scope,
                        &motion_identity,
                    )?;
                    if native_schema.virtual_window {
                        writeln!(output, "{pad}{input} = {input}.property(::argui::schema::PropertyId::from_raw({}), ::argui::schema::SchemaValue::Float({children_name}_viewport)).property(::argui::schema::PropertyId::from_raw({}), ::argui::schema::SchemaValue::Int({children_name}_count as i64)).property(::argui::schema::PropertyId::from_raw({}), ::argui::schema::SchemaValue::Int({children_name}_start as i64));", argui_schema::builtin::VIEWPORT_HEIGHT.raw(), argui_schema::builtin::ITEM_COUNT.raw(), argui_schema::builtin::WINDOW_START.raw()).unwrap();
                    }
                    if let Some(slot) = child_slot {
                        writeln!(output, "{pad}{input} = {input}.slot(::argui::schema::NativeSlotValue::new(::argui::schema::SlotId::from_raw({}), {children_name}));", slot.id.raw()).unwrap();
                    }
                    self.emit_native_events(
                        output,
                        native_schema,
                        properties,
                        events,
                        &input,
                        &pad,
                        scope,
                    )?;
                    writeln!(output, "{pad}{destination}.push(construct_native(::argui::schema::NativeTypeId::from_raw({}), &{input}).retained_identity({motion_identity}));", native.raw()).unwrap();
                    if let Some(effect) = effect {
                        self.emit_applied_effect(output, effect, destination, depth, scope)?;
                    }
                }
                IrElementTarget::Component(component) => {
                    self.emit_component_call(
                        output,
                        *component,
                        site.raw(),
                        properties,
                        events,
                        children,
                        destination,
                        depth,
                        scope,
                        repeater_key,
                    )?;
                    if let Some(effect) = effect {
                        self.emit_applied_effect(output, effect, destination, depth, scope)?;
                    }
                }
            },
            IrNode::Repeater {
                local,
                model,
                key,
                body,
                ..
            } => {
                let local_name = format!("local_{}", local.raw());
                writeln!(
                    output,
                    "{pad}for {local_name} in {}.into_iter() {{",
                    self.expression(model, scope)?
                )
                .unwrap();
                let mut nested = scope.clone();
                nested.locals.insert(*local, local_name);
                self.emit_nodes(output, body, destination, depth + 1, &nested, Some(key))?;
                writeln!(output, "{pad}}}").unwrap();
            }
            IrNode::Conditional {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let expression = self.expression(condition, scope)?;
                let condition = if matches!(
                    condition.kind,
                    argui_dsl_ir::IrExpressionKind::Binary { .. }
                ) {
                    &expression[1..expression.len() - 1]
                } else {
                    expression.as_str()
                };
                writeln!(output, "{pad}if {condition} {{").unwrap();
                self.emit_nodes(
                    output,
                    then_body,
                    destination,
                    depth + 1,
                    scope,
                    repeater_key,
                )?;
                if else_body.is_empty() {
                    writeln!(output, "{pad}}}").unwrap();
                    return Ok(());
                }
                writeln!(output, "{pad}}} else {{").unwrap();
                self.emit_nodes(
                    output,
                    else_body,
                    destination,
                    depth + 1,
                    scope,
                    repeater_key,
                )?;
                writeln!(output, "{pad}}}").unwrap();
            }
            IrNode::Slot { slot, .. } => {
                let value = scope.slots.get(slot).ok_or_else(|| {
                    CompilerError::Codegen(format!("slot {} is unavailable", slot.raw()))
                })?;
                writeln!(output, "{pad}{destination}.extend({value}.clone());").unwrap();
            }
        }
        Ok(())
    }

    /// Emits retained identity with an optional typed repeater key.
    fn identity(
        &self,
        site: u64,
        scope: &Scope,
        key: Option<&argui_dsl_ir::IrExpression>,
    ) -> Result<String, CompilerError> {
        let base = format!("::argui::ui::RetainedIdentity::new(owner, {site})");
        let Some(key) = key else {
            return Ok(base);
        };
        let value = self.expression(key, scope)?;
        Ok(match key.value_type {
            IrType::String => format!("{base}.with_name_key({value})"),
            IrType::Int => format!("{base}.with_signed_key({value})"),
            _ => {
                return Err(CompilerError::Codegen(
                    "repeater keys must lower to string or integer values".into(),
                ));
            }
        })
    }
}

/// Finds captured runtime values and observed sites used by an event closure.
///
/// * `scope` — generated names available to the handler.
/// * `statements` — checked handler body to inspect.
/// * `extra_properties` — two-way targets also captured by the handler.
///
/// Returns sorted capture names and sites to watch before dispatch.
fn statement_capture_names(
    scope: &Scope,
    statements: &[argui_dsl_ir::IrStatement],
    extra_properties: &[argui_dsl_ir::PropertyId],
) -> (Vec<String>, Vec<argui_dsl_ir::SiteId>) {
    let mut names = std::collections::BTreeSet::new();
    let mut observed_sites = std::collections::BTreeSet::new();
    for property in extra_properties {
        if let Some(name) = scope.properties.get(property) {
            names.insert(name.clone());
        }
    }
    for statement in statements {
        match statement {
            argui_dsl_ir::IrStatement::Expression(expression)
            | argui_dsl_ir::IrStatement::Return(Some(expression))
            | argui_dsl_ir::IrStatement::SetThemeMode(expression) => {
                expression_capture_names(scope, expression, &mut names, &mut observed_sites);
            }
            argui_dsl_ir::IrStatement::ScrollTo { x, y, .. } => {
                expression_capture_names(scope, x, &mut names, &mut observed_sites);
                expression_capture_names(scope, y, &mut names, &mut observed_sites);
            }
            argui_dsl_ir::IrStatement::Assignment { target, value, .. } => {
                match target {
                    argui_dsl_ir::IrAssignmentTarget::Property(property) => {
                        if let Some(name) = scope.properties.get(property) {
                            names.insert(name.clone());
                        }
                    }
                    argui_dsl_ir::IrAssignmentTarget::Local(local) => {
                        if let Some(name) = scope.locals.get(local) {
                            names.insert(name.clone());
                        }
                    }
                }
                expression_capture_names(scope, value, &mut names, &mut observed_sites);
            }
            argui_dsl_ir::IrStatement::Return(None)
            | argui_dsl_ir::IrStatement::FocusNext
            | argui_dsl_ir::IrStatement::FocusPrevious
            | argui_dsl_ir::IrStatement::PreventDefault
            | argui_dsl_ir::IrStatement::StopPropagation => {}
        }
    }
    (
        names.into_iter().collect(),
        observed_sites.into_iter().collect(),
    )
}

/// Returns whether a handler statement needs the shared host-effect queue.
///
/// `statement` is the lowered handler operation. The result controls closure
/// capture so handlers without host effects emit no unused variables.
fn statement_has_host_effect(statement: &argui_dsl_ir::IrStatement) -> bool {
    matches!(
        statement,
        argui_dsl_ir::IrStatement::FocusNext
            | argui_dsl_ir::IrStatement::FocusPrevious
            | argui_dsl_ir::IrStatement::PreventDefault
            | argui_dsl_ir::IrStatement::StopPropagation
            | argui_dsl_ir::IrStatement::ScrollTo { .. }
    )
}

/// Adds expression dependencies to the generated closure capture and watch sets.
///
/// * `scope` — generated names available to the handler.
/// * `expression` — expression to traverse recursively.
/// * `names` — output receiving captured generated values.
/// * `observed_sites` — output receiving native sites sampled for input updates.
fn expression_capture_names(
    scope: &Scope,
    expression: &argui_dsl_ir::IrExpression,
    names: &mut std::collections::BTreeSet<String>,
    observed_sites: &mut std::collections::BTreeSet<argui_dsl_ir::SiteId>,
) {
    use argui_dsl_ir::IrExpressionKind as Kind;
    match &expression.kind {
        Kind::PropertyRead(property) => {
            names.extend(scope.properties.get(property).cloned());
        }
        Kind::ChildPropertyRead { site, property } => {
            names.extend(scope.child_properties.get(&(*site, *property)).cloned());
        }
        Kind::ObservedRead { site, .. } => {
            names.insert("observer".into());
            names.insert("owner".into());
            observed_sites.insert(*site);
        }
        Kind::LocalRead(local) => {
            names.extend(scope.locals.get(local).cloned());
        }
        Kind::CallbackCall {
            callback,
            arguments,
        } => {
            names.extend(scope.callbacks.get(callback).cloned());
            for argument in arguments {
                expression_capture_names(scope, argument, names, observed_sites);
            }
        }
        Kind::FieldRead { base, .. } | Kind::Unary { operand: base, .. } => {
            expression_capture_names(scope, base, names, observed_sites);
        }
        Kind::Binary { left, right, .. } => {
            expression_capture_names(scope, left, names, observed_sites);
            expression_capture_names(scope, right, names, observed_sites);
        }
        Kind::Conditional {
            condition,
            then_value,
            else_value,
        } => {
            expression_capture_names(scope, condition, names, observed_sites);
            expression_capture_names(scope, then_value, names, observed_sites);
            expression_capture_names(scope, else_value, names, observed_sites);
        }
        Kind::Array(values)
        | Kind::BuiltinCall {
            arguments: values, ..
        } => {
            for value in values {
                expression_capture_names(scope, value, names, observed_sites);
            }
        }
        Kind::Constant(_) | Kind::TokenRead(_) | Kind::Asset(_) => {}
    }
}
