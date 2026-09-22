use std::{collections::HashMap, sync::Arc};

use argui_dsl_syntax::{FileId, Span};

use crate::{Diagnostic, Type};

/// Stable symbol identity derived from module path, kind, and public name.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SymbolId(u64);

impl SymbolId {
    /// Derives a deterministic symbol ID.
    ///
    /// * `module` — canonical module path.
    /// * `kind` — declaration-kind discriminator.
    /// * `name` — declaration name.
    #[must_use]
    pub fn derive(module: &str, kind: &str, name: &str) -> Self {
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in module
            .bytes()
            .chain([0])
            .chain(kind.bytes())
            .chain([0])
            .chain(name.bytes())
        {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        Self(hash)
    }

    /// Returns the stable numeric symbol representation.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for SymbolId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:016x}", self.0)
    }
}

/// One imported symbol and its optional local alias.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportItem {
    pub name: String,
    pub alias: String,
    pub span: Span,
}

/// Named import from a relative project module or package module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Import {
    pub source: String,
    pub items: Vec<ImportItem>,
    pub span: Span,
}

/// Struct field declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldDefinition {
    pub name: String,
    pub value_type: Type,
    pub span: Span,
}

/// User struct definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructDefinition {
    pub fields: Vec<FieldDefinition>,
}

/// User enum definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnumDefinition {
    pub variants: Vec<(String, Span)>,
}

/// Component property data-flow direction.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PropertyDirection {
    Private,
    Input,
    Output,
    InputOutput,
}

/// Typed component property declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyDefinition {
    pub name: String,
    pub value_type: Type,
    pub direction: PropertyDirection,
    pub required: bool,
    pub span: Span,
    pub dependencies: Vec<String>,
}

/// Typed component callback declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallbackDefinition {
    pub name: String,
    pub parameters: Vec<FieldDefinition>,
    pub result: Type,
    pub span: Span,
}

/// Named component content slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotDefinition {
    pub name: String,
    pub template: bool,
    pub span: Span,
}

/// Declarative component API and analyzed visual behavior summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentDefinition {
    pub properties: Vec<PropertyDefinition>,
    pub callbacks: Vec<CallbackDefinition>,
    pub slots: Vec<SlotDefinition>,
    pub visual_sites: usize,
    pub repeaters: usize,
    pub states: usize,
    pub animations: usize,
    pub asset_dependencies: Vec<String>,
}

/// Typed theme token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThemeTokenDefinition {
    pub name: String,
    pub value_type: Type,
    pub dependencies: Vec<String>,
    pub span: Span,
}

/// Theme definition and its declared runtime modes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ThemeDefinition {
    pub tokens: Vec<ThemeTokenDefinition>,
    pub modes: Vec<String>,
}

/// Named style target and property names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StyleDefinition {
    pub target: String,
    pub properties: Vec<String>,
}

/// Typed effect parameter declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectParameterDefinition {
    pub name: String,
    pub value_type: Type,
    /// Whether the declaration supplies a value when an application omits it.
    pub has_default: bool,
    pub span: Span,
}

/// External-WGSL effect definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectDefinition {
    pub shader: Option<String>,
    /// Explicit promise that every shader sample stays inside its layer.
    pub bounded_damage: bool,
    pub parameters: Vec<EffectParameterDefinition>,
}

/// Semantic definition payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DefinitionKind {
    Struct(StructDefinition),
    Enum(EnumDefinition),
    Component(ComponentDefinition),
    Theme(ThemeDefinition),
    Style(StyleDefinition),
    Effect(EffectDefinition),
}

/// Resolved module definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Definition {
    pub id: SymbolId,
    pub name: String,
    pub exported: bool,
    pub span: Span,
    pub kind: DefinitionKind,
}

/// One parsed and resolved project module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Module {
    pub file: FileId,
    pub path: String,
    pub imports: Vec<Import>,
    pub definitions: Vec<Definition>,
    pub scope: HashMap<String, SymbolId>,
    pub native_scope: HashMap<String, argui_schema::NativeTypeId>,
}

/// Complete semantic snapshot consumed by checking, IR lowering, LSP, and codegen.
#[derive(Clone, Debug)]
pub struct SemanticProject {
    pub modules: Vec<Module>,
    pub diagnostics: Vec<Diagnostic>,
    definitions: Arc<HashMap<SymbolId, Definition>>,
    syntax: Arc<HashMap<FileId, argui_dsl_syntax::GreenNode>>,
}

impl SemanticProject {
    /// Returns a definition by stable symbol ID.
    ///
    /// * `id` — symbol identity to resolve.
    #[must_use]
    pub fn definition(&self, id: SymbolId) -> Option<&Definition> {
        self.definitions.get(&id)
    }

    /// Returns whether no error-severity diagnostics were produced.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == crate::Severity::Error)
    }

    /// Returns the lossless syntax root retained for typed IR lowering and tooling.
    ///
    /// * `file` — source file identity.
    #[must_use]
    pub fn syntax(&self, file: FileId) -> Option<argui_dsl_syntax::SyntaxNode> {
        self.syntax
            .get(&file)
            .cloned()
            .map(argui_dsl_syntax::SyntaxNode::new_root)
    }

    pub(crate) fn new(
        modules: Vec<Module>,
        diagnostics: Vec<Diagnostic>,
        syntax: HashMap<FileId, argui_dsl_syntax::GreenNode>,
    ) -> Self {
        let definitions = modules
            .iter()
            .flat_map(|module| module.definitions.iter())
            .map(|definition| (definition.id, definition.clone()))
            .collect();
        Self {
            modules,
            diagnostics,
            definitions: Arc::new(definitions),
            syntax: Arc::new(syntax),
        }
    }
}
