//! AOT emission of checked user aggregate constructors.

use argui_dsl_ir::{FieldId, IrExpression, IrType, VariantId};
use argui_dsl_semantic::{DefinitionKind, SymbolId};

use super::Scope;
use crate::{CompilerError, codegen::Context};

impl Context<'_> {
    /// Emits one user enum variant from its stable semantic and variant IDs.
    ///
    /// * `symbol` — checked enum definition identity.
    /// * `variant` — checked variant identity in that enum.
    ///
    /// Returns generated Rust, or a codegen error if the semantic/IR tables differ.
    pub(super) fn emit_enum_variant(
        &self,
        symbol: SymbolId,
        variant: VariantId,
    ) -> Result<String, CompilerError> {
        let name = self.ir_rust_type(&IrType::Enum(symbol))?;
        let declaration = self
            .definitions
            .get(&symbol)
            .ok_or(CompilerError::Codegen(format!("unknown enum {symbol}")))?;
        let DefinitionKind::Enum(definition) = &declaration.kind else {
            return Err(CompilerError::Codegen(format!("{symbol} is not an enum")));
        };
        let lowered = self
            .ir
            .enums
            .iter()
            .find(|value| value.symbol == symbol)
            .ok_or(CompilerError::Codegen(format!("unknown enum {symbol}")))?;
        let variant_name = definition
            .variants
            .iter()
            .zip(&lowered.variants)
            .find(|(_, value)| value.id == variant)
            .map(|((name, _), _)| super::super::types::type_name(name))
            .ok_or(CompilerError::Codegen(format!(
                "unknown enum variant {}",
                variant.raw()
            )))?;
        Ok(format!("{name}::{variant_name}"))
    }

    /// Emits a typed user struct constructor from stable field IDs.
    ///
    /// * `symbol` — checked struct definition identity.
    /// * `fields` — field IDs and authored values in source order.
    /// * `scope` — resolved expression bindings at the constructor site.
    ///
    /// Returns generated Rust, or a codegen error if a field is missing from IR.
    pub(super) fn emit_struct(
        &self,
        symbol: SymbolId,
        fields: &[(FieldId, IrExpression)],
        scope: &Scope,
    ) -> Result<String, CompilerError> {
        let name = self.ir_rust_type(&IrType::Struct {
            symbol,
            fields: Vec::new(),
        })?;
        let declared = self
            .ir
            .structs
            .iter()
            .find(|value| value.symbol == symbol)
            .ok_or(CompilerError::Codegen(format!("unknown struct {symbol}")))?;
        let values = fields
            .iter()
            .map(|(id, expression)| {
                let field = declared.fields.iter().find(|field| field.id == *id).ok_or(
                    CompilerError::Codegen(format!("unknown field {}", id.raw())),
                )?;
                Ok(format!(
                    "{}: {}",
                    self.field_name(*id)?,
                    self.expression_as(expression, &field.value_type, scope)?
                ))
            })
            .collect::<Result<Vec<String>, CompilerError>>()?;
        Ok(format!("{name} {{ {} }}", values.join(", ")))
    }
}
