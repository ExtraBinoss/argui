//! AOT code generation for user-component calls and presentation overlays.

use std::fmt::Write;

use argui_dsl_ir::{EventTargetId, IrNode, IrPropertyBinding, PropertyTargetId, SiteId};

use super::statement_capture_names;
use crate::{
    CompilerError,
    codegen::{
        Context,
        expression::{Scope, TemplateSlot},
    },
};

impl Context<'_> {
    /// Emits canonical child handles and separate render-only presentation inputs.
    ///
    /// * `output` — generated renderer source.
    /// * `component` — called DSL component definition.
    /// * `site` — stable call-site identity.
    /// * `bindings` — child property assignments, including two-way aliases.
    /// * `events` — callbacks connected at the call site.
    /// * `children` — supplied slot content.
    /// * `destination` — generated element-vector destination.
    /// * `depth` — indentation level.
    /// * `outer` — canonical and presented parent bindings.
    /// * `repeater_key` — optional stable keyed-instance expression.
    ///
    /// # Errors
    ///
    /// Returns when a binding, state, animation, or slot cannot be generated.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_component_call(
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
        let child_identity = format!("child_identity_{site}");
        let identity = self.identity(site, outer, repeater_key)?;
        writeln!(output, "{pad}let {child_identity} = {identity};").unwrap();
        let mut inner = Scope {
            translator: outer.translator.clone(),
            ..Scope::default()
        };
        let mut canonical_outer = outer.clone();
        canonical_outer.rendered_properties.clear();
        let mut presented_inputs = Vec::new();
        for (property, lowered) in source.properties.iter().zip(&ir.properties) {
            let variable = format!("child_p_{}_{}", site, lowered.id.raw());
            let binding = bindings
                .iter()
                .find(|binding| binding.target == PropertyTargetId::Component(lowered.id));
            let initial = if let Some(binding) = binding {
                if binding.two_way {
                    if let argui_dsl_ir::IrExpressionKind::PropertyRead(source) = binding.value.kind
                    {
                        outer
                            .properties
                            .get(&source)
                            .map(|value| format!("{value}.clone()"))
                            .ok_or(CompilerError::Codegen(
                                "two-way source is outside scope".into(),
                            ))?
                    } else {
                        return Err(CompilerError::Codegen(
                            "two-way binding did not lower to a property ID".into(),
                        ));
                    }
                } else {
                    let value = self.expression(&binding.value, &canonical_outer)?;
                    format!(
                        "child_properties.controlled({child_identity}.clone(), {}_u64, {})",
                        lowered.id.raw(),
                        value,
                    )
                }
            } else {
                let value = if let Some(default) = &lowered.default {
                    let expression = self.expression(default, &inner)?;
                    format!(
                        "{{ let owner = child_owner(&{child_identity}); let _ = owner; {expression} }}"
                    )
                } else {
                    self.default_value(&property.value_type)?
                };
                format!(
                    "child_properties.defaulted({child_identity}.clone(), {}_u64, {value})",
                    lowered.id.raw()
                )
            };
            writeln!(output, "{pad}let {variable} = {initial};").unwrap();
            let target = PropertyTargetId::Component(lowered.id);
            let animated = outer
                .animations
                .contains_key(&(SiteId::from_raw(site), target));
            let has_state = outer
                .states
                .get(&SiteId::from_raw(site))
                .is_some_and(|states| {
                    states.iter().any(|state| {
                        state
                            .assignments
                            .iter()
                            .any(|(assigned, _)| *assigned == target)
                    })
                });
            let inherits_presentation = binding.is_some() && !outer.rendered_properties.is_empty();
            let presented = if inherits_presentation || animated || has_state {
                let base = if let Some(binding) = binding {
                    self.expression(&binding.value, outer)?
                } else {
                    format!("{variable}.get()")
                };
                let selected = self.select_state_value(
                    SiteId::from_raw(site),
                    target,
                    base,
                    &lowered.value_type,
                    &lowered.value_type,
                    outer,
                )?;
                let sampled = self.animated_expression(
                    SiteId::from_raw(site),
                    target,
                    &selected,
                    outer,
                    &child_identity,
                )?;
                format!("Some({sampled})")
            } else {
                "None".into()
            };
            let presented_variable = format!("presented_child_{}_{}", site, lowered.id.raw());
            writeln!(output, "{pad}let {presented_variable} = {presented};").unwrap();
            presented_inputs.push(presented_variable);
            inner.properties.insert(lowered.id, variable);
        }
        for (callback, lowered) in source.callbacks.iter().zip(&ir.callbacks) {
            let variable = format!("child_c_{}_{}", site, lowered.id.raw());
            let event = events
                .iter()
                .find(|event| event.target == EventTargetId::Component(lowered.id));
            if let Some(event) = event {
                let (captured_names, observed_sites) =
                    statement_capture_names(outer, &event.statements, &[]);
                let watches = observed_sites
                    .into_iter()
                    .map(|site| {
                        format!(
                            "observer.watch(&::argui::ui::RetainedIdentity::new(owner, {}));",
                            site.raw()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                let captures = captured_names
                    .into_iter()
                    .map(|name| format!("let {name} = {name}.clone();"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let captures = if event
                    .statements
                    .iter()
                    .any(super::statement_has_host_effect)
                {
                    format!("let host_effects = host_effects.clone(); {captures}")
                } else {
                    captures
                };
                let parameters = callback
                    .parameters
                    .iter()
                    .enumerate()
                    .map(|(index, _)| format!("_parameter_{index}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut handler_scope = outer.clone();
                for (index, parameter) in event.parameters.iter().enumerate() {
                    handler_scope
                        .locals
                        .insert(*parameter, format!("_parameter_{index}"));
                }
                let statements = event
                    .statements
                    .iter()
                    .map(|statement| self.statement(statement, &handler_scope))
                    .collect::<Result<Vec<_>, _>>()?
                    .join(" ");
                writeln!(output, "{pad}let {variable}: {} = {{ {watches} {captures} Rc::new(RefCell::new(Some(Box::new(move |{parameters}| {{ {statements} }})))) }};", self.callback_type(callback)?).unwrap();
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
            if ir.template_slots.contains(slot) {
                let [repeater] = children else {
                    return Err(CompilerError::Codegen(
                        "template slot requires exactly one keyed repeater".into(),
                    ));
                };
                inner.template = Some(TemplateSlot {
                    slot: *slot,
                    repeater: repeater.clone(),
                    caller: Box::new(outer.clone()),
                });
                continue;
            }
            let variable = format!("child_s_{}_{}", site, slot.raw());
            let mutability = if index == 0 && !children.is_empty() {
                "mut "
            } else {
                ""
            };
            writeln!(output, "{pad}let {mutability}{variable} = Vec::new();").unwrap();
            if index == 0 {
                self.emit_nodes(output, children, &variable, depth, outer, repeater_key)?;
            }
            inner.slots.insert(*slot, variable.clone());
            slot_values.push(variable);
        }
        if !ir.template_slots.is_empty() {
            self.emit_template_component(
                output,
                ir,
                &inner,
                &presented_inputs,
                &child_identity,
                destination,
                depth,
            )?;
            writeln!(output, "{pad}}}").unwrap();
            return Ok(());
        }
        let owner = format!("child_owner(&{child_identity})");
        write!(
            output,
            "{pad}{destination}.push(render_component_{}({owner}, translator, property_motions, child_properties, virtual_viewports, reduced_motion, observer.clone(), host_effects.clone()",
            component.raw()
        )
        .unwrap();
        for (index, lowered) in ir.properties.iter().enumerate() {
            write!(
                output,
                ", {}, {}",
                inner.properties[&lowered.id], presented_inputs[index]
            )
            .unwrap();
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

    /// Expands a template component at its typed call site without eager slot rows.
    ///
    /// `output` receives generated Rust, `component` is the called component IR,
    /// `scope` contains its reactive bindings and caller row template,
    /// `presented_inputs` are optional render-only child values, `identity`
    /// separates retained children, `destination` receives the root element,
    /// and `depth` controls indentation.
    ///
    /// # Errors
    ///
    /// Returns when component states, animations, or its visual body cannot be emitted.
    #[allow(clippy::too_many_arguments)]
    fn emit_template_component(
        &self,
        output: &mut String,
        component: &argui_dsl_ir::IrComponent,
        scope: &Scope,
        presented_inputs: &[String],
        identity: &str,
        destination: &str,
        depth: usize,
    ) -> Result<(), CompilerError> {
        let pad = "    ".repeat(depth);
        writeln!(output, "{pad}let owner = child_owner(&{identity});").unwrap();
        writeln!(output, "{pad}let _ = owner;").unwrap();
        let mut scope = scope.clone();
        scope.translator = Some("translator".into());
        for (index, property) in component.properties.iter().enumerate() {
            let canonical = &scope.properties[&property.id];
            let rendered = format!("template_rendered_{}_{}", identity, property.id.raw());
            writeln!(
                output,
                "{pad}let {rendered} = {}.unwrap_or_else(|| {canonical}.get());",
                presented_inputs[index]
            )
            .unwrap();
            scope
                .rendered_properties
                .insert(property.id, format!("{rendered}.clone()"));
        }
        for animation in &component.animations {
            if let Some(site) = animation.owner {
                scope
                    .animations
                    .insert((site, animation.property), animation.clone());
            }
        }
        for state in &component.states {
            if let Some(site) = state.owner {
                scope.states.entry(site).or_default().push(state.clone());
            } else {
                scope.component_states.push(state.clone());
            }
        }
        for property in &component.properties {
            let target = PropertyTargetId::Component(property.id);
            let base = format!("{}.get()", scope.properties[&property.id]);
            let selected =
                self.select_own_state_value(target, base, &property.value_type, &scope)?;
            let animation = component
                .animations
                .iter()
                .find(|animation| animation.owner.is_none() && animation.property == target);
            if animation.is_none() && selected.active.is_none() {
                continue;
            }
            let variable = format!("template_state_{}_{}", identity, property.id.raw());
            let sampled = if let Some(animation) = animation {
                let retained = format!(
                    "::argui::ui::RetainedIdentity::new(owner, {})",
                    component.id.raw()
                );
                self.animated_decl_expression(animation, target, &selected, &scope, &retained)?
            } else {
                selected.effective
            };
            writeln!(output, "{pad}let {variable} = {sampled};").unwrap();
            scope.rendered_properties.insert(property.id, variable);
        }
        let roots = format!("template_roots_{}", identity);
        writeln!(output, "{pad}let mut {roots} = Vec::new();").unwrap();
        self.emit_nodes(output, &component.body, &roots, depth + 1, &scope, None)?;
        writeln!(output, "{pad}{destination}.push(if {roots}.len() == 1 {{ {roots}.remove(0) }} else {{ ::argui::ui::Element::container({roots}) }});").unwrap();
        Ok(())
    }
}
