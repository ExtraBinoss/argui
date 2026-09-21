use std::fmt::Write;

use argui_dsl_ir::{
    EventTargetId, IrElementTarget, IrNode, IrPropertyBinding, IrType, PropertyTargetId,
};

use crate::{
    CompilerError,
    codegen::{Context, expression::Scope},
};

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
                ..
            } => {
                match target {
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
                        let children_name = format!("children_{}_{}", site.raw(), depth);
                        if child_slot.is_some() {
                            writeln!(output, "{pad}let mut {children_name}: Vec<::argui::ui::Element> = Vec::new();").unwrap();
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
                                !properties.iter().any(|binding| {
                                    binding.target == PropertyTargetId::Native(key.id)
                                })
                            })
                        {
                            let value = source_id.as_ref().map_or_else(
                                || format!("format!(\"dsl:{{}}:{}\", owner)", site.raw()),
                                |source_id| {
                                    format!("String::from(\"{}\")", source_id.escape_default())
                                },
                            );
                            writeln!(output, "{pad}{input} = {input}.property(::argui::schema::PropertyId::from_raw({}), ::argui::schema::SchemaValue::String({value}));", key.id.raw()).unwrap();
                        }
                        for property in properties {
                            let PropertyTargetId::Native(property_id) = property.target else {
                                return Err(CompilerError::Codegen(
                                    "component property reached a native element".into(),
                                ));
                            };
                            writeln!(output, "{pad}{input} = {input}.property(::argui::schema::PropertyId::from_raw({}), {});", property_id.raw(), self.schema_value(&property.value, scope)?).unwrap();
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
                        let identity = self.identity(site.raw(), scope, repeater_key)?;
                        writeln!(output, "{pad}{destination}.push(construct_native(::argui::schema::NativeTypeId::from_raw({}), &{input}).retained_identity({identity}));", native.raw()).unwrap();
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
                    }
                }
            }
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
                writeln!(output, "{pad}if {} {{", self.expression(condition, scope)?).unwrap();
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

    /// Emits a direct call into a generated DSL component renderer.
    #[allow(clippy::too_many_arguments)]
    fn emit_component_call(
        &self,
        output: &mut String,
        component: argui_dsl_ir::ComponentId,
        site: u64,
        bindings: &[IrPropertyBinding],
        events: &[argui_dsl_ir::IrEventBinding],
        children: &[IrNode],
        destination: &str,
        depth: usize,
        outer: &Scope,
        repeater_key: Option<&argui_dsl_ir::IrExpression>,
    ) -> Result<(), CompilerError> {
        let source = self.component_definition(component)?;
        let ir = self.components[&component];
        let pad = "    ".repeat(depth);
        writeln!(output, "{pad}{{").unwrap();
        let mut inner = Scope::default();
        for (property, lowered) in source.properties.iter().zip(&ir.properties) {
            let variable = format!("child_p_{}_{}", site, lowered.id.raw());
            let binding = bindings
                .iter()
                .find(|binding| binding.target == PropertyTargetId::Component(lowered.id));
            let initial = if let Some(binding) = binding {
                if binding.two_way {
                    if let argui_dsl_ir::IrExpressionKind::PropertyRead(source) = binding.value.kind
                    {
                        outer.properties.get(&source).cloned().ok_or_else(|| {
                            CompilerError::Codegen("two-way source is outside scope".into())
                        })?
                    } else {
                        return Err(CompilerError::Codegen(
                            "two-way binding did not lower to a property ID".into(),
                        ));
                    }
                } else {
                    format!(
                        "::argui::reactive::Property::new({})",
                        self.expression(&binding.value, outer)?
                    )
                }
            } else {
                let value = if let Some(default) = &lowered.default {
                    self.expression(default, &inner)?
                } else {
                    self.default_value(&property.value_type)?
                };
                format!("::argui::reactive::Property::new({value})")
            };
            writeln!(output, "{pad}let {variable} = {initial};").unwrap();
            inner.properties.insert(lowered.id, variable);
        }
        for (callback, lowered) in source.callbacks.iter().zip(&ir.callbacks) {
            let variable = format!("child_c_{}_{}", site, lowered.id.raw());
            let event = events
                .iter()
                .find(|event| event.target == EventTargetId::Component(lowered.id));
            if let Some(event) = event {
                let captures = statement_capture_names(outer, &event.statements, &[])
                    .into_iter()
                    .map(|name| format!("let {name} = {name}.clone();"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let parameters = callback
                    .parameters
                    .iter()
                    .enumerate()
                    .map(|(index, _)| format!("_parameter_{index}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let statements = event
                    .statements
                    .iter()
                    .map(|statement| self.statement(statement, outer))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(" ");
                writeln!(output, "{pad}let {variable}: {} = {{ {captures} Rc::new(RefCell::new(Some(Box::new(move |{parameters}| {{ {statements} }})))) }};", self.callback_type(callback)?).unwrap();
            } else {
                writeln!(
                    output,
                    "{pad}let {variable}: {} = Rc::new(RefCell::new(None));",
                    self.callback_type(callback)?
                )
                .unwrap();
            }
            inner.callbacks.insert(lowered.id, variable);
        }
        let mut slot_values = Vec::new();
        for (index, slot) in ir.slots.iter().enumerate() {
            let variable = format!("child_s_{}_{}", site, slot.raw());
            writeln!(output, "{pad}let mut {variable} = Vec::new();").unwrap();
            if index == 0 {
                self.emit_nodes(output, children, &variable, depth, outer, repeater_key)?;
            }
            inner.slots.insert(*slot, variable.clone());
            slot_values.push(variable);
        }
        let owner = format!("owner ^ {}_u64.rotate_left(17)", site);
        write!(
            output,
            "{pad}{destination}.push(render_component_{}({owner}, translator",
            component.raw()
        )
        .unwrap();
        for lowered in &ir.properties {
            write!(output, ", {}", inner.properties[&lowered.id]).unwrap();
        }
        for lowered in &ir.callbacks {
            write!(output, ", {}", inner.callbacks[&lowered.id]).unwrap();
        }
        for slot in slot_values {
            write!(output, ", {slot}").unwrap();
        }
        writeln!(output, ", handlers)); {pad}}}").unwrap();
        Ok(())
    }

    /// Emits native event registration and two-way payload propagation.
    #[allow(clippy::too_many_arguments)]
    fn emit_native_events(
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
            for name in statement_capture_names(scope, statements, &update_properties) {
                writeln!(output, "{pad}    let {name} = {name}.clone();").unwrap();
            }
            writeln!(
                output,
                "{pad}    let handler = register(Box::new(move |event| {{"
            )
            .unwrap();
            writeln!(output, "{pad}        let _ = event;").unwrap();
            for (binding, value_type) in updates {
                self.emit_two_way_update(output, binding, value_type, pad, scope)?;
            }
            if let Some(explicit) = explicit {
                for statement in &explicit.statements {
                    writeln!(output, "{pad}        {}", self.statement(statement, scope)?).unwrap();
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
            argui_schema::ValueType::String => writeln!(output, "{pad}        let value = match &event.kind {{ ::argui::ui::UiEventKind::TextChanged(value) | ::argui::ui::UiEventKind::Submitted(value) => value.clone(), _ => return }}; {property}.set(value);").unwrap(),
            unsupported => {
                return Err(CompilerError::Codegen(format!(
                    "native two-way event payload `{unsupported:?}` has no AOT conversion"
                )));
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

/// Returns only runtime values referenced by a generated event closure.
fn statement_capture_names(
    scope: &Scope,
    statements: &[argui_dsl_ir::IrStatement],
    extra_properties: &[argui_dsl_ir::PropertyId],
) -> Vec<String> {
    let mut names = std::collections::BTreeSet::new();
    for property in extra_properties {
        if let Some(name) = scope.properties.get(property) {
            names.insert(name.clone());
        }
    }
    for statement in statements {
        match statement {
            argui_dsl_ir::IrStatement::Expression(expression)
            | argui_dsl_ir::IrStatement::Return(Some(expression)) => {
                expression_capture_names(scope, expression, &mut names);
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
                expression_capture_names(scope, value, &mut names);
            }
            argui_dsl_ir::IrStatement::Return(None) => {}
        }
    }
    names.into_iter().collect()
}

/// Adds expression dependencies to the generated closure capture set.
fn expression_capture_names(
    scope: &Scope,
    expression: &argui_dsl_ir::IrExpression,
    names: &mut std::collections::BTreeSet<String>,
) {
    use argui_dsl_ir::IrExpressionKind as Kind;
    match &expression.kind {
        Kind::PropertyRead(property) => {
            names.extend(scope.properties.get(property).cloned());
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
                expression_capture_names(scope, argument, names);
            }
        }
        Kind::FieldRead { base, .. } | Kind::Unary { operand: base, .. } => {
            expression_capture_names(scope, base, names);
        }
        Kind::Binary { left, right, .. } => {
            expression_capture_names(scope, left, names);
            expression_capture_names(scope, right, names);
        }
        Kind::Conditional {
            condition,
            then_value,
            else_value,
        } => {
            expression_capture_names(scope, condition, names);
            expression_capture_names(scope, then_value, names);
            expression_capture_names(scope, else_value, names);
        }
        Kind::Array(values)
        | Kind::BuiltinCall {
            arguments: values, ..
        } => {
            for value in values {
                expression_capture_names(scope, value, names);
            }
        }
        Kind::Constant(_) | Kind::TokenRead(_) | Kind::Asset(_) => {}
    }
}
