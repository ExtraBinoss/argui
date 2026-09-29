use super::*;

impl NodeTransition {
    /// Retains `spec` at its resolved targets without starting a transition.
    pub(super) fn new(spec: NodeSpec<'_>) -> Self {
        Self {
            target: spec.target,
            element: spec
                .element
                .map(|(element, scroll)| (element.clone(), scroll)),
            matched: spec.matched,
            revision: 0,
            values: spec
                .values
                .into_iter()
                .map(|resolved| AnimatedProperty {
                    key: resolved.property.key,
                    target: resolved.property.value.clone(),
                    source: resolved.source,
                    value: AnimatedValue::new(resolved.property.value),
                })
                .collect(),
        }
    }

    /// Applies `spec`, animating its changes unless `reduced_motion` requests immediate values.
    /// Returns the strongest update, including paint for an immediately settled transform.
    pub(super) fn sync(&mut self, spec: NodeSpec<'_>, reduced_motion: bool) -> TreeUpdate {
        self.element = spec
            .element
            .map(|(element, scroll)| (element.clone(), scroll));
        let mut update = TreeUpdate::None;
        self.values.retain(|property| {
            let retained = spec
                .values
                .iter()
                .any(|value| value.property.key == property.key);
            if !retained {
                update = strongest(update, property_update(&property.key, true));
            }
            retained
        });
        for resolved in spec.values {
            let target = resolved.property;
            let Some(property) = self.values.iter_mut().find(|value| value.key == target.key)
            else {
                let update_kind = property_update(&target.key, true);
                self.values.push(AnimatedProperty {
                    key: target.key,
                    target: target.value.clone(),
                    source: resolved.source,
                    value: AnimatedValue::new(target.value),
                });
                update = strongest(update, update_kind);
                continue;
            };
            if property.target == target.value {
                property.source = resolved.source;
                if (reduced_motion || spec.transition.is_none()) && property.value.finish() {
                    update = strongest(update, property_update(&property.key, true));
                }
                continue;
            }
            let direction = property_direction(
                &self.matched,
                &spec.matched,
                property.source.as_ref(),
                resolved.source.as_ref(),
            );
            property.target = target.value.clone();
            property.source = resolved.source;
            if reduced_motion || spec.transition.is_none() {
                property.value.set(target.value);
            } else if let Some(style_transition) = spec.transition {
                property.value.retarget(
                    target.value,
                    style_transition.resolve(&target.key, direction),
                );
            }
            update = strongest(
                update,
                property_update(&target.key, !property.value.is_active()),
            );
        }
        self.matched = spec.matched;
        if update != TreeUpdate::None {
            self.revision = self.revision.wrapping_add(1);
        }
        update
    }

    /// Samples properties at `now`, repainting each transform independently when it settles.
    /// Returns the strongest required invalidation and advances this node's revision.
    pub(super) fn advance(&mut self, now: Time) -> TreeUpdate {
        let update = self
            .values
            .iter()
            .fold(TreeUpdate::None, |update, property| {
                let was_active = property.value.is_active();
                if !was_active {
                    return update;
                }
                let changed = property.value.advance(now);
                let settled = was_active && !property.value.is_active();
                if changed || settled {
                    strongest(update, property_update(&property.key, settled))
                } else {
                    update
                }
            });
        if update != TreeUpdate::None {
            self.revision = self.revision.wrapping_add(1);
        }
        update
    }

    /// Finishes active properties and returns the invalidation for their final pixels.
    pub(super) fn finish(&mut self) -> TreeUpdate {
        let update = self
            .values
            .iter()
            .fold(TreeUpdate::None, |update, property| {
                if property.value.finish() {
                    strongest(update, property_update(&property.key, true))
                } else {
                    update
                }
            });
        if update != TreeUpdate::None {
            self.revision = self.revision.wrapping_add(1);
        }
        update
    }

    /// Returns whether a retained property still needs animation samples.
    pub(super) fn is_active(&self) -> bool {
        self.values
            .iter()
            .any(|property| property.value.is_active())
    }

    /// Returns the impact of removing this node's retained presentation values.
    pub(super) fn restoration_update(&self) -> TreeUpdate {
        self.values
            .iter()
            .fold(TreeUpdate::None, |update, property| {
                strongest(update, property_update(&property.key, true))
            })
    }
}

/// Returns `key`'s update, repainting settled transforms while preserving opacity composition.
fn property_update(key: &PropertyKey, settled: bool) -> TreeUpdate {
    if settled && *key == PropertyKey::Transform {
        TreeUpdate::Paint
    } else {
        key.impact().into()
    }
}
