use super::{DocumentSelectionDrag, DocumentSelectionEndpoint, SelectionGranularity};
use crate::{InteractionUpdate, UiTree};

impl UiTree {
    /// Begins dragging one visible touch-selection endpoint.
    ///
    /// * `endpoint` — anchor or focus endpoint whose position will follow later drag updates.
    ///
    /// Returns the interaction changes caused by beginning the drag. Returns an empty update when
    /// touch handles are not visible or the selection is empty.
    pub fn begin_document_selection_handle_drag(
        &mut self,
        endpoint: DocumentSelectionEndpoint,
    ) -> InteractionUpdate {
        if !self.document_selection.touch_handles || !self.has_document_selection() {
            return InteractionUpdate::default();
        }
        let Some(selection) = self.document_selection.selection else {
            return InteractionUpdate::default();
        };
        let drag = DocumentSelectionDrag {
            granularity: SelectionGranularity::Character,
            origin: selection,
            endpoint: Some(endpoint),
        };
        if self.document_selection.drag == Some(drag) {
            return InteractionUpdate::default();
        }
        self.document_selection.drag = Some(drag);
        self.document_selection_update(false)
    }
}
