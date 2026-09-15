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
    /// or propagates errors from scroll and portal layout.
    pub fn apply_scroll(
        &mut self,
        ui: &UiTree,
        output: &mut LayoutOutput,
    ) -> Result<(), LayoutError> {
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
        self.scroll_anchors = crate::anchor::capture(root, output);
        self.repaint(ui, output);
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
