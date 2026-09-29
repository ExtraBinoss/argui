//! Intrinsic content measurement for ordinary roots and detached popup surfaces.

use crate::{
    LayoutError,
    assets::{AssetMetrics, resolve_intrinsic},
    layout_tree::LayoutTree,
};
use argui_core::Size;
use argui_text::TextEngine;
use argui_ui::{ElementKind, UiTree};
use taffy::{
    AvailableSpace, NodeId, compute_leaf_layout, geometry::Size as TaffySize,
    tree::LayoutOutput as TaffyLayoutOutput,
};

pub(super) struct Measurement<'a> {
    pub(super) assets: &'a AssetMetrics,
    pub(super) elements: &'a [&'a argui_ui::Element],
    pub(super) ui: &'a UiTree,
    pub(super) text_engine: &'a mut TextEngine,
}

impl Measurement<'_> {
    /// Measures `root` against `available`, preserving an optional parent boundary input.
    /// Returns any native layout or text measurement error.
    pub(super) fn compute(
        &mut self,
        tree: &mut LayoutTree,
        root: NodeId,
        available: Size,
        boundary_input: Option<taffy::tree::LayoutInput>,
    ) -> Result<(), LayoutError> {
        self.compute_with_space(
            tree,
            root,
            AvailableSpace::Definite(available.width),
            available.height,
            boundary_input,
        )
    }

    /// Measures an independent portal intrinsically when its width is automatic.
    /// `tree` owns geometry, `root` identifies the detached surface, and `available`
    /// bounds its eventual placement. Explicit and percentage widths retain their sizing.
    /// Returns any layout or measurement error.
    pub(super) fn compute_portal(
        &mut self,
        tree: &mut LayoutTree,
        root: NodeId,
        available: Size,
    ) -> Result<(), LayoutError> {
        let width = if tree.style(root)?.size.width.is_auto() {
            AvailableSpace::MaxContent
        } else {
            AvailableSpace::Definite(available.width)
        };
        self.compute_with_space(tree, root, width, available.height, None)
    }

    /// Shapes the leaf content while measuring `root` in the supplied available space.
    /// `boundary_input` preserves the parent constraint for a retained layout boundary.
    /// Returns any layout or text measurement error.
    fn compute_with_space(
        &mut self,
        tree: &mut LayoutTree,
        root: NodeId,
        width: AvailableSpace,
        height: f32,
        boundary_input: Option<taffy::tree::LayoutInput>,
    ) -> Result<(), LayoutError> {
        tree.compute_layout_with_measure(
            root,
            TaffySize {
                width,
                height: AvailableSpace::Definite(height),
            },
            boundary_input,
            |inputs, _, context, style| {
                let index = context;
                let intrinsic =
                    index.and_then(|index| self.assets.intrinsic(&self.elements[index].kind));
                let text = index.and_then(|index| {
                    let node = self.ui.node_id_at(index)?;
                    let (content, text_style) =
                        crate::text::content(self.ui, node, self.elements[index])?;
                    let width = inputs
                        .known_dimensions
                        .width
                        .or_else(|| inputs.available_space.width.into_option());
                    Some(
                        if matches!(self.elements[index].kind, ElementKind::TextEditor { .. }) {
                            self.text_engine
                                .measure_editor_content(&content, &text_style, width)
                        } else {
                            self.text_engine
                                .measure_content(&content, &text_style, width)
                        },
                    )
                });
                if index.is_some_and(|index| {
                    matches!(
                        self.elements[index].kind,
                        ElementKind::Text { .. } | ElementKind::TextEditor { .. }
                    )
                }) {
                    debug_assert!(
                        text.is_some_and(|measurement| measurement.first_baseline.is_some())
                    );
                }
                let baselines = text.map_or(taffy::tree::Baselines::NONE, |measurement| {
                    taffy::tree::Baselines {
                        first: measurement.first_baseline,
                        last: measurement.last_baseline,
                    }
                });
                let size = compute_leaf_layout(
                    inputs,
                    style,
                    |_, _| 0.0,
                    |known, _| {
                        if let Some(intrinsic) = intrinsic {
                            return resolve_intrinsic(known, intrinsic);
                        }
                        let Some(measured) = text else {
                            return TaffySize::ZERO;
                        };
                        TaffySize {
                            width: known.width.unwrap_or(measured.size.width),
                            height: known.height.unwrap_or(measured.size.height),
                        }
                    },
                );
                TaffyLayoutOutput { baselines, ..size }
            },
        )?;
        Ok(())
    }
}
