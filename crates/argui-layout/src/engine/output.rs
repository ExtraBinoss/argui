use argui_text::TextEngine;
use argui_ui::UiTree;

use crate::{LayoutError, input, paint};

use super::{LayoutEngine, LayoutOutput, ScrollPlacement, apply_scroll_layout, flattened};

impl LayoutEngine {
    /// Recomputes scroll placement, portal geometry, and paint for retained layout.
    ///
    /// * `ui` — current retained UI tree.
    /// * `output` — layout output whose scroll geometry is updated.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutError::MissingRoot`] if layout has not been computed yet,
    /// [`LayoutError::StaleOutput`] if `ui` changed since the output was built,
    /// or propagates errors from scroll and portal layout.
    pub fn apply_scroll(
        &mut self,
        ui: &UiTree,
        output: &mut LayoutOutput,
    ) -> Result<(), LayoutError> {
        self.apply_scroll_geometry(ui, output)?;
        self.repaint(ui, output);
        Ok(())
    }

    /// Recomputes scroll placement and refreshes virtualized editor text when required.
    ///
    /// * `ui` — current retained UI tree whose scroll offsets are authoritative.
    /// * `text_engine` — shaping engine used to prepare newly visible editor lines.
    /// * `output` — retained layout output updated in place.
    ///
    /// Returns whether prepared glyph data must be rebuilt. A stale UI tree
    /// triggers a full layout and returns true; otherwise only a newly visible
    /// editor text window does so.
    ///
    /// # Errors
    ///
    /// Returns [`LayoutError::MissingRoot`] if layout has not been computed yet,
    /// or propagates errors from layout, scroll, and portal placement.
    pub fn apply_scroll_with_text(
        &mut self,
        ui: &mut UiTree,
        text_engine: &mut TextEngine,
        output: &mut LayoutOutput,
    ) -> Result<bool, LayoutError> {
        if self.revision != Some(ui.revision()) || output.nodes.len() != ui.node_ids().len() {
            *output = self.compute(ui, text_engine, output.viewport.size)?;
            return Ok(true);
        }
        let refresh_text = needs_scroll_refresh(ui, output);
        self.apply_scroll_geometry(ui, output)?;
        if refresh_text {
            input::update(ui, text_engine, output);
        }
        self.repaint(ui, output);
        Ok(refresh_text)
    }

    /// Updates retained scroll and overlay geometry without repainting the output.
    /// Returns an error when the output belongs to an older UI revision.
    fn apply_scroll_geometry(
        &mut self,
        ui: &UiTree,
        output: &mut LayoutOutput,
    ) -> Result<(), LayoutError> {
        if self.revision != Some(ui.revision()) || output.nodes.len() != ui.node_ids().len() {
            return Err(LayoutError::StaleOutput);
        }
        let elements = flattened(ui.root());
        let root = self.root.as_ref().ok_or(LayoutError::MissingRoot)?;
        output.scroll_regions.clear();
        apply_scroll_layout(
            &self.tree,
            root,
            &elements,
            ui,
            ScrollPlacement {
                translation: argui_core::Point::default(),
                clip: Some(output.viewport),
                sticky_container: None,
            },
            output,
        )?;
        crate::overlay::resolve(&self.tree, root, &elements, ui, output)?;
        self.scroll_anchors = crate::anchor::capture(root, output, ui);
        Ok(())
    }

    /// Repaints retained primitives and reports whether prepared text needs refreshing.
    ///
    /// * `ui` — current retained UI tree.
    /// * `output` — layout output whose paint data is updated.
    pub fn repaint(&mut self, ui: &UiTree, output: &mut LayoutOutput) -> bool {
        paint::repaint(self.root.as_ref(), ui, output, &mut self.paint_cache)
    }

    /// Updates text-input geometry and repaints the resulting caret and selection state.
    ///
    /// * `ui` — retained tree whose text-input state is updated.
    /// * `text_engine` — shaping engine used to compute input caret geometry.
    /// * `output` — layout output containing the input regions to update.
    pub fn update_text_inputs(
        &mut self,
        ui: &mut UiTree,
        text_engine: &mut TextEngine,
        output: &mut LayoutOutput,
    ) {
        input::update(ui, text_engine, output);
        self.repaint(ui, output);
    }
}

/// Reports whether a vertical scroll exposed a new virtualized editor text window.
fn needs_scroll_refresh(ui: &UiTree, output: &LayoutOutput) -> bool {
    output.text_inputs.iter().any(|region| {
        let offset = ui.scroll_offset(region.node);
        if (offset.y - region.scroll_y).abs() <= f32::EPSILON
            || region.content_size.height <= region.viewport.size.height
        {
            return false;
        }
        let virtualized = output
            .nodes
            .iter()
            .find(|node| node.node == region.node)
            .and_then(|node| node.text_index)
            .is_some_and(|index| {
                output.text.blocks()[index].style.wrap == argui_text::TextWrap::None
            });
        if !virtualized {
            return false;
        }
        let Some(text_window) = output.input_windows.get(&region.node) else {
            return true;
        };
        let visible_end = offset.y + region.viewport.size.height;
        offset.y + f32::EPSILON < text_window.start || visible_end > text_window.end + f32::EPSILON
    })
}
