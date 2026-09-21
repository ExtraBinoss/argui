use std::fmt::Write;

use argui_dsl_ir::IrComponent;
use argui_dsl_semantic::{ComponentDefinition, PropertyDirection};

use crate::{
    CompilerError,
    codegen::{Context, expression::Scope, types},
};

impl Context<'_> {
    /// Emits constructor, typed public ABI methods, and render entrypoint.
    pub(super) fn emit_public_impl(
        &self,
        output: &mut String,
        source: &ComponentDefinition,
        ir: &IrComponent,
        name: &str,
    ) -> Result<(), CompilerError> {
        write!(output, "impl {name} {{ pub fn new(").unwrap();
        let required = source
            .properties
            .iter()
            .filter(|property| property.required)
            .collect::<Vec<_>>();
        for (index, property) in required.iter().enumerate() {
            if index > 0 {
                write!(output, ", ").unwrap();
            }
            write!(
                output,
                "{}: {}",
                types::rust_identifier(&property.name),
                self.rust_type(&property.value_type)?
            )
            .unwrap();
        }
        writeln!(output, ") -> Self {{").unwrap();
        let mut scope = Scope::default();
        for (property, lowered) in source.properties.iter().zip(&ir.properties) {
            let field = types::rust_identifier(&property.name);
            let initial = if property.required {
                field.clone()
            } else if let Some(default) = &lowered.default {
                self.expression(default, &scope)?
            } else {
                self.default_value(&property.value_type)?
            };
            writeln!(
                output,
                "let {field} = ::argui::reactive::Property::new({initial});"
            )
            .unwrap();
            scope.properties.insert(lowered.id, field);
        }
        writeln!(
            output,
            "Self {{ instance: next_instance(), translator: RefCell::new(Rc::new(|_| None)), property_motions: RefCell::new(::argui::schema::PropertyMotionStore::new()), child_properties: RefCell::new(::argui::reactive::RetainedPropertyStore::new()), virtual_viewports: RefCell::new(::std::collections::HashMap::new()),"
        )
        .unwrap();
        for property in &source.properties {
            let field = types::rust_identifier(&property.name);
            writeln!(output, "{field},").unwrap();
        }
        for callback in &source.callbacks {
            writeln!(
                output,
                "callback_{}: Rc::new(RefCell::new(None)),",
                types::rust_identifier(&callback.name)
            )
            .unwrap();
        }
        for slot in &source.slots {
            writeln!(
                output,
                "slot_{}: RefCell::new(Vec::new()),",
                types::rust_identifier(&slot.name)
            )
            .unwrap();
        }
        writeln!(output, "}} }}").unwrap();
        writeln!(
            output,
            "/// Installs the Fluent-compatible resolver used by DSL `tr()` expressions."
        )
        .unwrap();
        writeln!(output, "pub fn set_translator(&self, translator: impl Fn(&str) -> Option<String> + 'static) {{ *self.translator.borrow_mut() = Rc::new(translator); }}").unwrap();
        writeln!(
            output,
            "/// Removes the localization resolver so `tr()` returns source message IDs."
        )
        .unwrap();
        writeln!(output, "pub fn clear_translator(&self) {{ *self.translator.borrow_mut() = Rc::new(|_| None); }}").unwrap();
        writeln!(
            output,
            "/// Returns the latest invalid dynamic animation value instead of crashing a render."
        )
        .unwrap();
        writeln!(output, "pub fn animation_error(&self) -> Option<String> {{ self.property_motions.borrow().last_error().map(str::to_owned) }}").unwrap();
        for property in &source.properties {
            let field = types::rust_identifier(&property.name);
            let value_type = self.rust_type(&property.value_type)?;
            if property.direction != PropertyDirection::Private {
                writeln!(
                    output,
                    "pub fn {field}(&self) -> {value_type} {{ self.{field}.get() }}"
                )
                .unwrap();
            }
            if matches!(
                property.direction,
                PropertyDirection::Input | PropertyDirection::InputOutput
            ) {
                writeln!(
                    output,
                    "pub fn set_{field}(&self, value: {value_type}) -> bool {{ self.{field}.set(value) }}"
                )
                .unwrap();
            }
        }
        for callback in &source.callbacks {
            let callback_name = types::rust_identifier(&callback.name);
            let parameters = callback
                .parameters
                .iter()
                .map(|parameter| self.rust_type(&parameter.value_type))
                .collect::<Result<Vec<_>, _>>()?
                .join(", ");
            let result = if callback.result == argui_dsl_semantic::Type::Void {
                String::new()
            } else {
                format!(" -> {}", self.rust_type(&callback.result)?)
            };
            writeln!(
                output,
                "pub fn on_{callback_name}(&self, handler: impl Fn({parameters}){result} + 'static) {{ *self.callback_{callback_name}.borrow_mut() = Some(Box::new(handler)); }}"
            )
            .unwrap();
        }
        for slot in &source.slots {
            let slot = types::rust_identifier(&slot.name);
            writeln!(output, "pub fn set_{slot}(&self, elements: impl IntoIterator<Item = ::argui::ui::Element>) {{ *self.slot_{slot}.borrow_mut() = elements.into_iter().collect(); }}").unwrap();
        }
        writeln!(
            output,
            "/// Builds a static snapshot without registering interactive handlers."
        )
        .unwrap();
        write!(output, "pub fn render(&self) -> ::argui::ui::Element {{ let mut handlers: NativeEventRegistrar<'_> = None; let translator = self.translator.borrow().clone(); let mut property_motions = self.property_motions.borrow_mut(); property_motions.begin_render(); let mut child_properties = self.child_properties.borrow_mut(); child_properties.begin_render(); let virtual_viewports = self.virtual_viewports.borrow(); let element = render_component_{}(self.instance, &translator, &mut property_motions, &mut child_properties, &virtual_viewports, false", ir.id.raw()).unwrap();
        for property in &source.properties {
            write!(
                output,
                ", self.{}.clone(), None",
                types::rust_identifier(&property.name)
            )
            .unwrap();
        }
        for callback in &source.callbacks {
            write!(
                output,
                ", self.callback_{}.clone()",
                types::rust_identifier(&callback.name)
            )
            .unwrap();
        }
        for slot in &source.slots {
            write!(
                output,
                ", self.slot_{}.borrow().clone()",
                types::rust_identifier(&slot.name)
            )
            .unwrap();
        }
        writeln!(output, ", &mut handlers); property_motions.end_render(); child_properties.end_render(); element }} }}").unwrap();
        if required.is_empty() {
            writeln!(
                output,
                "impl Default for {name} {{ fn default() -> Self {{ Self::new() }} }}"
            )
            .unwrap();
        }
        writeln!(output, "impl ::argui::runtime::Render for {name} {{").unwrap();
        writeln!(output, "fn render(&mut self, cx: &mut ::argui::runtime::Context<Self>) -> ::argui::ui::Element {{").unwrap();
        writeln!(
            output,
            "let reduced_motion = cx.environment().reduced_motion;"
        )
        .unwrap();
        writeln!(output, "let mut register = |callback: NativeEventCallback| cx.event_handler(move |_, event, cx| {{ callback(event); cx.notify(); }});").unwrap();
        writeln!(
            output,
            "let mut handlers: NativeEventRegistrar<'_> = Some(&mut register);"
        )
        .unwrap();
        writeln!(output, "let translator = self.translator.borrow().clone();").unwrap();
        writeln!(output, "let mut property_motions = self.property_motions.borrow_mut(); property_motions.begin_render();").unwrap();
        writeln!(output, "let mut child_properties = self.child_properties.borrow_mut(); child_properties.begin_render();").unwrap();
        writeln!(
            output,
            "let virtual_viewports = self.virtual_viewports.borrow();"
        )
        .unwrap();
        write!(
            output,
            "let element = render_component_{}(self.instance, &translator, &mut property_motions, &mut child_properties, &virtual_viewports, reduced_motion",
            ir.id.raw()
        )
        .unwrap();
        for property in &source.properties {
            write!(
                output,
                ", self.{}.clone(), None",
                types::rust_identifier(&property.name)
            )
            .unwrap();
        }
        for callback in &source.callbacks {
            write!(
                output,
                ", self.callback_{}.clone()",
                types::rust_identifier(&callback.name)
            )
            .unwrap();
        }
        for slot in &source.slots {
            write!(
                output,
                ", self.slot_{}.borrow().clone()",
                types::rust_identifier(&slot.name)
            )
            .unwrap();
        }
        writeln!(output, ", &mut handlers); property_motions.end_render(); child_properties.end_render(); element }}").unwrap();
        writeln!(output, "fn animation_frame(&mut self, frame: ::argui::animation::Frame, cx: &mut ::argui::runtime::Context<Self>) {{ if self.property_motions.borrow().advance(frame.now) {{ cx.notify(); }} }}").unwrap();
        writeln!(output, "fn wants_animation_frame(&self) -> bool {{ self.property_motions.borrow().needs_frame() }}").unwrap();
        writeln!(output, "/// Refreshes measured viewport heights and rebuilds virtual rows only when geometry changes.").unwrap();
        writeln!(output, "fn layout_changed(&mut self, layout: &::argui::runtime::LayoutSnapshot, cx: &mut ::argui::runtime::Context<Self>) {{ let next = layout.nodes.iter().filter_map(|node| node.retained_identity.as_ref().map(|identity| (identity.clone(), node.bounds.size.height))).collect::<::std::collections::HashMap<_, _>>(); if *self.virtual_viewports.borrow() != next {{ *self.virtual_viewports.borrow_mut() = next; cx.notify(); }} }}").unwrap();
        writeln!(
            output,
            "fn image_assets(&self) -> Vec<::argui::paint::ImageAsset> {{ image_assets() }}"
        )
        .unwrap();
        writeln!(
            output,
            "fn vector_assets(&self) -> Vec<::argui::paint::VectorAsset> {{ vector_assets() }}"
        )
        .unwrap();
        writeln!(output, "}}").unwrap();
        Ok(())
    }
}
