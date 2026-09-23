//! Named styles normalize into the same bindings and states used by both backends.
use super::*;
use crate::{ExpressionId, IrStyle, StyleId};

impl VisualLowerer<'_> {
    /// Adds the named style on `node` to `bindings` at `site`, then appends its
    /// interaction states. Inline bindings override ordinary style properties;
    /// authored states are lowered afterwards and override style states.
    pub(super) fn apply_styles(
        &mut self,
        node: &SyntaxNode,
        site: SiteId,
        bindings: &mut Vec<IrPropertyBinding>,
    ) {
        let Some(style) = node
            .children()
            .find(|child| child.kind() == SyntaxKind::StyleApplication)
            .and_then(|node| direct_identifier(&node))
            .and_then(|name| self.module.scope.get(&name))
            .and_then(|id| {
                self.styles
                    .iter()
                    .find(|style| style.id == StyleId::from_raw(id.raw()))
            })
            .cloned()
        else {
            return;
        };
        for binding in &style.properties {
            if !bindings
                .iter()
                .any(|inline| inline.target == binding.target)
            {
                bindings.push(binding.clone());
            }
        }
        self.style_states(&style, site);
    }

    /// Appends interaction states from resolved `style`, rebasing identities to
    /// `site`. Returns no values; shared style expressions retain token dependencies.
    fn style_states(&mut self, style: &IrStyle, site: SiteId) {
        for state in &style.states {
            let state_site = SiteId::from_raw(derive(site.raw(), "style-state", state.id.raw()));
            self.states.push(IrState {
                site: state_site,
                owner: Some(site),
                condition: IrExpression {
                    id: ExpressionId::from_raw(derive(state_site.raw(), "condition", 0)),
                    value_type: IrType::Bool,
                    kind: IrExpressionKind::ObservedRead {
                        site,
                        observation: state.observation,
                        property: match state.observation {
                            crate::IrObservation::Hover => argui_schema::builtin::HAS_HOVER,
                            crate::IrObservation::Pressed => argui_schema::builtin::PRESSED,
                            crate::IrObservation::Focused => argui_schema::builtin::HAS_FOCUS,
                            _ => argui_schema::builtin::FOCUS_VISIBLE,
                        },
                    },
                    source: SourceInfo {
                        component: Some(self.component),
                        site: Some(site),
                        ..state.source.clone()
                    },
                },
                assignments: state
                    .properties
                    .iter()
                    .map(|binding| (binding.target, binding.value.clone()))
                    .collect(),
                source: state.source.clone(),
            });
        }
    }
}
