//! Generated typed host surface for the live backend.

use std::fmt::Write;

use argui_dsl_ir::{ComponentId, IrComponent};
use argui_dsl_semantic::{ComponentDefinition, Definition, DefinitionKind, PropertyDirection};

use super::{Context, types};
use crate::CompilerError;

impl Context<'_> {
    /// Emits live codecs for reachable user structs and enums.
    ///
    /// `output` receives generated Rust. Returns a codegen error for unresolved
    /// fields or type names.
    ///
    /// # Errors
    ///
    /// Returns a code-generation error for an unresolved user type.
    pub(super) fn emit_live_types(&self, output: &mut String) -> Result<(), CompilerError> {
        let mut definitions = self.definitions.values().copied().collect::<Vec<_>>();
        definitions.sort_by_key(|definition| definition.id);
        for definition in definitions {
            match &definition.kind {
                DefinitionKind::Struct(value)
                    if self.reachable.structs.contains(&definition.id) =>
                {
                    let ir = self
                        .ir
                        .structs
                        .iter()
                        .find(|item| item.symbol == definition.id)
                        .ok_or_else(|| CompilerError::Codegen("user struct has no IR".into()))?;
                    self.emit_live_struct(output, definition, &value.fields, ir)?;
                }
                DefinitionKind::Enum(value) if self.reachable.enums.contains(&definition.id) => {
                    let ir = self
                        .ir
                        .enums
                        .iter()
                        .find(|item| item.symbol == definition.id)
                        .ok_or_else(|| CompilerError::Codegen("user enum has no IR".into()))?;
                    self.emit_live_enum(output, definition, &value.variants, ir);
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Emits one user struct conversion using stable field IDs.
    ///
    /// `output` receives Rust; `definition` names the struct; `fields` and `ir`
    /// provide its declared types and IDs. Returns unresolved type errors.
    ///
    /// # Errors
    ///
    /// Returns for a missing Rust representation of any field type.
    fn emit_live_struct(
        &self,
        output: &mut String,
        definition: &Definition,
        fields: &[argui_dsl_semantic::FieldDefinition],
        ir: &argui_dsl_ir::IrStruct,
    ) -> Result<(), CompilerError> {
        let name = types::type_name(&definition.name);
        writeln!(
            output,
            "#[cfg(feature = \"argui-live\")] impl ::argui_dsl_runtime::LiveValue for {name} {{"
        )
        .unwrap();
        writeln!(output, "fn into_dsl(self) -> ::argui_dsl_runtime::DslValue {{ let mut fields = ::std::collections::BTreeMap::new();").unwrap();
        for (field, lowered) in fields.iter().zip(&ir.fields) {
            let name = types::rust_identifier(&field.name);
            let ty = self.rust_type(&field.value_type)?;
            writeln!(output, "fields.insert(::argui_dsl_runtime::ir::FieldId::from_raw({}), <{ty} as ::argui_dsl_runtime::LiveValue>::into_dsl(self.{name}));", lowered.id.raw()).unwrap();
        }
        writeln!(output, "::argui_dsl_runtime::DslValue::Struct(fields) }}").unwrap();
        writeln!(output, "fn from_dsl(value: ::argui_dsl_runtime::DslValue) -> Result<Self, ::argui_dsl_runtime::RuntimeError> {{ let ::argui_dsl_runtime::DslValue::Struct(mut fields) = value else {{ return Err(::argui_dsl_runtime::RuntimeError::TypeMismatch {{ expected: \"struct\".into(), actual: value.type_name().into() }}); }}; Ok(Self {{").unwrap();
        for (field, lowered) in fields.iter().zip(&ir.fields) {
            let name = types::rust_identifier(&field.name);
            let ty = self.rust_type(&field.value_type)?;
            writeln!(output, "{name}: <{ty} as ::argui_dsl_runtime::LiveValue>::from_dsl(fields.remove(&::argui_dsl_runtime::ir::FieldId::from_raw({})).ok_or(::argui_dsl_runtime::RuntimeError::MissingProperty({}))?)?,", lowered.id.raw(), lowered.id.raw()).unwrap();
        }
        writeln!(output, "}}) }} }}").unwrap();
        Ok(())
    }

    /// Emits one user enum conversion using stable variant IDs.
    ///
    /// `output` receives Rust; `definition` names the enum; `variants` and `ir`
    /// provide its names and IDs. The generated conversion rejects unknown IDs.
    fn emit_live_enum(
        &self,
        output: &mut String,
        definition: &Definition,
        variants: &[(String, argui_dsl_syntax::Span)],
        ir: &argui_dsl_ir::IrEnum,
    ) {
        let name = types::type_name(&definition.name);
        let symbol = definition.id.raw();
        writeln!(
            output,
            "#[cfg(feature = \"argui-live\")] impl ::argui_dsl_runtime::LiveValue for {name} {{"
        )
        .unwrap();
        writeln!(
            output,
            "fn into_dsl(self) -> ::argui_dsl_runtime::DslValue {{ let variant = match self {{"
        )
        .unwrap();
        for ((variant, _), lowered) in variants.iter().zip(&ir.variants) {
            writeln!(
                output,
                "Self::{} => {},",
                types::type_name(variant),
                lowered.id.raw()
            )
            .unwrap();
        }
        writeln!(
            output,
            "}}; ::argui_dsl_runtime::DslValue::Enum {{ symbol: {symbol}, variant }} }}"
        )
        .unwrap();
        writeln!(output, "fn from_dsl(value: ::argui_dsl_runtime::DslValue) -> Result<Self, ::argui_dsl_runtime::RuntimeError> {{ match value {{ ::argui_dsl_runtime::DslValue::Enum {{ symbol: {symbol}, variant }} => match variant {{").unwrap();
        for ((variant, _), lowered) in variants.iter().zip(&ir.variants) {
            writeln!(
                output,
                "{} => Ok(Self::{}),",
                lowered.id.raw(),
                types::type_name(variant)
            )
            .unwrap();
        }
        writeln!(output, "_ => Err(::argui_dsl_runtime::RuntimeError::TypeMismatch {{ expected: \"enum variant\".into(), actual: format!(\"{{variant}}\") }}), }}, other => Err(::argui_dsl_runtime::RuntimeError::TypeMismatch {{ expected: \"enum\".into(), actual: other.type_name().into() }}), }} }} }}").unwrap();
    }

    /// Emits a live component facade with the same named Rust values as AOT.
    ///
    /// `output` receives Rust; `definition` and `source` contain the public API;
    /// `ir` supplies stable internal IDs. Returns unresolved type errors.
    ///
    /// # Errors
    ///
    /// Returns a code-generation error for an unsupported public type.
    pub(super) fn emit_live_component(
        &self,
        output: &mut String,
        definition: &Definition,
        source: &ComponentDefinition,
        ir: &IrComponent,
    ) -> Result<(), CompilerError> {
        let name = format!("{}Live", types::type_name(&definition.name));
        writeln!(output, "/// Typed live host handle for `{}`.\n#[cfg(feature = \"argui-live\")]\n#[derive(Clone, Copy)]\npub struct {name} {{ #[allow(dead_code)] instance: ::argui_dsl_runtime::InstanceId }}", definition.name).unwrap();
        writeln!(output, "#[cfg(feature = \"argui-live\")] impl {name} {{").unwrap();
        writeln!(output, "/// Attaches to a mounted live root after checking its component and public ABI.\n/// Returns an incompatibility error when the live package differs from this build.\npub fn attach(runtime: &::argui_dsl_runtime::LiveRuntime) -> Result<Self, ::argui_dsl_runtime::RuntimeError> {{ let instance = runtime.root().ok_or_else(|| ::argui_dsl_runtime::RuntimeError::IncompatiblePackage(\"no root is mounted\".into()))?; let mounted = runtime.instance(instance).ok_or(::argui_dsl_runtime::RuntimeError::MissingComponent(instance.raw()))?; if mounted.component.raw() != {} || runtime.public_api_hash() != PUBLIC_API_HASH {{ return Err(::argui_dsl_runtime::RuntimeError::IncompatiblePackage(\"generated facade and live root differ\".into())); }} Ok(Self {{ instance }}) }}", ComponentId::from_raw(definition.id.raw()).raw()).unwrap();
        for (property, lowered) in source.properties.iter().zip(&ir.properties) {
            self.emit_live_property(output, property, lowered.id)?;
        }
        for (callback, lowered) in source.callbacks.iter().zip(&ir.callbacks) {
            self.emit_live_callback(output, callback, lowered.id)?;
        }
        for (slot, id) in source.slots.iter().zip(&ir.slots) {
            let method = types::rust_identifier(&slot.name);
            writeln!(output, "/// Replaces the public `{}` slot in this mounted live root.\n/// Returns an error if the slot is no longer available.\npub fn set_{method}(&self, runtime: &mut ::argui_dsl_runtime::LiveRuntime, elements: impl IntoIterator<Item = ::argui::ui::Element>) -> Result<(), ::argui_dsl_runtime::RuntimeError> {{ runtime.set_root_slot(::argui_dsl_runtime::ir::SlotId::from_raw({}), elements) }}", slot.name, id.raw()).unwrap();
        }
        writeln!(output, "}}").unwrap();
        Ok(())
    }

    /// Emits typed live property access and direct model edits.
    ///
    /// `output` receives Rust, `property` declares direction and type, and `id`
    /// identifies the property. Returns type-generation errors.
    ///
    /// # Errors
    ///
    /// Returns when the public Rust property type cannot be emitted.
    fn emit_live_property(
        &self,
        output: &mut String,
        property: &argui_dsl_semantic::PropertyDefinition,
        id: argui_dsl_ir::PropertyId,
    ) -> Result<(), CompilerError> {
        let name = types::rust_identifier(&property.name);
        let ty = self.rust_type(&property.value_type)?;
        let id = id.raw();
        if property.direction != PropertyDirection::Private {
            let asset = matches!(property.value_type, argui_dsl_semantic::Type::Asset);
            let resolve = if asset {
                "let value = match value { ::argui_dsl_runtime::DslValue::Asset(id) => ::argui_dsl_runtime::DslValue::AssetHandle(runtime.asset_handle(id)?), other => other };"
            } else {
                ""
            };
            if let argui_dsl_semantic::Type::Model(item) = &property.value_type {
                let item_type = self.rust_type(item)?;
                writeln!(output, "/// Returns a typed `{name}` snapshot with stable row identities.\n/// Returns an error if the mounted model or its rows are incompatible.\npub fn {name}(&self, runtime: &::argui_dsl_runtime::LiveRuntime) -> Result<{ty}, ::argui_dsl_runtime::RuntimeError> {{ runtime.model_snapshot::<{item_type}>(self.instance, ::argui_dsl_runtime::ir::PropertyId::from_raw({id})) }}").unwrap();
            } else {
                writeln!(output, "/// Returns the current typed `{name}` value from the mounted live component.\n/// Returns an error if the instance or property is unavailable or incompatible.\npub fn {name}(&self, runtime: &::argui_dsl_runtime::LiveRuntime) -> Result<{ty}, ::argui_dsl_runtime::RuntimeError> {{ let value = runtime.instance(self.instance).ok_or(::argui_dsl_runtime::RuntimeError::MissingComponent(self.instance.raw()))?.properties.get(&::argui_dsl_runtime::ir::PropertyId::from_raw({id})).ok_or(::argui_dsl_runtime::RuntimeError::MissingProperty({id}))?.get().clone(); {resolve} <{ty} as ::argui_dsl_runtime::LiveValue>::from_dsl(value) }}").unwrap();
            }
        }
        if matches!(
            property.direction,
            PropertyDirection::Input | PropertyDirection::InputOutput
        ) {
            writeln!(output, "/// Assigns the typed `{name}` value to this live component.\n/// Returns whether it changed, or a runtime type/identity error.\npub fn set_{name}(&self, runtime: &mut ::argui_dsl_runtime::LiveRuntime, value: {ty}) -> Result<bool, ::argui_dsl_runtime::RuntimeError> {{ runtime.set_property(self.instance, ::argui_dsl_runtime::ir::PropertyId::from_raw({id}), <{ty} as ::argui_dsl_runtime::LiveValue>::into_dsl(value)) }}").unwrap();
            if let argui_dsl_semantic::Type::Model(item) = &property.value_type {
                self.emit_live_model(output, &name, id, &self.rust_type(item)?)?;
            }
        }
        Ok(())
    }

    /// Emits direct typed live model row operations.
    ///
    /// `output` receives Rust, `name` and `id` select the model property, and
    /// `item_type` names its row type. Returns a formatting error only on OOM.
    ///
    /// # Errors
    ///
    /// Returns a code-generation error if output formatting fails.
    fn emit_live_model(
        &self,
        output: &mut String,
        name: &str,
        id: u64,
        item_type: &str,
    ) -> Result<(), CompilerError> {
        let ty = item_type;
        writeln!(output, "/// Inserts a `{name}` row at `index`; returns its stable ID or `None` outside the model.\n/// Returns a runtime type or identity error.\npub fn {name}_insert(&self, runtime: &mut ::argui_dsl_runtime::LiveRuntime, index: usize, row: {ty}) -> Result<Option<u64>, ::argui_dsl_runtime::RuntimeError> {{ runtime.model_insert(self.instance, ::argui_dsl_runtime::ir::PropertyId::from_raw({id}), index, <{ty} as ::argui_dsl_runtime::LiveValue>::into_dsl(row)) }}").unwrap();
        writeln!(output, "/// Removes a `{name}` row at `index`, returning its stable ID and value.\n/// Returns a runtime type or identity error.\npub fn {name}_remove(&self, runtime: &mut ::argui_dsl_runtime::LiveRuntime, index: usize) -> Result<Option<(u64, {ty})>, ::argui_dsl_runtime::RuntimeError> {{ match runtime.model_remove(self.instance, ::argui_dsl_runtime::ir::PropertyId::from_raw({id}), index)? {{ Some((id, value)) => Ok(Some((id, <{ty} as ::argui_dsl_runtime::LiveValue>::from_dsl(value)?))), None => Ok(None) }} }}").unwrap();
        writeln!(output, "/// Moves a `{name}` row from `from` to `to`, preserving its stable ID.\n/// Returns a runtime type or identity error.\npub fn {name}_move(&self, runtime: &mut ::argui_dsl_runtime::LiveRuntime, from: usize, to: usize) -> Result<bool, ::argui_dsl_runtime::RuntimeError> {{ runtime.model_move(self.instance, ::argui_dsl_runtime::ir::PropertyId::from_raw({id}), from, to) }}").unwrap();
        writeln!(output, "/// Edits a `{name}` row at `index`; `edit` reports whether it changed.\n/// Returns a runtime type or identity error.\npub fn {name}_update(&self, runtime: &mut ::argui_dsl_runtime::LiveRuntime, index: usize, edit: impl FnOnce(&mut {ty}) -> bool) -> Result<bool, ::argui_dsl_runtime::RuntimeError> {{ let Some((_, value)) = runtime.model_row(self.instance, ::argui_dsl_runtime::ir::PropertyId::from_raw({id}), index)? else {{ return Ok(false); }}; let mut row = <{ty} as ::argui_dsl_runtime::LiveValue>::from_dsl(value)?; if !edit(&mut row) {{ return Ok(false); }} runtime.model_update(self.instance, ::argui_dsl_runtime::ir::PropertyId::from_raw({id}), index, <{ty} as ::argui_dsl_runtime::LiveValue>::into_dsl(row)) }}").unwrap();
        Ok(())
    }

    /// Emits a named, typed live callback binder with checked argument decoding.
    ///
    /// `output` receives Rust; `callback` supplies parameter/result types and
    /// `id` is its stable internal ID. Returns type-generation errors.
    ///
    /// # Errors
    ///
    /// Returns for an unresolved callback type.
    fn emit_live_callback(
        &self,
        output: &mut String,
        callback: &argui_dsl_semantic::CallbackDefinition,
        id: argui_dsl_ir::CallbackId,
    ) -> Result<(), CompilerError> {
        let name = types::rust_identifier(&callback.name);
        let types = callback
            .parameters
            .iter()
            .map(|parameter| self.rust_type(&parameter.value_type))
            .collect::<Result<Vec<_>, _>>()?;
        let result = self.rust_type(&callback.result)?;
        let argument_names = (0..types.len())
            .map(|index| format!("arg{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        writeln!(output, "/// Binds the named `{name}` callback with the same typed Rust signature as AOT.\n/// Returns an error if the mounted instance no longer exists.\n/// Panics only if an invalid live package supplies an incompatible callback value.\npub fn on_{name}(&self, runtime: &mut ::argui_dsl_runtime::LiveRuntime, handler: impl Fn({}) -> {result} + 'static) -> Result<(), ::argui_dsl_runtime::RuntimeError> {{ runtime.bind_callback(self.instance, ::argui_dsl_runtime::ir::CallbackId::from_raw({}), move |args| {{ #[allow(unused_mut, unused_variables)] let mut args = args.into_iter();", types.join(", "), id.raw()).unwrap();
        for (index, ty) in types.iter().enumerate() {
            writeln!(output, "let arg{index} = <{ty} as ::argui_dsl_runtime::LiveValue>::from_dsl(args.next().expect(\"checked callback arity\")).expect(\"checked callback type\");").unwrap();
        }
        writeln!(output, "<{result} as ::argui_dsl_runtime::LiveValue>::into_dsl(handler({argument_names})) }}) }}").unwrap();
        Ok(())
    }
}
