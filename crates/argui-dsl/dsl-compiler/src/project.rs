use std::{path::PathBuf, sync::Arc};

use argui_dsl_semantic::{
    CompilerDatabase, DefinitionKind, PropertyDirection, QueryStats, SemanticProject,
};
use argui_shader::{ShaderParameterMetadata, validate_effect_source};

use crate::{CompilerError, Reachability, codegen};

mod asset_source;

/// One canonical project module supplied to the compiler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceModule {
    pub path: String,
    pub source: Arc<str>,
}

impl SourceModule {
    /// Creates an in-memory source module.
    ///
    /// * `path` — project-relative canonical module path.
    /// * `source` — lossless UTF-8 `.argui` source.
    #[must_use]
    pub fn new(path: impl Into<String>, source: impl Into<Arc<str>>) -> Self {
        Self {
            path: path.into(),
            source: source.into(),
        }
    }
}

/// Complete accepted AOT compilation artifact.
pub struct CompiledProject {
    pub semantic: Arc<SemanticProject>,
    pub ir: argui_dsl_ir::IrProject,
    pub reachability: Reachability,
    pub roots: Vec<argui_dsl_ir::ComponentId>,
    pub rust: String,
    pub public_api_hash: u64,
    pub dependencies: Vec<String>,
}

/// Version-matched Argui DSL compiler frontend and AOT backend.
pub struct Compiler;

/// Long-lived incremental compiler database used by the external dev service.
pub struct CompilerSession {
    database: CompilerDatabase,
    entry_module: String,
}

impl CompilerSession {
    /// Creates an empty incremental project for one canonical entry module.
    ///
    /// # Errors
    ///
    /// Returns if the built-in native schema cannot be constructed.
    pub fn new(entry_module: impl Into<String>) -> Result<Self, CompilerError> {
        Ok(Self {
            database: CompilerDatabase::with_builtins()?,
            entry_module: entry_module.into(),
        })
    }

    /// Creates an incremental session with the same application native registry as runtime.
    ///
    /// * `entry_module` — canonical root module path.
    /// * `registry` — validated built-ins plus application native extensions.
    ///
    /// Returns an empty source session with standard DSL modules loaded.
    #[must_use]
    pub fn with_registry(
        entry_module: impl Into<String>,
        registry: argui_schema::SchemaRegistry,
    ) -> Self {
        Self {
            database: CompilerDatabase::with_registry(registry),
            entry_module: entry_module.into(),
        }
    }

    /// Adds or updates one module while retaining every unaffected parse cache.
    pub fn update_module(&mut self, module: SourceModule) {
        self.database.set_file(&module.path, module.source);
    }

    /// Removes one module and invalidates graph-level semantic queries.
    pub fn remove_module(&mut self, path: &str) -> bool {
        self.database.remove_path(path)
    }

    /// Returns cumulative parse and semantic query execution counts.
    #[must_use]
    pub const fn stats(&self) -> QueryStats {
        self.database.stats()
    }

    /// Returns the canonical source path associated with `file`, if it is loaded.
    #[must_use]
    pub fn file_path(&self, file: argui_dsl_syntax::FileId) -> Option<&str> {
        self.database.file_path(file)
    }

    /// Compiles the current snapshot through the shared IR and AOT backend.
    ///
    /// # Errors
    ///
    /// Returns diagnostics, lowering, asset, shader, or code-generation errors.
    pub fn compile(
        &mut self,
        mut load_asset: impl FnMut(&str) -> Result<Vec<u8>, String>,
    ) -> Result<CompiledProject, CompilerError> {
        let semantic = self.database.check();
        compile_semantic(
            semantic,
            &self.entry_module,
            &mut load_asset,
            self.database.schema(),
        )
    }
}

impl Compiler {
    /// Compiles a complete module snapshot and validates every reachable asset.
    ///
    /// * `modules` — complete source graph snapshot.
    /// * `entry_module` — module whose exported declarations form release roots.
    /// * `load_asset` — callback reading canonical project-relative asset bytes.
    ///
    /// # Errors
    ///
    /// Returns diagnostics, lowering failures, shader/asset failures, or codegen errors.
    pub fn compile(
        modules: impl IntoIterator<Item = SourceModule>,
        entry_module: &str,
        load_asset: impl FnMut(&str) -> Result<Vec<u8>, String>,
    ) -> Result<CompiledProject, CompilerError> {
        Self::compile_with_registry(
            modules,
            entry_module,
            argui_schema::builtin::registry()?,
            load_asset,
        )
    }

    /// Compiles against one application-owned native registry used by every backend.
    ///
    /// * `modules` — complete source graph snapshot.
    /// * `entry_module` — module exporting release roots.
    /// * `registry` — built-ins and application extensions with stable ABI IDs.
    /// * `load_asset` — loader for reachable project assets.
    ///
    /// # Errors
    ///
    /// Returns semantic, lowering, asset, ABI, or code-generation errors.
    pub fn compile_with_registry(
        modules: impl IntoIterator<Item = SourceModule>,
        entry_module: &str,
        registry: argui_schema::SchemaRegistry,
        mut load_asset: impl FnMut(&str) -> Result<Vec<u8>, String>,
    ) -> Result<CompiledProject, CompilerError> {
        let mut database = CompilerDatabase::with_registry(registry);
        for module in modules {
            database.set_file(&module.path, module.source);
        }
        let semantic = database.check();
        compile_semantic(semantic, entry_module, &mut load_asset, database.schema())
    }
}

/// Completes both backends from one checked semantic snapshot.
fn compile_semantic(
    semantic: Arc<SemanticProject>,
    entry_module: &str,
    load_asset: &mut impl FnMut(&str) -> Result<Vec<u8>, String>,
    schema: &argui_schema::SchemaRegistry,
) -> Result<CompiledProject, CompilerError> {
    if !semantic.is_valid() {
        return Err(CompilerError::Semantic(semantic.diagnostics.clone()));
    }
    if !semantic
        .modules
        .iter()
        .any(|module| module.path == entry_module)
    {
        return Err(CompilerError::MissingEntry(entry_module.into()));
    }
    let ir = argui_dsl_ir::lower(&semantic, schema).map_err(CompilerError::Lower)?;
    let reachability = Reachability::analyze(&semantic, &ir, entry_module);
    let roots = entry_roots(&semantic, entry_module);
    validate_shaders(&semantic, &ir, &reachability, load_asset)?;
    validate_media_bindings(&ir, &reachability)?;
    validate_asset_sources(&ir, &reachability, load_asset)?;
    let public_api_hash = public_api_hash(&semantic, entry_module);
    let rust = codegen::emit(
        &semantic,
        &ir,
        &reachability,
        entry_module,
        public_api_hash,
        schema,
    )?;
    let mut dependencies = ir
        .assets
        .iter()
        .filter(|asset| reachability.assets.contains(&asset.id) && asset.inline_bytes.is_none())
        .map(|asset| asset.path.clone())
        .collect::<Vec<_>>();
    dependencies.sort();
    dependencies.dedup();
    let mut ir = ir;
    ir.assets
        .retain(|asset| reachability.assets.contains(&asset.id));
    ir.effects
        .retain(|effect| reachability.effects.contains(&effect.id));
    Ok(CompiledProject {
        semantic,
        ir,
        reachability,
        roots,
        rust,
        public_api_hash,
        dependencies,
    })
}

/// Rejects statically known media kind mismatches at their native element sites.
///
/// * `ir` — lowered project containing source asset declarations.
/// * `reachable` — selected release component graph.
///
/// # Errors
///
/// Returns an asset error if an Image references SVG or a Svg references raster data.
fn validate_media_bindings(
    ir: &argui_dsl_ir::IrProject,
    reachable: &Reachability,
) -> Result<(), CompilerError> {
    for component in &ir.components {
        if reachable.components.contains(&component.id) {
            for node in &component.body {
                validate_media_node(node, ir)?;
            }
        }
    }
    Ok(())
}

/// Validates direct `asset(...)` bindings in a nested visual node.
///
/// * `node` — visual node under inspection.
/// * `ir` — project asset table.
///
/// # Errors
///
/// Returns an asset error for a source kind incompatible with Image, Svg, or Path.
fn validate_media_node(
    node: &argui_dsl_ir::IrNode,
    ir: &argui_dsl_ir::IrProject,
) -> Result<(), CompilerError> {
    use argui_dsl_ir::{IrElementTarget, IrExpressionKind, IrNode, PropertyTargetId};
    match node {
        IrNode::Element {
            target,
            properties,
            children,
            ..
        } => {
            let expected = match target {
                IrElementTarget::Native(id) if *id == argui_schema::builtin::IMAGE => {
                    Some(argui_dsl_ir::AssetKind::Image)
                }
                IrElementTarget::Native(id)
                    if *id == argui_schema::builtin::SVG || *id == argui_schema::builtin::PATH =>
                {
                    Some(argui_dsl_ir::AssetKind::Vector)
                }
                _ => None,
            };
            if let Some(expected) = expected {
                for binding in properties {
                    if binding.target != PropertyTargetId::Native(argui_schema::builtin::SOURCE) {
                        continue;
                    }
                    let IrExpressionKind::Asset(id) = &binding.value.kind else {
                        continue;
                    };
                    let Some(asset) = ir.assets.iter().find(|asset| asset.id == *id) else {
                        continue;
                    };
                    if asset.kind != expected {
                        return Err(CompilerError::Asset {
                            path: PathBuf::from(&asset.path),
                            message: format!(
                                "expected {expected:?} media for this native element, found {:?}",
                                asset.kind
                            ),
                            source_span: binding.value.source.span,
                        });
                    }
                }
            }
            for child in children {
                validate_media_node(child, ir)?;
            }
        }
        IrNode::Repeater { body, .. } => {
            for child in body {
                validate_media_node(child, ir)?;
            }
        }
        IrNode::Conditional {
            then_body,
            else_body,
            ..
        } => {
            for child in then_body.iter().chain(else_body) {
                validate_media_node(child, ir)?;
            }
        }
        IrNode::Slot { fallback: body, .. } | IrNode::SlotContent { body, .. } => {
            for child in body {
                validate_media_node(child, ir)?;
            }
        }
    }
    Ok(())
}

/// Resolves embedded standard-library icons without a project filesystem path.
///
/// * `path` — canonical compiler asset path.
///
/// Returns icon bytes when the virtual asset exists; otherwise `None`.
pub(crate) fn builtin_asset_bytes(path: &str) -> Option<Vec<u8>> {
    let name = path.strip_prefix("@argui/icons/")?.strip_suffix(".svg")?;
    argui_dsl_stdlib::icon_svg(name).map(String::into_bytes)
}

/// Validates each reachable non-shader asset before either backend consumes it.
///
/// * `ir` — lowered project with asset kinds.
/// * `reachable` — assets retained by release roots.
/// * `load` — project-local byte loader for non-builtin assets.
///
/// # Errors
///
/// Returns an asset error for missing bytes or malformed raster/vector media.
fn validate_asset_sources(
    ir: &argui_dsl_ir::IrProject,
    reachable: &Reachability,
    load: &mut impl FnMut(&str) -> Result<Vec<u8>, String>,
) -> Result<(), CompilerError> {
    let mut registry = argui_assets::AssetRegistry::new();
    for asset in &ir.assets {
        if !reachable.assets.contains(&asset.id) {
            continue;
        }
        let bytes = if let Some(bytes) = &asset.inline_bytes {
            Ok(bytes.clone())
        } else {
            match asset.kind {
                argui_dsl_ir::AssetKind::Image | argui_dsl_ir::AssetKind::Vector => {
                    builtin_asset_bytes(&asset.path).map_or_else(|| load(&asset.path), Ok)
                }
                argui_dsl_ir::AssetKind::Other => load(&asset.path),
                argui_dsl_ir::AssetKind::Shader => continue,
            }
        }
        .map_err(|message| CompilerError::Asset {
            path: PathBuf::from(&asset.path),
            message,
            source_span: asset_source::find(ir, asset.id),
        })?;
        if asset.kind == argui_dsl_ir::AssetKind::Other {
            continue;
        }
        let key =
            argui_assets::AssetKey::new(&asset.path).map_err(|error| CompilerError::Asset {
                path: PathBuf::from(&asset.path),
                message: error.to_string(),
                source_span: asset_source::find(ir, asset.id),
            })?;
        let result = match asset.kind {
            argui_dsl_ir::AssetKind::Image => registry.upsert_image(key, &bytes),
            argui_dsl_ir::AssetKind::Vector => registry.upsert_vector(key, &bytes),
            _ => unreachable!("only image and vector assets are decoded"),
        };
        result.map_err(|error| CompilerError::Asset {
            path: PathBuf::from(&asset.path),
            message: error.to_string(),
            source_span: asset_source::find(ir, asset.id),
        })?;
    }
    Ok(())
}

/// Returns exported component roots from the selected entry module in source order.
fn entry_roots(project: &SemanticProject, entry: &str) -> Vec<argui_dsl_ir::ComponentId> {
    project
        .modules
        .iter()
        .find(|module| module.path == entry)
        .into_iter()
        .flat_map(|module| &module.definitions)
        .filter(|definition| {
            definition.exported
                && matches!(
                    definition.kind,
                    argui_dsl_semantic::DefinitionKind::Component(_)
                )
        })
        .map(|definition| argui_dsl_ir::ComponentId::from_raw(definition.id.raw()))
        .collect()
}

/// Validates every reachable external WGSL source against the Argui ABI.
fn validate_shaders(
    semantic: &SemanticProject,
    ir: &argui_dsl_ir::IrProject,
    reachable: &Reachability,
    load: &mut impl FnMut(&str) -> Result<Vec<u8>, String>,
) -> Result<(), CompilerError> {
    for effect in &ir.effects {
        if !reachable.effects.contains(&effect.id) {
            continue;
        }
        let Some(asset) = ir.assets.iter().find(|asset| asset.id == effect.shader) else {
            continue;
        };
        let bytes = load(&asset.path).map_err(|message| CompilerError::Asset {
            path: PathBuf::from(&asset.path),
            message,
            source_span: effect.source.span,
        })?;
        let source = std::str::from_utf8(&bytes).map_err(|error| CompilerError::Asset {
            path: PathBuf::from(&asset.path),
            message: error.to_string(),
            source_span: effect.source.span,
        })?;
        let definition = semantic
            .modules
            .iter()
            .flat_map(|module| &module.definitions)
            .find(|definition| definition.id.raw() == effect.id.raw());
        let parameters = definition
            .and_then(|definition| match &definition.kind {
                DefinitionKind::Effect(effect) => Some(effect),
                _ => None,
            })
            .map(|effect| {
                effect
                    .parameters
                    .iter()
                    .map(|parameter| {
                        ShaderParameterMetadata::new(
                            parameter.name.clone(),
                            shader_words(&parameter.value_type),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        validate_effect_source(&asset.path, source, &parameters).map_err(|error| {
            CompilerError::Asset {
                path: PathBuf::from(&asset.path),
                message: error.to_string(),
                source_span: effect.source.span,
            }
        })?;
    }
    Ok(())
}

/// Returns the effect ABI word width of one semantic parameter type.
fn shader_words(value: &argui_dsl_semantic::Type) -> usize {
    match value {
        argui_dsl_semantic::Type::Color => 4,
        argui_dsl_semantic::Type::Transform => 9,
        _ => 1,
    }
}

/// Hashes the public Rust-facing component ABI deterministically.
fn public_api_hash(project: &SemanticProject, entry: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    if let Some(module) = project.modules.iter().find(|module| module.path == entry) {
        for definition in module
            .definitions
            .iter()
            .filter(|definition| definition.exported)
        {
            hash_bytes(&mut hash, &definition.id.raw().to_le_bytes());
            match &definition.kind {
                DefinitionKind::Component(component) => {
                    for property in component
                        .properties
                        .iter()
                        .filter(|property| property.direction != PropertyDirection::Private)
                    {
                        hash_text(
                            &mut hash,
                            &format!(
                                "property:{}:{:?}:{:?}:{}",
                                property.name,
                                property.value_type,
                                property.direction,
                                property.required
                            ),
                        );
                    }
                    for callback in &component.callbacks {
                        hash_text(&mut hash, &format!("callback:{}", callback.name));
                        for parameter in &callback.parameters {
                            hash_text(
                                &mut hash,
                                &format!("parameter:{}:{:?}", parameter.name, parameter.value_type),
                            );
                        }
                        hash_text(&mut hash, &format!("result:{:?}", callback.result));
                    }
                    for slot in &component.slots {
                        hash_text(&mut hash, &format!("slot:{}", slot.name));
                    }
                }
                DefinitionKind::Struct(structure) => {
                    for field in &structure.fields {
                        hash_text(
                            &mut hash,
                            &format!("field:{}:{:?}", field.name, field.value_type),
                        );
                    }
                }
                DefinitionKind::Enum(enumeration) => {
                    for (variant, _) in &enumeration.variants {
                        hash_text(&mut hash, &format!("variant:{variant}"));
                    }
                }
                DefinitionKind::Function(function) => {
                    for parameter in &function.parameters {
                        hash_text(
                            &mut hash,
                            &format!("parameter:{}:{:?}", parameter.name, parameter.value_type),
                        );
                    }
                    hash_text(&mut hash, &format!("result:{:?}", function.result));
                }
                _ => {}
            }
        }
    }
    hash
}

/// Extends an FNV-1a ABI hash with raw bytes.
fn hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash = (*hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
    }
}

/// Extends an ABI hash with one unambiguous length-prefixed text field.
fn hash_text(hash: &mut u64, value: &str) {
    hash_bytes(hash, &(value.len() as u64).to_le_bytes());
    hash_bytes(hash, value.as_bytes());
}
