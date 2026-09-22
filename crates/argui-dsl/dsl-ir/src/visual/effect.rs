//! Lowers visual effect applications to stable IDs and typed ordered arguments.

use std::collections::HashMap;

use argui_dsl_syntax::{Span, SyntaxKind, SyntaxNode};

use crate::{IrEffectArgument, IrEffectBinding, IrEffectScope, LowerError, SiteId};

use super::VisualLowerer;

impl VisualLowerer<'_> {
    /// Lowers a validated named effect on one visual site.
    ///
    /// `node` contains the authored argument expressions and `site` identifies
    /// the element whose output receives the effect. Returns `None` and records
    /// an error if the effect declaration is unavailable.
    pub(super) fn effect_application(
        &mut self,
        node: &SyntaxNode,
        site: SiteId,
    ) -> Option<IrEffectBinding> {
        let name = node
            .children_with_tokens()
            .filter_map(|item| item.into_token())
            .find(|token| token.kind() == SyntaxKind::Ident)?
            .text()
            .to_string();
        let definition = self.module.scope.get(&name)?;
        let Some(effect) = self
            .effects
            .iter()
            .find(|effect| effect.id.raw() == definition.raw())
            .cloned()
        else {
            self.errors.push(LowerError::new(
                format!("effect `{name}` is unavailable during IR lowering"),
                Span::new(self.module.file, node.text_range()),
            ));
            return None;
        };
        let supplied = node
            .children()
            .filter(|child| child.kind() == SyntaxKind::PropertyAssignment)
            .filter_map(|assignment| {
                let name = assignment
                    .children_with_tokens()
                    .filter_map(|item| item.into_token())
                    .find(|token| token.kind() == SyntaxKind::Ident)?
                    .text()
                    .to_string();
                Some((name, assignment))
            })
            .collect::<HashMap<_, _>>();
        let scope = supplied
            .get("scope")
            .map_or(IrEffectScope::Whole, |assignment| {
                let value = assignment
                    .children()
                    .find(|child| child.kind() == SyntaxKind::Expr)
                    .map(|expr| expr.text().to_string());
                match value.as_deref().map(str::trim) {
                    Some("\"background\"") => IrEffectScope::Background,
                    Some("\"border\"") => IrEffectScope::Border,
                    Some("\"content\"") => IrEffectScope::Content,
                    Some("\"text\"") => IrEffectScope::Text,
                    Some("\"backdrop\"") => IrEffectScope::Backdrop,
                    _ => IrEffectScope::Whole,
                }
            });
        let mut parameters = Vec::with_capacity(effect.parameters.len());
        for parameter in &effect.parameters {
            let (value, source) = if let Some(assignment) = supplied.get(&parameter.name) {
                let value = assignment
                    .children()
                    .find(|child| child.kind() == SyntaxKind::Expr)?;
                (
                    self.expression(&value, Some(site)),
                    self.source(assignment, Some(site)),
                )
            } else if let Some(default) = &parameter.default {
                (default.clone(), parameter.source.clone())
            } else {
                self.errors.push(LowerError::new(
                    format!("effect `{name}` requires parameter `{}`", parameter.name),
                    Span::new(self.module.file, node.text_range()),
                ));
                return None;
            };
            parameters.push(IrEffectArgument {
                parameter: parameter.id,
                value,
                source,
            });
        }
        Some(IrEffectBinding {
            effect: effect.id,
            scope,
            parameters,
            source: self.source(node, Some(site)),
        })
    }
}
