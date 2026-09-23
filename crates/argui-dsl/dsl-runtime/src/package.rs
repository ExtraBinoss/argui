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
    /// Referenced native sites indexed by owning component for render subscriptions.
    pub observed_sites: HashMap<argui_dsl_ir::ComponentId, Vec<argui_dsl_ir::SiteId>>,
    /// Child output dependencies computed once for this immutable generation.
    pub(crate) child_reference_sites:
        HashMap<argui_dsl_ir::ComponentId, std::collections::HashSet<argui_dsl_ir::SiteId>>,
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
        mut assets: HashMap<AssetId, AssetPayload>,
    ) -> Result<Self, RuntimeError> {
        for asset in &ir.assets {
            if let Some(bytes) = &asset.inline_bytes {
                match assets.entry(asset.id) {
                    std::collections::hash_map::Entry::Occupied(entry) => {
                        if entry.get().bytes.as_ref() != bytes.as_slice() {
                            return Err(RuntimeError::IncompatiblePackage(format!(
                                "inline asset {} differs from its wire payload",
                                asset.id.raw()
                            )));
                        }
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(AssetPayload::new(1, bytes.clone()));
                    }
                }
            }
        }
        for asset in &ir.assets {
            if !assets.contains_key(&asset.id) {
                return Err(RuntimeError::MissingAsset(asset.id.raw()));
            }
        }
        let mut shader_hashes = HashMap::new();
        for effect in &ir.effects {
            if let Some(parameter) = effect.parameters.iter().find(|parameter| {
                !matches!(
                    parameter.value_type,
                    argui_dsl_ir::IrType::Float
                        | argui_dsl_ir::IrType::Int
                        | argui_dsl_ir::IrType::Bool
                        | argui_dsl_ir::IrType::Color
                        | argui_dsl_ir::IrType::Length
                        | argui_dsl_ir::IrType::Transform
                )
            }) {
                return Err(RuntimeError::InvalidShader(format!(
                    "unsupported effect parameter type {:?}",
                    parameter.value_type
                )));
            }
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
                        parameter.name.clone(),
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
        let mut observed_sites = HashMap::<_, std::collections::HashSet<_>>::new();
        for program in programs.values() {
            if let Some(owner) = program.owner {
                for instruction in &program.instructions {
                    if let crate::Instruction::Observed(site, _) = instruction {
                        observed_sites.entry(owner).or_default().insert(*site);
                    }
                }
            }
        }
        let observed_sites = observed_sites
            .into_iter()
            .map(|(owner, sites)| {
                let mut sites = sites.into_iter().collect::<Vec<_>>();
                sites.sort_unstable();
                (owner, sites)
            })
            .collect();
        let child_reference_sites = ir
            .components
            .iter()
            .filter_map(|component| {
                let sites = component.referenced_child_sites();
                (!sites.is_empty()).then_some((component.id, sites))
            })
            .collect();
        Ok(Self {
            generation: generation.max(1),
            public_api_hash,
            roots: Vec::new(),
            ir: Arc::new(ir),
            programs,
            observed_sites,
            child_reference_sites,
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
            effect,
            events,
            children,
            ..
        } => {
            for property in properties {
                collect_expression(&property.value, programs);
            }
            if let Some(binding) = effect {
                for parameter in &binding.parameters {
                    collect_expression(&parameter.value, programs);
                }
            }
            for event in events {
                for statement in &event.statements {
                    collect_statement(statement, programs);
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
        IrNode::Slot { fallback: body, .. } | IrNode::SlotContent { body, .. } => {
            for child in body {
                collect_node(child, programs);
            }
        }
    }
}

/// Collects every expression reachable from one lexical handler statement.
fn collect_statement(
    statement: &argui_dsl_ir::IrStatement,
    programs: &mut HashMap<ExpressionId, Program>,
) {
    use argui_dsl_ir::IrStatement;
    match statement {
        IrStatement::Let { value, .. }
        | IrStatement::Expression(value)
        | IrStatement::Assignment { value, .. }
        | IrStatement::Return(Some(value))
        | IrStatement::SetThemeMode(value) => collect_expression(value, programs),
        IrStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            collect_expression(condition, programs);
            for statement in then_body.iter().chain(else_body) {
                collect_statement(statement, programs);
            }
        }
        IrStatement::ScrollTo { x, y, .. } => {
            collect_expression(x, programs);
            collect_expression(y, programs);
        }
        IrStatement::Return(None)
        | IrStatement::FocusNext
        | IrStatement::FocusPrevious
        | IrStatement::PreventDefault
        | IrStatement::StopPropagation => {}
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
        argui_dsl_ir::IrExpressionKind::Struct { fields, .. } => {
            for (_, value) in fields {
                collect_expression(value, programs);
            }
        }
        argui_dsl_ir::IrExpressionKind::Index { base, index } => {
            collect_expression(base, programs);
            collect_expression(index, programs);
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
