//! Event handler lowering with lexical local scopes.

use super::*;

impl VisualLowerer<'_> {
    /// Lowers one event block, typed payload locals, and restricted statements.
    pub(super) fn event(
        &mut self,
        node: &SyntaxNode,
        site: SiteId,
        target: &Target,
    ) -> Option<IrEventBinding> {
        let name = identifier_after(node, SyntaxKind::OnKw)?;
        let (target, expected) = target.events.get(&name)?.clone();
        let parameter_names = event_parameters(node);
        let previous = self.locals.clone();
        let parameters = parameter_names
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let id = LocalId::from_raw(derive(site.raw(), "event-parameter", index as u64));
                self.locals
                    .insert(name.clone(), (id, expected[index].clone()));
                id
            })
            .collect();
        let statements = node
            .children()
            .filter(|child| {
                matches!(
                    child.kind(),
                    SyntaxKind::Statement | SyntaxKind::IfStatement
                )
            })
            .filter_map(|statement| self.statement(&statement, site))
            .collect();
        self.locals = previous;
        Some(IrEventBinding {
            target,
            parameters,
            statements,
            source: self.source(node, Some(site)),
        })
    }

    /// Lowers a checked lexical handler statement or conditional branch.
    fn statement(&mut self, node: &SyntaxNode, site: SiteId) -> Option<IrStatement> {
        if node.kind() == SyntaxKind::IfStatement {
            let condition = node
                .children()
                .find(|child| child.kind() == SyntaxKind::Expr)?;
            let mut blocks = node
                .children()
                .filter(|child| child.kind() == SyntaxKind::Block);
            let condition = self.expression(&condition, Some(site));
            let then_body = blocks
                .next()
                .map(|block| self.handler_block(&block, site))
                .unwrap_or_default();
            let else_body = blocks
                .next()
                .map(|block| self.handler_block(&block, site))
                .unwrap_or_default();
            return Some(IrStatement::If {
                condition,
                then_body,
                else_body,
            });
        }
        let expressions = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::Expr)
            .collect::<Vec<_>>();
        if has_token(node, SyntaxKind::LetKw) {
            let name = expressions
                .first()?
                .children()
                .find(|child| child.kind() == SyntaxKind::PathExpr)
                .and_then(|path| direct_identifier(&path))?;
            let initializer = expressions
                .get(1)
                .map(|value| self.expression(value, Some(site)))?;
            let local = LocalId::from_raw(derive(
                site.raw(),
                "handler-local",
                u64::from(u32::from(node.text_range().start())),
            ));
            self.locals
                .insert(name, (local, initializer.value_type.clone()));
            return Some(IrStatement::Let {
                local,
                value: initializer,
            });
        }
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

    /// Lowers a branch body with a fresh lexical local namespace.
    fn handler_block(&mut self, node: &SyntaxNode, site: SiteId) -> Vec<IrStatement> {
        let previous = self.locals.clone();
        let statements = node
            .children()
            .filter(|child| {
                matches!(
                    child.kind(),
                    SyntaxKind::Statement | SyntaxKind::IfStatement
                )
            })
            .filter_map(|statement| self.statement(&statement, site))
            .collect();
        self.locals = previous;
        statements
    }
}
