use std::collections::{HashMap, HashSet, VecDeque};

use argui_dsl_ir::{
    AssetId, ComponentId, EffectId, IrExpression, IrExpressionKind, IrNode, IrProject, IrType,
    StyleId, ThemeId, TokenId,
};
use argui_dsl_semantic::{DefinitionKind, SemanticProject, SymbolId};

/// Transitive release roots retained by dead-code elimination.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Reachability {
    pub components: HashSet<ComponentId>,
    pub structs: HashSet<SymbolId>,
    pub enums: HashSet<SymbolId>,
    pub themes: HashSet<ThemeId>,
    pub tokens: HashSet<TokenId>,
    pub styles: HashSet<StyleId>,
    pub effects: HashSet<EffectId>,
    pub assets: HashSet<AssetId>,
}

impl Reachability {
    /// Computes transitive reachability from exported definitions in `entry_module`.
    ///
    /// * `semantic` — analyzed project used to locate public release roots.
    /// * `ir` — normalized project whose references form the reachability graph.
    /// * `entry_module` — canonical module path selected by the application build.
    #[must_use]
    pub fn analyze(semantic: &SemanticProject, ir: &IrProject, entry_module: &str) -> Self {
        let mut result = Self::default();
        let mut queue = VecDeque::new();
        if let Some(entry) = semantic
            .modules
            .iter()
            .find(|module| module.path == entry_module)
        {
            for definition in entry
                .definitions
                .iter()
                .filter(|definition| definition.exported)
            {
                match definition.kind {
                    DefinitionKind::Component(_) => {
                        queue.push_back(ComponentId::from_raw(definition.id.raw()));
                    }
                    DefinitionKind::Struct(_) => {
                        result.structs.insert(definition.id);
                    }
                    DefinitionKind::Enum(_) => {
                        result.enums.insert(definition.id);
                    }
                    DefinitionKind::Theme(_) => {
                        result.themes.insert(ThemeId::from_raw(definition.id.raw()));
                    }
                    DefinitionKind::Style(_) => {
                        result.styles.insert(StyleId::from_raw(definition.id.raw()));
                    }
                    DefinitionKind::Effect(_) => {
                        result
                            .effects
                            .insert(EffectId::from_raw(definition.id.raw()));
                    }
                    DefinitionKind::Function(_) => {}
                }
            }
        }
        let components = ir
            .components
            .iter()
            .map(|component| (component.id, component))
            .collect::<HashMap<_, _>>();
        while let Some(id) = queue.pop_front() {
            if !result.components.insert(id) {
                continue;
            }
            let Some(component) = components.get(&id) else {
                continue;
            };
            for property in &component.properties {
                visit_type(&property.value_type, &mut result);
                if let Some(default) = &property.default {
                    visit_expression(default, &mut result);
                }
            }
            for node in &component.body {
                visit_node(node, &mut result, &mut queue);
            }
            for state in &component.states {
                visit_expression(&state.condition, &mut result);
                for (_, value) in &state.assignments {
                    visit_expression(value, &mut result);
                }
            }
            for animation in &component.animations {
                for parameter in &animation.parameters {
                    visit_expression(&parameter.value, &mut result);
                }
                for frame in &animation.keyframes {
                    visit_expression(&frame.value, &mut result);
                }
            }
        }
        for style in &ir.styles {
            if result.styles.contains(&style.id) {
                for binding in &style.properties {
                    visit_expression(&binding.value, &mut result);
                }
                for state in &style.states {
                    for binding in &state.properties {
                        visit_expression(&binding.value, &mut result);
                    }
                }
            }
        }
        for effect in &ir.effects {
            if result.effects.contains(&effect.id) {
                result.assets.insert(effect.shader);
            }
        }
        loop {
            let previous = (result.themes.len(), result.tokens.len());
            for theme in &ir.themes {
                if result.themes.contains(&theme.id)
                    || theme
                        .tokens
                        .iter()
                        .any(|token| result.tokens.contains(&token.id))
                    || theme.modes.iter().any(|mode| {
                        mode.overrides
                            .iter()
                            .any(|(token, _)| result.tokens.contains(token))
                    })
                {
                    result.themes.insert(theme.id);
                    for token in &theme.tokens {
                        visit_expression(&token.default, &mut result);
                    }
                    for mode in &theme.modes {
                        for (_, value) in &mode.overrides {
                            visit_expression(value, &mut result);
                        }
                    }
                }
            }
            if previous == (result.themes.len(), result.tokens.len()) {
                break;
            }
        }
        result
    }

    /// Removes declarations and assets unreachable from the selected entry roots.
    ///
    /// * `ir` — complete lowered project to reduce for transport or release use.
    pub fn prune(&self, ir: &mut IrProject) {
        ir.structs
            .retain(|value| self.structs.contains(&value.symbol));
        ir.enums.retain(|value| self.enums.contains(&value.symbol));
        ir.components
            .retain(|value| self.components.contains(&value.id));
        ir.themes.retain(|value| self.themes.contains(&value.id));
        ir.styles.retain(|value| self.styles.contains(&value.id));
        ir.effects.retain(|value| self.effects.contains(&value.id));
        ir.assets.retain(|value| self.assets.contains(&value.id));
        let definitions = ir
            .structs
            .iter()
            .map(|value| value.symbol.raw())
            .chain(ir.enums.iter().map(|value| value.symbol.raw()))
            .chain(ir.components.iter().map(|value| value.id.raw()))
            .chain(ir.themes.iter().map(|value| value.id.raw()))
            .chain(ir.styles.iter().map(|value| value.id.raw()))
            .chain(ir.effects.iter().map(|value| value.id.raw()))
            .collect::<HashSet<_>>();
        for module in &mut ir.modules {
            module
                .definitions
                .retain(|definition| definitions.contains(&definition.raw()));
        }
    }
}

/// Visits a resolved user type and its nested collection types.
fn visit_type(value: &IrType, output: &mut Reachability) {
    match value {
        IrType::Struct { symbol, .. } => {
            output.structs.insert(*symbol);
        }
        IrType::Enum(symbol) => {
            output.enums.insert(*symbol);
        }
        IrType::Optional(inner) | IrType::Array(inner) | IrType::Model(inner) => {
            visit_type(inner, output);
        }
        IrType::Callback { parameters, result } => {
            for parameter in parameters {
                visit_type(parameter, output);
            }
            visit_type(result, output);
        }
        _ => {}
    }
}

/// Visits one declarative node and all nested references.
fn visit_node(node: &IrNode, output: &mut Reachability, queue: &mut VecDeque<ComponentId>) {
    match node {
        IrNode::Element {
            target,
            properties,
            effect,
            events,
            children,
            ..
        } => {
            if let argui_dsl_ir::IrElementTarget::Component(component) = target {
                queue.push_back(*component);
            }
            for property in properties {
                visit_expression(&property.value, output);
            }
            if let Some(binding) = effect {
                output.effects.insert(binding.effect);
                for parameter in &binding.parameters {
                    visit_expression(&parameter.value, output);
                }
            }
            for event in events {
                for statement in &event.statements {
                    visit_statement(statement, output);
                }
            }
            for child in children {
                visit_node(child, output, queue);
            }
        }
        IrNode::Repeater {
            model, key, body, ..
        } => {
            visit_expression(model, output);
            visit_expression(key, output);
            for child in body {
                visit_node(child, output, queue);
            }
        }
        IrNode::Conditional {
            condition,
            then_body,
            else_body,
            ..
        } => {
            visit_expression(condition, output);
            for child in then_body.iter().chain(else_body) {
                visit_node(child, output, queue);
            }
        }
        IrNode::Slot { fallback: body, .. } | IrNode::SlotContent { body, .. } => {
            for child in body {
                visit_node(child, output, queue);
            }
        }
    }
}

/// Visits assets and nested operations referenced by an expression.
fn visit_expression(expression: &IrExpression, output: &mut Reachability) {
    match &expression.kind {
        IrExpressionKind::Asset(asset) => {
            output.assets.insert(*asset);
        }
        IrExpressionKind::FieldRead { base, .. }
        | IrExpressionKind::Unary { operand: base, .. } => visit_expression(base, output),
        IrExpressionKind::BuiltinCall { arguments, .. }
        | IrExpressionKind::CallbackCall { arguments, .. }
        | IrExpressionKind::Array(arguments) => {
            for argument in arguments {
                visit_expression(argument, output);
            }
        }
        IrExpressionKind::Struct { symbol, fields } => {
            output.structs.insert(*symbol);
            for (_, value) in fields {
                visit_expression(value, output);
            }
        }
        IrExpressionKind::Index { base, index } => {
            visit_expression(base, output);
            visit_expression(index, output);
        }
        IrExpressionKind::Binary { left, right, .. } => {
            visit_expression(left, output);
            visit_expression(right, output);
        }
        IrExpressionKind::Conditional {
            condition,
            then_value,
            else_value,
        } => {
            visit_expression(condition, output);
            visit_expression(then_value, output);
            visit_expression(else_value, output);
        }
        IrExpressionKind::EnumVariant { symbol, .. } => {
            output.enums.insert(*symbol);
        }
        IrExpressionKind::TokenRead(token) => {
            output.tokens.insert(*token);
        }
        IrExpressionKind::Constant(_)
        | IrExpressionKind::PropertyRead(_)
        | IrExpressionKind::ChildPropertyRead { .. }
        | IrExpressionKind::ObservedRead { .. }
        | IrExpressionKind::LocalRead(_) => {}
    }
}

/// Visits expressions nested in one event statement.
fn visit_statement(statement: &argui_dsl_ir::IrStatement, output: &mut Reachability) {
    match statement {
        argui_dsl_ir::IrStatement::Let { value, .. }
        | argui_dsl_ir::IrStatement::Expression(value)
        | argui_dsl_ir::IrStatement::Assignment { value, .. }
        | argui_dsl_ir::IrStatement::Return(Some(value))
        | argui_dsl_ir::IrStatement::SetThemeMode(value) => visit_expression(value, output),
        argui_dsl_ir::IrStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            visit_expression(condition, output);
            for statement in then_body.iter().chain(else_body) {
                visit_statement(statement, output);
            }
        }
        argui_dsl_ir::IrStatement::ScrollTo { x, y, .. } => {
            visit_expression(x, output);
            visit_expression(y, output);
        }
        argui_dsl_ir::IrStatement::Return(None)
        | argui_dsl_ir::IrStatement::FocusNext
        | argui_dsl_ir::IrStatement::FocusPrevious
        | argui_dsl_ir::IrStatement::PreventDefault
        | argui_dsl_ir::IrStatement::StopPropagation => {}
    }
}
