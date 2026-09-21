//! Typed lowering for generic property-animation declarations.

use argui_dsl_syntax::{Span, SyntaxKind, SyntaxNode};

use crate::{
    AnimationId, IrAnimation, IrAnimationDriver, IrAnimationKeyframe, IrAnimationParameter,
    IrExpression, IrExpressionKind, IrTransitionPolicy, IrType, IrValue, LowerError, PropertyId,
    SiteId, declaration, expression, id::derive, id::hash_text,
};

use super::{Target, VisualLowerer, identifier_after};

impl VisualLowerer<'_> {
    /// Lowers each animation owned by one component or visual node.
    ///
    /// * `node` — syntax node containing animation declarations.
    /// * `owner` — optional visual site owning the target property.
    /// * `target` — resolved target members and types.
    pub(super) fn lower_animations(
        &mut self,
        node: &SyntaxNode,
        owner: Option<SiteId>,
        target: &Target,
    ) {
        for animation in node
            .children()
            .filter(|child| child.kind() == SyntaxKind::AnimateDecl)
        {
            let site = self.register_site(&animation);
            let Some(name) = identifier_after(&animation, SyntaxKind::AnimateKw) else {
                continue;
            };
            let Some((property, value_type)) = target.properties.get(&name) else {
                self.errors.push(LowerError::new(
                    format!("animation target property `{name}` is unavailable during IR lowering"),
                    Span::new(self.module.file, animation.text_range()),
                ));
                continue;
            };
            let parameters = animation
                .descendants()
                .filter(|child| child.kind() == SyntaxKind::PropertyAssignment)
                .filter_map(|assignment| {
                    let name = declaration::direct_name_or_theme(&assignment)?;
                    if name == "transition" {
                        return None;
                    }
                    let value = declaration::child_expression(&assignment)
                        .map(|value| self.animation_parameter_value(&name, &value, site))?;
                    Some(IrAnimationParameter {
                        id: PropertyId::from_raw(derive(
                            site.raw(),
                            "animation-parameter",
                            hash_text(&name),
                        )),
                        name,
                        value,
                        source: self.source(&assignment, Some(site)),
                    })
                })
                .collect::<Vec<_>>();
            let transition = animation
                .descendants()
                .filter(|child| child.kind() == SyntaxKind::PropertyAssignment)
                .find(|assignment| {
                    declaration::direct_name_or_theme(assignment).as_deref() == Some("transition")
                })
                .and_then(|assignment| declaration::child_expression(&assignment))
                .and_then(|value| match value.text().to_string().trim_matches('"') {
                    "enter" => Some(IrTransitionPolicy::Enter),
                    "leave" => Some(IrTransitionPolicy::Leave),
                    "in-out" => Some(IrTransitionPolicy::InOut),
                    _ => None,
                });
            let has_spring_block = animation.descendants().any(|child| {
                child.kind() == SyntaxKind::StyleStateDecl
                    && child
                        .children_with_tokens()
                        .filter_map(|item| item.into_token())
                        .any(|token| token.kind() == SyntaxKind::Ident && token.text() == "spring")
            });
            let driver = if has_spring_block
                || parameters
                    .iter()
                    .any(|parameter| matches!(parameter.name.as_str(), "stiffness" | "damping"))
            {
                IrAnimationDriver::Spring
            } else {
                IrAnimationDriver::Timeline
            };
            let keyframes = animation
                .descendants()
                .filter(|child| child.kind() == SyntaxKind::KeyframeDecl)
                .filter_map(|frame| self.lower_keyframe(&frame, site))
                .collect();
            self.animations.push(IrAnimation {
                id: AnimationId::from_raw(derive(site.raw(), "animation", 0)),
                owner,
                property: *property,
                value_type: value_type.clone(),
                driver,
                transition,
                parameters,
                keyframes,
                source: self.source(&animation, Some(site)),
            });
        }
    }

    /// Lowers an animation parameter, including contextual bare CSS easing names.
    ///
    /// * `name` — parameter identifier.
    /// * `value` — parsed parameter expression.
    /// * `site` — owning animation site for stable expression identity.
    ///
    /// Returns an expression with a resolved runtime type and source span.
    fn animation_parameter_value(
        &mut self,
        name: &str,
        value: &SyntaxNode,
        site: SiteId,
    ) -> IrExpression {
        let spelling = value.text().to_string();
        if name == "easing"
            && matches!(
                spelling.as_str(),
                "linear" | "ease" | "ease-in" | "ease-out" | "ease-in-out"
            )
        {
            let source = self.source(value, Some(site));
            return IrExpression {
                id: expression::expression_id(&source),
                value_type: IrType::String,
                kind: IrExpressionKind::Constant(IrValue::String(spelling)),
                source,
            };
        }
        self.expression(value, Some(site))
    }

    /// Lowers one validated percentage stop and its target-typed expression.
    ///
    /// * `frame` — parsed keyframe declaration.
    /// * `site` — stable animation site for expression identities.
    ///
    /// Returns the typed stop, or records an IR error for malformed recovery syntax.
    fn lower_keyframe(&mut self, frame: &SyntaxNode, site: SiteId) -> Option<IrAnimationKeyframe> {
        let offset = frame
            .children_with_tokens()
            .filter_map(|item| item.into_token())
            .find(|token| token.kind() == SyntaxKind::Number)
            .and_then(|token| {
                token
                    .text()
                    .strip_suffix('%')
                    .and_then(|value| value.replace('_', "").parse::<f32>().ok())
            });
        let Some(offset) = offset else {
            self.errors.push(LowerError::new(
                "keyframe offset must be a percentage",
                Span::new(self.module.file, frame.text_range()),
            ));
            return None;
        };
        let Some(value) = frame
            .children()
            .find(|node| node.kind() == SyntaxKind::Expr)
        else {
            self.errors.push(LowerError::new(
                "keyframe has no value expression",
                Span::new(self.module.file, frame.text_range()),
            ));
            return None;
        };
        Some(IrAnimationKeyframe {
            offset: offset / 100.0,
            value: self.expression(&value, Some(site)),
            source: self.source(frame, Some(site)),
        })
    }
}
