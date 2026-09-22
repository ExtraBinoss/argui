use std::collections::HashMap;

use argui_dsl_ir::{
    EventTargetId, IrEventBinding, IrExpressionKind, IrPropertyBinding, LocalId, PropertyTargetId,
};

use crate::{ComponentInstance, DslValue, LiveRuntime, RuntimeError};

impl LiveRuntime {
    /// Registers explicit and two-way native event routes through the host context.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn bind_native_events(
        &mut self,
        instance: &ComponentInstance,
        schema: &argui_schema::NativeSchema,
        properties: &[IrPropertyBinding],
        events: &[IrEventBinding],
        locals: &HashMap<LocalId, DslValue>,
        mut input: argui_schema::NativeElementInput,
        context: &mut Option<&mut argui_runtime::Context<Self>>,
    ) -> Result<argui_schema::NativeElementInput, RuntimeError> {
        for event_schema in &schema.events {
            let explicit = events
                .iter()
                .find(|binding| binding.target == EventTargetId::Native(event_schema.id));
            let updates = properties
                .iter()
                .filter(|binding| binding.two_way)
                .filter_map(|binding| two_way_update(schema, event_schema.id, binding))
                .collect::<Vec<_>>();
            if explicit.is_none() && updates.is_empty() {
                continue;
            }
            let Some(host) = context.as_deref_mut() else {
                continue;
            };
            let observer = host.observation_reader();
            let observed_sites = self
                .package
                .observed_sites
                .get(&instance.component)
                .cloned()
                .unwrap_or_default();
            let statements = explicit
                .map(|binding| binding.statements.clone())
                .unwrap_or_default();
            let parameter = explicit.and_then(|binding| binding.parameters.first().copied());
            let payload_type = event_schema.payload;
            let event_id = event_schema.id;
            let locals = locals.clone();
            let instance = instance.id;
            let handler = host.event_handler(move |runtime, event, host| {
                runtime.pending_focus = None;
                runtime.pending_scroll = None;
                if let Some(mounted) = runtime.instances.get_mut(&instance) {
                    for site in &observed_sites {
                        let identity = argui_ui::RetainedIdentity::new(instance.raw(), site.raw());
                        mounted.observations.insert(*site, observer.get(&identity));
                    }
                }
                match runtime.deliver_native_event(
                    instance,
                    event_id,
                    &statements,
                    locals.clone(),
                    parameter,
                    payload_type,
                    &updates,
                    event,
                ) {
                    Ok(()) => runtime.event_error = None,
                    Err(error) => runtime.event_error = Some(error),
                }
                match runtime.pending_focus.take() {
                    Some(argui_ui::FocusRequest::Next) => host.focus_next(),
                    Some(argui_ui::FocusRequest::Previous) => host.focus_previous(),
                    _ => {}
                }
                if let Some(request) = runtime.pending_scroll.take() {
                    host.scroll(request);
                }
                host.notify();
            });
            input = input.event(argui_schema::NativeEventValue::new(
                event_schema.id,
                handler,
            ));
        }
        Ok(input)
    }
}

impl argui_runtime::Render for LiveRuntime {
    /// Returns the current validated effect definitions for renderer registration.
    fn effect_definitions(&self) -> Vec<argui_render::EffectDefinition> {
        LiveRuntime::effect_definitions(self)
    }

    /// Renders the live package and preserves the last valid tree if evaluation fails.
    fn render(&mut self, context: &mut argui_runtime::Context<Self>) -> argui_ui::Element {
        #[cfg(not(target_arch = "wasm32"))]
        self.ensure_client_listener(context);
        #[cfg(target_arch = "wasm32")]
        self.ensure_web_listener(context);
        match self.render_with_context(context) {
            Ok(element) => {
                self.render_error = None;
                self.last_valid_element = Some(element.clone());
                element
            }
            Err(error) => {
                self.render_error = Some(error);
                self.last_valid_element
                    .clone()
                    .unwrap_or_else(|| argui_ui::Element::container([]))
            }
        }
    }

    /// Advances active DSL property motions and requests a rebuild only on change.
    ///
    /// * `frame` — monotonic timestamp supplied by the host scheduler.
    /// * `context` — host context receiving a notification when values changed.
    fn animation_frame(
        &mut self,
        frame: argui_animation::Frame,
        context: &mut argui_runtime::Context<Self>,
    ) {
        if self.property_motions.advance(frame.now) {
            context.notify();
        }
    }

    /// Returns whether any DSL property motion still needs a frame.
    fn wants_animation_frame(&self) -> bool {
        self.property_motions.needs_frame()
    }

    /// Captures laid-out VirtualWindow heights and rebuilds when geometry changes.
    ///
    /// `layout` contains source-identified bounds; `context` schedules a new tree
    /// when a virtual viewport must select a different mounted row window.
    fn layout_changed(
        &mut self,
        layout: &argui_runtime::LayoutSnapshot,
        context: &mut argui_runtime::Context<Self>,
    ) {
        let next = layout
            .nodes
            .iter()
            .filter_map(|node| {
                node.retained_identity
                    .as_ref()
                    .map(|identity| (identity.clone(), node.bounds.size.height))
            })
            .collect();
        if self.virtual_viewports != next {
            self.virtual_viewports = next;
            context.notify();
        }
    }

    /// Returns decoded raster assets for the current live generation.
    fn image_assets(&self) -> Vec<argui_paint::ImageAsset> {
        self.assets()
            .records()
            .filter_map(|record| match record {
                argui_assets::AssetRecord::Image { asset, .. } => Some(asset.clone()),
                _ => None,
            })
            .collect()
    }

    /// Returns decoded SVG assets for the current live generation.
    fn vector_assets(&self) -> Vec<argui_paint::VectorAsset> {
        self.assets()
            .records()
            .filter_map(|record| match record {
                argui_assets::AssetRecord::Vector { asset, .. } => Some(asset.clone()),
                _ => None,
            })
            .collect()
    }
}

/// Resolves one native two-way binding into its owning DSL property update.
fn two_way_update(
    schema: &argui_schema::NativeSchema,
    event: argui_schema::EventId,
    binding: &IrPropertyBinding,
) -> Option<(argui_dsl_ir::PropertyId, argui_schema::ValueType)> {
    let PropertyTargetId::Native(native_property) = binding.target else {
        return None;
    };
    let property_schema = schema
        .properties
        .iter()
        .find(|property| property.id == native_property)?;
    if property_schema.change_event != Some(event) {
        return None;
    }
    let IrExpressionKind::PropertyRead(property) = binding.value.kind else {
        return None;
    };
    Some((property, property_schema.value_type))
}
