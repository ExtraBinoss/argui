use std::collections::HashMap;

use argui_dsl_semantic::Module;
use argui_dsl_syntax::{Span, SyntaxKind, SyntaxNode};

use crate::{
    AssignmentOperator, ComponentId, EventTargetId, IrAnimation, IrAssignmentTarget, IrEffect,
    IrElementTarget, IrEventBinding, IrExpression, IrExpressionKind, IrNode, IrPropertyBinding,
    IrState, IrStatement, IrType, LocalId, LowerError, PropertyTargetId, SiteId, SourceInfo,
    declaration, expression, id::derive, lower::ComponentMembers, lower::Tables, site,
};

mod animation;
mod effect;
mod handler;
pub(crate) mod reference;

/// Resolved properties and events for one element target.
struct Target {
    element: IrElementTarget,
    properties: HashMap<String, (PropertyTargetId, IrType)>,
    events: HashMap<String, (EventTargetId, Vec<IrType>)>,
}

/// Stateful component-tree lowering context.
pub(crate) struct VisualLowerer<'a> {
    module: &'a Module,
    component: ComponentId,
    members: &'a ComponentMembers,
    tables: &'a Tables,
    effects: &'a [IrEffect],
    schema: &'a argui_schema::SchemaRegistry,
    assets: &'a mut HashMap<String, crate::IrAsset>,
    errors: &'a mut Vec<LowerError>,
    locals: HashMap<String, (LocalId, IrType)>,
    sites: HashMap<SiteId, Span>,
    states: Vec<IrState>,
    animations: Vec<IrAnimation>,
    references: expression::References,
    reference_sites: HashMap<String, SiteId>,
}

impl<'a> VisualLowerer<'a> {
    /// Creates a lowering context for `component` in `module` using resolved
    /// `members`, `tables`, `effects`, and `schema`. The context adds reached
    /// `assets` and lowering `errors`; returns an empty visual state.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        module: &'a Module,
        component: ComponentId,
        members: &'a ComponentMembers,
        tables: &'a Tables,
        effects: &'a [IrEffect],
        schema: &'a argui_schema::SchemaRegistry,
        assets: &'a mut HashMap<String, crate::IrAsset>,
        errors: &'a mut Vec<LowerError>,
    ) -> Self {
        Self {
            module,
            component,
            members,
            tables,
            effects,
            schema,
            assets,
            errors,
            locals: HashMap::new(),
            sites: HashMap::new(),
            states: Vec::new(),
            animations: Vec::new(),
            references: HashMap::new(),
            reference_sites: HashMap::new(),
        }
    }

    /// Lowers direct component visual content and component-level behavior.
    pub(crate) fn component_body(&mut self, node: &SyntaxNode) -> Vec<IrNode> {
        self.references =
            reference::collect(self.component, self.module, self.schema, self.tables, node);
        self.reference_sites = reference::collect_sites(self.component, self.module, node);
        self.lower_behavior(node, None, &self.component_target());
        self.visual_children(node)
    }

    /// Returns behavior accumulated while recursively lowering the tree.
    pub(crate) fn finish(self) -> (Vec<IrState>, Vec<IrAnimation>) {
        (self.states, self.animations)
    }

    /// Lowers immediate visual children without descending through unrelated nodes.
    fn visual_children(&mut self, node: &SyntaxNode) -> Vec<IrNode> {
        let mut output = Vec::new();
        for child in node.children() {
            let lowered = match child.kind() {
                SyntaxKind::Element => self.element(&child),
                SyntaxKind::ForExpr => self.repeater(&child),
                SyntaxKind::IfExpr => self.conditional(&child),
                SyntaxKind::PathExpr => self.slot(&child),
                SyntaxKind::Block | SyntaxKind::ElseBranch => {
                    output.extend(self.visual_children(&child));
                    None
                }
                _ => None,
            };
            if let Some(lowered) = lowered {
                output.push(lowered);
            }
        }
        output
    }

    /// Lowers a resolved native or DSL component element.
    fn element(&mut self, node: &SyntaxNode) -> Option<IrNode> {
        let name = direct_identifier(node)?;
        let target = self.target(&name, node)?;
        let site = self.register_site(node);
        let properties = node
            .children()
            .filter(|child| {
                matches!(
                    child.kind(),
                    SyntaxKind::PropertyAssignment | SyntaxKind::TwoWayBinding
                )
            })
            .filter_map(|assignment| self.property(&assignment, site, &target))
            .collect();
        let effect = node
            .children()
            .find(|child| child.kind() == SyntaxKind::EffectApplication)
            .and_then(|application| self.effect_application(&application, site));
        let events = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::EventBlock)
            .filter_map(|event| self.event(&event, site, &target))
            .collect();
        self.lower_behavior(node, Some(site), &target);
        let children = self.visual_children(node);
        Some(IrNode::Element {
            site,
            target: target.element,
            source_id: site::explicit_id(node),
            properties,
            effect,
            events,
            children,
            source: self.source(node, Some(site)),
        })
    }

    /// Lowers a keyed repeater and introduces its typed local binding.
    fn repeater(&mut self, node: &SyntaxNode) -> Option<IrNode> {
        let site = self.register_site(node);
        let expressions = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::Expr)
            .collect::<Vec<_>>();
        let model = expressions
            .first()
            .map(|value| self.expression(value, Some(site)))?;
        let local_type = match &model.value_type {
            IrType::Model(inner) | IrType::Array(inner) => (**inner).clone(),
            _ => IrType::Unknown,
        };
        let binding = identifier_after(node, SyntaxKind::ForKw)?;
        let local = LocalId::from_raw(derive(site.raw(), "repeater-local", 0));
        let previous = self.locals.insert(binding.clone(), (local, local_type));
        let key = expressions
            .get(1)
            .map(|value| self.expression(value, Some(site)))?;
        let body = node
            .children()
            .find(|child| child.kind() == SyntaxKind::Block)
            .map_or_else(Vec::new, |block| self.visual_children(&block));
        if let Some(previous) = previous {
            self.locals.insert(binding, previous);
        } else {
            self.locals.remove(&binding);
        }
        Some(IrNode::Repeater {
            site,
            local,
            model,
            key,
            body,
            source: self.source(node, Some(site)),
        })
    }

    /// Lowers a conditional and its optional else branch.
    fn conditional(&mut self, node: &SyntaxNode) -> Option<IrNode> {
        let site = self.register_site(node);
        let condition = node
            .children()
            .find(|child| child.kind() == SyntaxKind::Expr)
            .map(|value| self.expression(&value, Some(site)))?;
        let then_body = node
            .children()
            .find(|child| child.kind() == SyntaxKind::Block)
            .map_or_else(Vec::new, |block| self.visual_children(&block));
        let else_body = node
            .children()
            .find(|child| child.kind() == SyntaxKind::ElseBranch)
            .map_or_else(Vec::new, |branch| self.visual_children(&branch));
        Some(IrNode::Conditional {
            site,
            condition,
            then_body,
            else_body,
            source: self.source(node, Some(site)),
        })
    }

    /// Lowers a named component slot reference.
    fn slot(&mut self, node: &SyntaxNode) -> Option<IrNode> {
        let name = direct_identifier(node)?;
        let slot = self.members.slots.get(&name).copied()?;
        let site = self.register_site(node);
        Some(IrNode::Slot {
            site,
            slot,
            source: self.source(node, Some(site)),
        })
    }

    /// Lowers one element property assignment.
    fn property(
        &mut self,
        node: &SyntaxNode,
        site: SiteId,
        target: &Target,
    ) -> Option<IrPropertyBinding> {
        let name = declaration::direct_name_or_theme(node)?;
        let (target, _) = target.properties.get(&name)?;
        Some(IrPropertyBinding {
            target: *target,
            value: declaration::child_expression(node)
                .map(|value| self.expression(&value, Some(site)))?,
            two_way: node.kind() == SyntaxKind::TwoWayBinding,
            source: self.source(node, Some(site)),
        })
    }

    /// Lowers one event block, typed payload locals, and restricted statements.
    fn event(
        &mut self,
        node: &SyntaxNode,
        site: SiteId,
        target: &Target,
    ) -> Option<IrEventBinding> {
        let name = identifier_after(node, SyntaxKind::OnKw)?;
        let (target, expected) = target.events.get(&name)?.clone();
        let parameter_names = event_parameters(node);
        let mut previous = Vec::new();
        let parameters = parameter_names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let id = LocalId::from_raw(derive(site.raw(), "event-parameter", index as u64));
                previous.push((
                    name.clone(),
                    self.locals
                        .insert(name.clone(), (id, expected[index].clone())),
                ));
                id
            })
            .collect();
        let statements = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::Statement)
            .filter_map(|statement| self.statement(&statement, site))
            .collect();
        for (name, value) in previous {
            if let Some(value) = value {
                self.locals.insert(name, value);
            } else {
                self.locals.remove(&name);
            }
        }
        Some(IrEventBinding {
            target,
            parameters,
            statements,
            source: self.source(node, Some(site)),
        })
    }

    /// Lowers a restricted handler statement including property mutations.
    fn statement(&mut self, node: &SyntaxNode, site: SiteId) -> Option<IrStatement> {
        let expressions = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::Expr)
            .collect::<Vec<_>>();
        if has_token(node, SyntaxKind::ReturnKw) {
            return Some(IrStatement::Return(
                expressions
                    .first()
                    .map(|value| self.expression(value, Some(site))),
            ));
        }
        if let Some(mode) = expressions.first().and_then(handler::theme_mode_call) {
            return Some(IrStatement::SetThemeMode(
                self.expression(&mode, Some(site)),
            ));
        }
        if let Some(action) = expressions.first().and_then(handler::host_action_call) {
            return Some(action);
        }
        if let Some((name, x, y)) = expressions.first().and_then(handler::scroll_to_call) {
            let Some(site) = self.reference_sites.get(&name).copied() else {
                self.errors.push(LowerError::new(
                    format!("unresolved scroll target `#{name}`"),
                    Span::new(self.module.file, node.text_range()),
                ));
                return None;
            };
            return Some(IrStatement::ScrollTo {
                site,
                x: self.expression(&x, Some(site)),
                y: self.expression(&y, Some(site)),
            });
        }
        let operator = assignment_operator(node);
        if let Some(operator) = operator {
            let destination = expressions
                .first()
                .map(|value| self.expression(value, Some(site)))?;
            let target = match destination.kind {
                IrExpressionKind::PropertyRead(id) => IrAssignmentTarget::Property(id),
                IrExpressionKind::LocalRead(id) => IrAssignmentTarget::Local(id),
                _ => return None,
            };
            let value = expressions
                .get(1)
                .map(|value| self.expression(value, Some(site)))?;
            return Some(IrStatement::Assignment {
                target,
                operator,
                value,
            });
        }
        expressions
            .first()
            .map(|value| IrStatement::Expression(self.expression(value, Some(site))))
    }

    /// Lowers state and animation clauses owned by a component or element.
    fn lower_behavior(&mut self, node: &SyntaxNode, owner: Option<SiteId>, target: &Target) {
        for states in node
            .children()
            .filter(|child| child.kind() == SyntaxKind::StatesBlock)
        {
            for state in states
                .children()
                .filter(|child| child.kind() == SyntaxKind::StateDecl)
            {
                let site = self.register_site(&state);
                let Some(condition) = state
                    .children()
                    .find(|child| child.kind() == SyntaxKind::Expr)
                    .map(|value| self.expression(&value, Some(site)))
                else {
                    continue;
                };
                let assignments = state
                    .descendants()
                    .filter(|child| child.kind() == SyntaxKind::PropertyAssignment)
                    .filter_map(|assignment| {
                        let name = declaration::direct_name_or_theme(&assignment)?;
                        let target = target.properties.get(&name)?.0;
                        let value = declaration::child_expression(&assignment)
                            .map(|value| self.expression(&value, Some(site)))?;
                        Some((target, value))
                    })
                    .collect();
                self.states.push(IrState {
                    site,
                    owner,
                    condition,
                    assignments,
                    source: self.source(&state, Some(site)),
                });
            }
        }
        self.lower_animations(node, owner, target);
    }

    /// Resolves one element target and all assignable members.
    fn target(&mut self, name: &str, node: &SyntaxNode) -> Option<Target> {
        if let Some(native) = self.module.native_scope.get(name).copied() {
            let Some(schema) = self.schema.schema(native) else {
                self.errors.push(LowerError::new(
                    format!("native schema `{name}` is unavailable during IR lowering"),
                    Span::new(self.module.file, node.text_range()),
                ));
                return None;
            };
            return Some(Target {
                element: IrElementTarget::Native(native),
                properties: schema
                    .properties
                    .iter()
                    .map(|property| {
                        (
                            property.name.as_str().to_string(),
                            (
                                PropertyTargetId::Native(property.id),
                                IrType::from_schema(property.value_type),
                            ),
                        )
                    })
                    .collect(),
                events: schema
                    .events
                    .iter()
                    .map(|event| {
                        (
                            event.name.as_str().to_string(),
                            (
                                EventTargetId::Native(event.id),
                                event.payload.map(IrType::from_schema).into_iter().collect(),
                            ),
                        )
                    })
                    .collect(),
            });
        }
        let symbol = self.module.scope.get(name)?;
        let component = *self.tables.components.get(symbol)?;
        let members = self.tables.component_members.get(symbol)?;
        Some(Target {
            element: IrElementTarget::Component(component),
            properties: members
                .properties
                .iter()
                .map(|(name, (id, value_type))| {
                    (
                        name.clone(),
                        (PropertyTargetId::Component(*id), value_type.clone()),
                    )
                })
                .collect(),
            events: members
                .callbacks
                .iter()
                .map(|(name, (id, parameters, _))| {
                    (
                        name.clone(),
                        (EventTargetId::Component(*id), parameters.clone()),
                    )
                })
                .collect(),
        })
    }

    /// Constructs the current component itself as a behavior assignment target.
    fn component_target(&self) -> Target {
        Target {
            element: IrElementTarget::Component(self.component),
            properties: self
                .members
                .properties
                .iter()
                .map(|(name, (id, value_type))| {
                    (
                        name.clone(),
                        (PropertyTargetId::Component(*id), value_type.clone()),
                    )
                })
                .collect(),
            events: self
                .members
                .callbacks
                .iter()
                .map(|(name, (id, parameters, _))| {
                    (
                        name.clone(),
                        (EventTargetId::Component(*id), parameters.clone()),
                    )
                })
                .collect(),
        }
    }

    /// Lowers an expression using the current component and local environment.
    fn expression(&mut self, node: &SyntaxNode, site: Option<SiteId>) -> IrExpression {
        let mut context = expression::Context {
            file: self.module.file,
            module_path: &self.module.path,
            component: Some(self.component),
            site,
            properties: &self.members.properties,
            callbacks: &self.members.callbacks,
            locals: &self.locals,
            tokens: &self.tables.tokens,
            fields: &self.tables.named_fields,
            references: Some(&self.references),
            assets: self.assets,
            errors: self.errors,
        };
        expression::lower(node, &mut context)
    }

    /// Registers a stable site and diagnoses explicit/structural collisions.
    fn register_site(&mut self, node: &SyntaxNode) -> SiteId {
        let site = self.site(node);
        let span = Span::new(self.module.file, node.text_range());
        if let Some(previous) = self.sites.insert(site, span)
            && previous != span
        {
            self.errors.push(LowerError::new(
                "visual source sites resolve to the same retained identity; add distinct `#id` values",
                span,
            ));
        }
        site
    }

    /// Derives a stable source-site identity.
    fn site(&self, node: &SyntaxNode) -> SiteId {
        site::identify(self.component, node)
    }

    /// Creates component/site-scoped source metadata.
    fn source(&self, node: &SyntaxNode, site: Option<SiteId>) -> SourceInfo {
        SourceInfo::new(
            Span::new(self.module.file, node.text_range()),
            Some(self.component),
            site,
        )
    }
}

/// Returns the first direct identifier.
fn direct_identifier(node: &SyntaxNode) -> Option<String> {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text().to_string())
}

/// Returns the identifier immediately following a direct keyword.
fn identifier_after(node: &SyntaxNode, keyword: SyntaxKind) -> Option<String> {
    let mut seen = false;
    for token in node
        .children_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| !token.kind().is_trivia())
    {
        if seen && token.kind() == SyntaxKind::Ident {
            return Some(token.text().to_string());
        }
        seen |= token.kind() == keyword;
    }
    None
}

/// Returns optional event parameter names from the handler's direct syntax.
///
/// * `node` — event block containing an optional parenthesized name list.
///
/// Returns names in payload order, or an empty list when none are bound.
fn event_parameters(node: &SyntaxNode) -> Vec<String> {
    let mut inside = false;
    let mut names = Vec::new();
    for token in node
        .children_with_tokens()
        .filter_map(|element| element.into_token())
    {
        match token.kind() {
            SyntaxKind::LParen => inside = true,
            SyntaxKind::RParen => break,
            SyntaxKind::Ident if inside => names.push(token.text().to_string()),
            _ => {}
        }
    }
    names
}

/// Returns whether a node directly owns a token kind.
fn has_token(node: &SyntaxNode, kind: SyntaxKind) -> bool {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
        .any(|token| token.kind() == kind)
}

/// Returns a handler assignment operator when present.
fn assignment_operator(node: &SyntaxNode) -> Option<AssignmentOperator> {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
        .find_map(|token| match token.kind() {
            SyntaxKind::Eq => Some(AssignmentOperator::Set),
            SyntaxKind::PlusEq => Some(AssignmentOperator::Add),
            SyntaxKind::MinusEq => Some(AssignmentOperator::Subtract),
            SyntaxKind::StarEq => Some(AssignmentOperator::Multiply),
            SyntaxKind::SlashEq => Some(AssignmentOperator::Divide),
            _ => None,
        })
}
