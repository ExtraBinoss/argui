use super::Computation;
use argui_core::{Point, Rect, Size};
use argui_ui::{
    CustomConstraints, CustomDescription, CustomLayoutContext, CustomMeasurement, CustomState,
};
use taffy::{
    AvailableSpace, NodeId, Overflow,
    geometry::{Line, Size as TSize},
    tree::{Baselines, Layout, LayoutInput, LayoutOutput, RequestedAxis, RunMode, SizingMode},
    util::ResolveOrZero,
};

struct Children<'a, 'b> {
    computation: &'a mut Computation<'b>,
    parent: NodeId,
    inputs: LayoutInput,
    available: CustomConstraints,
    known: CustomConstraints,
    origin: Point,
    placed: Vec<bool>,
    extent: taffy::geometry::Rect<f32>,
}

fn space(value: Option<f32>) -> AvailableSpace {
    value.map_or(AvailableSpace::MaxContent, AvailableSpace::Definite)
}
fn valid(constraints: CustomConstraints) -> Result<(), String> {
    if [constraints.width, constraints.height]
        .into_iter()
        .flatten()
        .any(|value| !value.is_finite() || value < 0.0)
    {
        Err("child constraints must be finite non-negative lengths or unbounded".into())
    } else {
        Ok(())
    }
}

impl Computation<'_> {
    pub(super) fn custom_layout(
        &mut self,
        id: NodeId,
        inputs: LayoutInput,
        description: &CustomDescription,
        state: &CustomState,
    ) -> Result<LayoutOutput, String> {
        let style = self.tree.nodes[&id].style.clone();
        let padding = style
            .padding
            .resolve_or_zero(inputs.parent_size.width, |_, _| 0.0);
        let border = style
            .border
            .resolve_or_zero(inputs.parent_size.width, |_, _| 0.0);
        let mut inset = padding + border;
        if style.overflow.y == Overflow::Scroll {
            inset.right += style.scrollbar_width;
        }
        if style.overflow.x == Overflow::Scroll {
            inset.bottom += style.scrollbar_width;
        }
        let subtract =
            |value: Option<f32>, amount: f32| value.map(|value| (value - amount).max(0.0));
        let available = CustomConstraints {
            width: subtract(
                inputs
                    .known_dimensions
                    .width
                    .or_else(|| inputs.available_space.width.into_option()),
                inset.left + inset.right,
            ),
            height: subtract(
                inputs
                    .known_dimensions
                    .height
                    .or_else(|| inputs.available_space.height.into_option()),
                inset.top + inset.bottom,
            ),
        };
        let known = CustomConstraints {
            width: subtract(inputs.known_dimensions.width, inset.left + inset.right),
            height: subtract(inputs.known_dimensions.height, inset.top + inset.bottom),
        };
        let count = self.tree.nodes[&id].children.len();
        let mut context = Children {
            computation: self,
            parent: id,
            inputs,
            available,
            known,
            origin: Point::new(inset.left, inset.top),
            placed: vec![false; count],
            extent: taffy::geometry::Rect::ZERO,
        };
        let mut measured = context.invoke(description, state)?;
        let mut output = taffy::compute_leaf_layout(
            inputs,
            &style,
            |_, _| 0.0,
            |known, _| TSize {
                width: known.width.unwrap_or(measured.size.width),
                height: known.height.unwrap_or(measured.size.height),
            },
        );
        let final_size = CustomConstraints {
            width: Some((output.size.width - inset.left - inset.right).max(0.0)),
            height: Some((output.size.height - inset.top - inset.bottom).max(0.0)),
        };
        if count > 0 && context.known != final_size && inputs.run_mode == RunMode::PerformLayout {
            context.known = final_size;
            context.available = final_size;
            context.placed.fill(false);
            context.extent = taffy::geometry::Rect::ZERO;
            measured = context.invoke(description, state)?;
        }
        output.baselines = Baselines {
            first: measured.baseline.map(|value| value + inset.top),
            last: measured.baseline.map(|value| value + inset.top),
        };
        if count > 0 {
            output.scrollable_overflow_rect = context.extent;
            if style.overflow.x.is_scroll_container() || style.overflow.y.is_scroll_container() {
                output.scrollable_overflow_rect.right += padding.right;
                output.scrollable_overflow_rect.bottom += padding.bottom;
            }
        }
        Ok(output)
    }
}

impl Children<'_, '_> {
    fn invoke(
        &mut self,
        description: &CustomDescription,
        state: &CustomState,
    ) -> Result<CustomMeasurement, String> {
        let measured = description
            .layout(state, self)
            .map_err(|error| format!("{}: {error}", description.type_name()))?;
        let measured = crate::custom::validate_measure(description, measured)?;
        if let Some(index) = self.placed.iter().position(|placed| !placed) {
            return Err(format!(
                "{} did not place child {index}",
                description.type_name()
            ));
        }
        Ok(measured)
    }
    fn child(&self, index: usize) -> Result<NodeId, String> {
        self.computation.tree.nodes[&self.parent]
            .children
            .get(index)
            .copied()
            .ok_or_else(|| format!("child index {index} is out of bounds"))
    }
    fn child_inputs(
        &self,
        available: CustomConstraints,
        known: TSize<Option<f32>>,
        run_mode: RunMode,
    ) -> LayoutInput {
        LayoutInput {
            run_mode,
            sizing_mode: SizingMode::InherentSize,
            axis: RequestedAxis::Both,
            known_dimensions: known,
            known_dimensions_are_definite: TSize {
                width: true,
                height: true,
            },
            parent_size: TSize {
                width: self.known.width.or(self.available.width),
                height: self.known.height.or(self.available.height),
            },
            available_space: TSize {
                width: space(available.width),
                height: space(available.height),
            },
            vertical_margins_are_collapsible: Line::FALSE,
        }
    }
}

impl CustomLayoutContext for Children<'_, '_> {
    fn available(&self) -> CustomConstraints {
        self.available
    }
    fn known_size(&self) -> CustomConstraints {
        self.known
    }
    fn child_count(&self) -> usize {
        self.placed.len()
    }
    fn measure_child(
        &mut self,
        index: usize,
        available: CustomConstraints,
    ) -> Result<CustomMeasurement, String> {
        valid(available)?;
        let id = self.child(index)?;
        let inputs = self.child_inputs(available, TSize::NONE, RunMode::ComputeSize);
        let output = self.computation.compute(id, inputs, None);
        if let Some(error) = &self.computation.error {
            return Err(error.clone());
        }
        Ok(CustomMeasurement {
            size: Size::new(output.size.width, output.size.height),
            baseline: output.baselines.first,
        })
    }
    fn place_child(&mut self, index: usize, bounds: Rect) -> Result<(), String> {
        valid(CustomConstraints {
            width: Some(bounds.size.width),
            height: Some(bounds.size.height),
        })?;
        if !bounds.origin.x.is_finite() || !bounds.origin.y.is_finite() {
            return Err("child origin must be finite".into());
        }
        let id = self.child(index)?;
        if self.placed[index] {
            return Err(format!("child {index} was placed twice"));
        }
        let available = CustomConstraints {
            width: Some(bounds.size.width),
            height: Some(bounds.size.height),
        };
        let inputs = self.child_inputs(
            available,
            TSize {
                width: available.width,
                height: available.height,
            },
            self.inputs.run_mode,
        );
        let output = self.computation.compute(id, inputs, None);
        if let Some(error) = &self.computation.error {
            return Err(error.clone());
        }
        if self.inputs.run_mode == RunMode::PerformLayout {
            let node = self
                .computation
                .tree
                .nodes
                .get_mut(&id)
                .expect("live layout child");
            let style = &node.style;
            node.unrounded = Layout {
                order: index as u32,
                location: taffy::geometry::Point {
                    x: self.origin.x + bounds.origin.x,
                    y: self.origin.y + bounds.origin.y,
                },
                size: output.size,
                scrollable_overflow_rect: output.scrollable_overflow_rect,
                padding: style
                    .padding
                    .resolve_or_zero(inputs.parent_size.width, |_, _| 0.0),
                border: style
                    .border
                    .resolve_or_zero(inputs.parent_size.width, |_, _| 0.0),
                margin: style
                    .margin
                    .resolve_or_zero(inputs.parent_size.width, |_, _| 0.0),
                scrollbar_size: TSize {
                    width: if style.overflow.y == Overflow::Scroll {
                        style.scrollbar_width
                    } else {
                        0.0
                    },
                    height: if style.overflow.x == Overflow::Scroll {
                        style.scrollbar_width
                    } else {
                        0.0
                    },
                },
            };
        }
        let child_style = &self.computation.tree.nodes[&id].style;
        let clipped = child_style.overflow.x.is_scroll_container()
            || child_style.overflow.y.is_scroll_container()
            || child_style.contain.contains_scrollable_overflow();
        let visible_x = !clipped && child_style.overflow.x == Overflow::Visible;
        let visible_y = !clipped && child_style.overflow.y == Overflow::Visible;
        let overflow = output.scrollable_overflow_rect;
        let x = self.origin.x + bounds.origin.x;
        let y = self.origin.y + bounds.origin.y;
        self.extent.left = self.extent.left.min(
            x + if visible_x {
                overflow.left.min(0.0)
            } else {
                0.0
            },
        );
        self.extent.top = self.extent.top.min(
            y + if visible_y {
                overflow.top.min(0.0)
            } else {
                0.0
            },
        );
        self.extent.right = self.extent.right.max(
            x + if visible_x {
                output.size.width.max(overflow.right)
            } else {
                output.size.width
            },
        );
        self.extent.bottom = self.extent.bottom.max(
            y + if visible_y {
                output.size.height.max(overflow.bottom)
            } else {
                output.size.height
            },
        );
        self.placed[index] = true;
        Ok(())
    }
}
