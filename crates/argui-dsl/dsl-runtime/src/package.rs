use std::{collections::HashMap, sync::Arc};

use argui_dsl_ir::{AssetId, ExpressionId, IrExpression, IrNode, IrProject};

use crate::{Program, RuntimeError};

/// Complete bytes and monotonic revision for one live source asset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssetPayload {
    pub revision: u64,
    pub bytes: Arc<[u8]>,
}

impl AssetPayload {
    /// Creates a non-zero-revision asset payload.
    #[must_use]
    pub fn new(revision: u64, bytes: impl Into<Arc<[u8]>>) -> Self {
        Self {
            revision: revision.max(1),
            bytes: bytes.into(),
        }
    }
}

/// Fully prepared, immutable live package ready for transactional migration.
pub struct LivePackage {
    pub generation: u64,
    pub public_api_hash: u64,
    pub roots: Vec<argui_dsl_ir::ComponentId>,
    pub ir: Arc<IrProject>,
    pub programs: HashMap<ExpressionId, Program>,
    pub assets: HashMap<AssetId, AssetPayload>,
    pub shader_hashes: HashMap<argui_dsl_ir::EffectId, u64>,
}

impl LivePackage {
    /// Converts a compatible wire envelope into a fully validated live package.
    ///
    /// # Errors
    ///
    /// Rejects incompatible protocol/IR/engine versions, duplicate assets, and
    /// every ordinary package preparation failure.
    pub fn from_envelope(
        envelope: argui_dsl_protocol::LivePackageEnvelope,
    ) -> Result<Self, RuntimeError> {
        argui_dsl_protocol::RuntimeVersions::current()
            .check(&envelope.header)
            .map_err(|error| RuntimeError::IncompatiblePackage(error.to_string()))?;
        let mut assets = HashMap::new();
        for asset in envelope.assets {
            if assets
                .insert(asset.id, AssetPayload::new(asset.revision, asset.bytes))
                .is_some()
            {
                return Err(RuntimeError::IncompatiblePackage(format!(
                    "asset {} is present more than once",
                    asset.id.raw()
                )));
            }
        }
        let mut package = Self::prepare(
            envelope.header.generation,
            envelope.header.public_api_hash,
            envelope.ir,
            assets,
        )?;
        package.roots = envelope.roots;
        Ok(package)
    }

    /// Validates assets/WGSL and compiles every expression before commit.
    ///
    /// * `generation` — monotonically increasing compiler generation.
    /// * `public_api_hash` — Rust-facing root ABI compatibility hash.
    /// * `ir` — complete normalized project.
    /// * `assets` — complete asset payloads referenced by the IR.
    ///
    /// # Errors
    ///
    /// Returns without a package when required bytes or shader validation fail.
    pub fn prepare(
        generation: u64,
        public_api_hash: u64,
        ir: IrProject,
        assets: HashMap<AssetId, AssetPayload>,
    ) -> Result<Self, RuntimeError> {
        for asset in &ir.assets {
            if !assets.contains_key(&asset.id) {
                return Err(RuntimeError::MissingAsset(asset.id.raw()));
            }
        }
        let mut shader_hashes = HashMap::new();
        for effect in &ir.effects {
            let payload = assets
                .get(&effect.shader)
                .ok_or(RuntimeError::MissingAsset(effect.shader.raw()))?;
            let source = std::str::from_utf8(&payload.bytes)
                .map_err(|error| RuntimeError::InvalidShader(error.to_string()))?;
            let parameters = effect
                .parameters
                .iter()
                .map(|parameter| {
                    argui_shader::ShaderParameterMetadata::new(
                        format!("p_{}", parameter.id.raw()),
                        shader_words(&parameter.value_type),
                    )
                })
                .collect::<Vec<_>>();
            let validated = argui_shader::validate_effect_source(
                format!("asset_{}", effect.shader.raw()),
                source,
                &parameters,
            )
            .map_err(|error| RuntimeError::InvalidShader(error.to_string()))?;
            shader_hashes.insert(effect.id, validated.hash);
        }
        let mut programs = HashMap::new();
        collect_project_expressions(&ir, &mut programs);
        Ok(Self {
            generation: generation.max(1),
            public_api_hash,
            roots: Vec::new(),
            ir: Arc::new(ir),
            programs,
            assets,
            shader_hashes,
        })
    }

    /// Returns a precompiled expression program by stable ID.
    #[must_use]
    pub fn program(&self, id: ExpressionId) -> Option<&Program> {
        self.programs.get(&id)
    }
}

/// Returns packed shader words for the supported effect parameter types.
fn shader_words(value: &argui_dsl_ir::IrType) -> usize {
    match value {
        argui_dsl_ir::IrType::Color => 4,
        argui_dsl_ir::IrType::Transform => 9,
        _ => 1,
    }
}

/// Compiles all project expressions exactly once.
fn collect_project_expressions(project: &IrProject, programs: &mut HashMap<ExpressionId, Program>) {
    for component in &project.components {
        for property in &component.properties {
            if let Some(value) = &property.default {
                collect_expression(value, programs);
            }
        }
        for node in &component.body {
            collect_node(node, programs);
        }
        for state in &component.states {
            collect_expression(&state.condition, programs);
            for (_, value) in &state.assignments {
                collect_expression(value, programs);
            }
        }
        for animation in &component.animations {
            for parameter in &animation.parameters {
                collect_expression(&parameter.value, programs);
            }
            for keyframe in &animation.keyframes {
                collect_expression(&keyframe.value, programs);
            }
        }
    }
    for theme in &project.themes {
        for token in &theme.tokens {
            collect_expression(&token.default, programs);
        }
        for mode in &theme.modes {
            for (_, value) in &mode.overrides {
                collect_expression(value, programs);
            }
        }
    }
    for style in &project.styles {
        for property in &style.properties {
            collect_expression(&property.value, programs);
        }
        for state in &style.states {
            for property in &state.properties {
                collect_expression(&property.value, programs);
            }
        }
    }
    for effect in &project.effects {
        for parameter in &effect.parameters {
            if let Some(value) = &parameter.default {
                collect_expression(value, programs);
            }
        }
    }
}

/// Compiles expressions recursively nested in a visual node.
fn collect_node(node: &IrNode, programs: &mut HashMap<ExpressionId, Program>) {
    match node {
        IrNode::Element {
            properties,
            events,
            children,
            ..
        } => {
            for property in properties {
                collect_expression(&property.value, programs);
            }
            for event in events {
                for statement in &event.statements {
                    match statement {
                        argui_dsl_ir::IrStatement::Expression(value)
                        | argui_dsl_ir::IrStatement::Assignment { value, .. }
                        | argui_dsl_ir::IrStatement::Return(Some(value))
                        | argui_dsl_ir::IrStatement::SetThemeMode(value) => {
                            collect_expression(value, programs);
                        }
                        argui_dsl_ir::IrStatement::Return(None) => {}
                    }
                }
            }
            for child in children {
                collect_node(child, programs);
            }
        }
        IrNode::Repeater {
            model, key, body, ..
        } => {
            collect_expression(model, programs);
            collect_expression(key, programs);
            for child in body {
                collect_node(child, programs);
            }
        }
        IrNode::Conditional {
            condition,
            then_body,
            else_body,
            ..
        } => {
            collect_expression(condition, programs);
            for child in then_body.iter().chain(else_body) {
                collect_node(child, programs);
            }
        }
        IrNode::Slot { .. } => {}
    }
}

/// Compiles an expression and all nested expressions as addressable programs.
fn collect_expression(expression: &IrExpression, programs: &mut HashMap<ExpressionId, Program>) {
    programs
        .entry(expression.id)
        .or_insert_with(|| Program::compile(expression));
    match &expression.kind {
        argui_dsl_ir::IrExpressionKind::FieldRead { base, .. }
        | argui_dsl_ir::IrExpressionKind::Unary { operand: base, .. } => {
            collect_expression(base, programs);
        }
        argui_dsl_ir::IrExpressionKind::BuiltinCall { arguments, .. }
        | argui_dsl_ir::IrExpressionKind::CallbackCall { arguments, .. }
        | argui_dsl_ir::IrExpressionKind::Array(arguments) => {
            for argument in arguments {
                collect_expression(argument, programs);
            }
        }
        argui_dsl_ir::IrExpressionKind::Binary { left, right, .. } => {
            collect_expression(left, programs);
            collect_expression(right, programs);
        }
        argui_dsl_ir::IrExpressionKind::Conditional {
            condition,
            then_value,
            else_value,
        } => {
            collect_expression(condition, programs);
            collect_expression(then_value, programs);
            collect_expression(else_value, programs);
        }
        _ => {}
    }
}
