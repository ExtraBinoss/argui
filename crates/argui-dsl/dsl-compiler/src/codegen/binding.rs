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
            "Self {{ instance: next_instance(), translator: RefCell::new(Rc::new(|_| None)),"
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
        write!(output, "pub fn render(&self) -> ::argui::ui::Element {{ let mut handlers: NativeEventRegistrar<'_> = None; let translator = self.translator.borrow().clone(); render_component_{}(self.instance, &translator", ir.id.raw()).unwrap();
        for property in &source.properties {
            write!(
                output,
                ", self.{}.clone()",
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
        writeln!(output, ", &mut handlers) }} }}").unwrap();
        if required.is_empty() {
            writeln!(
                output,
                "impl Default for {name} {{ fn default() -> Self {{ Self::new() }} }}"
            )
            .unwrap();
        }
        writeln!(output, "impl ::argui::runtime::Render for {name} {{").unwrap();
        writeln!(output, "fn render(&mut self, cx: &mut ::argui::runtime::Context<Self>) -> ::argui::ui::Element {{").unwrap();
        writeln!(output, "let mut register = |callback: NativeEventCallback| cx.event_handler(move |_, event, cx| {{ callback(event); cx.notify(); }});").unwrap();
        writeln!(
            output,
            "let mut handlers: NativeEventRegistrar<'_> = Some(&mut register);"
        )
        .unwrap();
        writeln!(output, "let translator = self.translator.borrow().clone();").unwrap();
        write!(
            output,
            "render_component_{}(self.instance, &translator",
            ir.id.raw()
        )
        .unwrap();
        for property in &source.properties {
            write!(
                output,
                ", self.{}.clone()",
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
        writeln!(output, ", &mut handlers) }} }}").unwrap();
        Ok(())
    }
}
