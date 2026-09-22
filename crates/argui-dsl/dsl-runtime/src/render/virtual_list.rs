//! Live rendering of a keyed repeater through a bounded virtual window.

use std::collections::HashMap;

use argui_dsl_ir::{
    IrExpressionKind, IrNode, IrPropertyBinding, LocalId, PropertyTargetId, SiteId, SlotId,
};

use super::{TemplateSlot, identity::retained};
use crate::{ComponentInstance, DslValue, InstanceId, LiveRuntime, RuntimeError};

impl LiveRuntime {
    /// Renders only mounted VirtualWindow rows, preserving each repeater key and component instance.
    ///
    /// `instance` is the owning component and `component` its IR. `site`,
    /// `identity_owner`, and `repeater_key` identify the measured viewport;
    /// `children` is its one keyed
    /// repeater or template-slot reference, `properties` configure the window,
    /// `locals` and `slots` provide the surrounding scope, `template` borrows
    /// a caller-authored lazy row recipe when present. `identity_owner` isolates
    /// retained row keys by list instance, and `context` registers row events. Returns the
    /// mounted rows, collection length, first mounted index, and viewport height.
    ///
    /// # Errors
    ///
    /// Returns for malformed bytecode, invalid dimensions, or row rendering errors.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_virtual_children(
        &mut self,
        instance: &mut ComponentInstance,
        component: &argui_dsl_ir::IrComponent,
        site: SiteId,
        children: &[IrNode],
        properties: &[IrPropertyBinding],
        locals: &HashMap<LocalId, DslValue>,
        slots: &HashMap<SlotId, Vec<argui_ui::Element>>,
        template: Option<&mut TemplateSlot<'_>>,
        identity_owner: InstanceId,
        repeater_key: Option<&DslValue>,
        context: &mut Option<&mut argui_runtime::Context<Self>>,
    ) -> Result<(Vec<argui_ui::Element>, usize, usize, f32), RuntimeError> {
        let row_height = self.virtual_float(
            instance,
            properties,
            argui_schema::builtin::ROW_HEIGHT,
            locals,
            None,
        )?;
        let viewport_identity = retained(identity_owner, site, repeater_key)?;
        let explicit_viewport = self.virtual_float(
            instance,
            properties,
            argui_schema::builtin::VIEWPORT_HEIGHT,
            locals,
            Some(0.0),
        )?;
        let viewport = if explicit_viewport > 0.0 {
            explicit_viewport
        } else {
            self.virtual_viewports
                .get(&viewport_identity)
                .copied()
                .unwrap_or(0.0)
        };
        let offset = self.virtual_float(
            instance,
            properties,
            argui_schema::builtin::SCROLL_OFFSET,
            locals,
            Some(0.0),
        )?;
        let overscan = self.virtual_int(
            instance,
            properties,
            argui_schema::builtin::OVERSCAN,
            locals,
            3,
        )?;
        if !row_height.is_finite()
            || row_height <= 0.0
            || !viewport.is_finite()
            || viewport < 0.0
            || !offset.is_finite()
        {
            return Err(RuntimeError::Schema("VirtualWindow requires finite positive row_height, nonnegative viewport_height, and finite offset".into()));
        }
        let (repeater, row_instance, row_component, row_locals, row_slots) = match children {
            [repeater @ IrNode::Repeater { .. }] => (repeater, instance, component, locals, slots),
            [IrNode::Slot { slot, .. }] => {
                let template = template.ok_or_else(|| {
                    RuntimeError::InvalidBytecode(
                        "VirtualWindow template slot is outside its caller".into(),
                    )
                })?;
                if template.slot != *slot {
                    return Err(RuntimeError::InvalidBytecode(
                        "VirtualWindow template slot identity does not match its caller".into(),
                    ));
                }
                (
                    template.repeater,
                    &mut *template.owner,
                    template.component,
                    template.locals,
                    template.slots,
                )
            }
            _ => {
                return Err(RuntimeError::InvalidBytecode(
                    "VirtualWindow requires exactly one keyed repeater".into(),
                ));
            }
        };
        let IrNode::Repeater {
            local,
            model,
            key,
            body,
            ..
        } = repeater
        else {
            return Err(RuntimeError::InvalidBytecode(
                "VirtualWindow template requires one keyed repeater".into(),
            ));
        };
        let mounted = |items: &[DslValue]| {
            let window = argui_ui::VirtualList::fixed(items.len(), row_height, viewport)
                .overscan(overscan)
                .window(offset);
            let first = window.range.start;
            let selected = window
                .range
                .map(|index| items[index].clone())
                .collect::<Vec<_>>();
            (items.len(), first, selected)
        };
        let (count, start, selected) = if let IrExpressionKind::PropertyRead(property) = &model.kind
        {
            let value = row_instance
                .properties
                .get(property)
                .ok_or(RuntimeError::MissingProperty(property.raw()))?
                .get();
            let DslValue::Array(items) = value else {
                return Err(RuntimeError::TypeMismatch {
                    expected: "model/array".into(),
                    actual: value.type_name().into(),
                });
            };
            mounted(items)
        } else {
            // Computed model expressions currently evaluate into an owned array;
            // direct model properties above clone only the mounted window.
            let value = self.evaluate(row_instance, model, row_locals)?;
            let DslValue::Array(items) = value else {
                return Err(RuntimeError::TypeMismatch {
                    expected: "model/array".into(),
                    actual: value.type_name().into(),
                });
            };
            mounted(&items)
        };
        let mut rows = Vec::with_capacity(selected.len());
        for item in selected {
            let mut nested = row_locals.clone();
            nested.insert(*local, item);
            let row_key = self.evaluate(row_instance, key, &nested)?;
            let rendered = self.render_nodes(
                row_instance,
                row_component,
                body,
                &nested,
                row_slots,
                None,
                identity_owner,
                Some(&row_key),
                context,
            )?;
            rows.push(if rendered.len() == 1 {
                rendered
                    .into_iter()
                    .next()
                    .expect("single row already checked")
            } else {
                argui_ui::Element::column(rendered)
            });
        }
        Ok((rows, count, start, viewport))
    }

    /// Evaluates one VirtualWindow float property, with a fallback for optional values.
    ///
    /// `instance` owns the binding, `properties` are native assignments,
    /// `id` selects one, `locals` resolves its expression, and `fallback`
    /// supplies an optional default. Returns logical pixels or an offset.
    ///
    /// # Errors
    ///
    /// Returns for a missing required property or incompatible runtime value.
    fn virtual_float(
        &mut self,
        instance: &mut ComponentInstance,
        properties: &[IrPropertyBinding],
        id: argui_schema::PropertyId,
        locals: &HashMap<LocalId, DslValue>,
        fallback: Option<f32>,
    ) -> Result<f32, RuntimeError> {
        let binding = properties
            .iter()
            .find(|binding| binding.target == PropertyTargetId::Native(id));
        let Some(binding) = binding else {
            return fallback.ok_or_else(|| {
                RuntimeError::Schema(format!("VirtualWindow requires property {}", id.raw()))
            });
        };
        let value = self.evaluate(instance, &binding.value, locals)?;
        match value {
            DslValue::Float(value) => Ok(value as f32),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "float".into(),
                actual: value.type_name().into(),
            }),
        }
    }

    /// Evaluates a nonnegative VirtualWindow integer property, using `fallback` when absent.
    ///
    /// `instance` owns the binding, `properties` are assignments, `id` selects
    /// one, and `locals` resolves its expression. Returns the row count.
    ///
    /// # Errors
    ///
    /// Returns for a negative or non-integer runtime value.
    fn virtual_int(
        &mut self,
        instance: &mut ComponentInstance,
        properties: &[IrPropertyBinding],
        id: argui_schema::PropertyId,
        locals: &HashMap<LocalId, DslValue>,
        fallback: usize,
    ) -> Result<usize, RuntimeError> {
        let Some(binding) = properties
            .iter()
            .find(|binding| binding.target == PropertyTargetId::Native(id))
        else {
            return Ok(fallback);
        };
        let value = self.evaluate(instance, &binding.value, locals)?;
        match value {
            DslValue::Int(value) => usize::try_from(value).map_err(|_| {
                RuntimeError::Schema("VirtualWindow overscan must be nonnegative".into())
            }),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "int".into(),
                actual: value.type_name().into(),
            }),
        }
    }
}
