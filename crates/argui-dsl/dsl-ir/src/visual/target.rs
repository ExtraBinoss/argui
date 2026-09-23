//! Resolved element properties and callback signatures.
use super::*;
impl VisualLowerer<'_> {
    /// Resolves one element target and all assignable members.
    pub(super) fn target(&mut self, name: &str, node: &SyntaxNode) -> Option<Target> {
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
    pub(super) fn component_target(&self) -> Target {
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
}
