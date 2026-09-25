use argui_ui::Element;

use super::Application;

impl Application {
    /// Renders the current model view and applies platform and inspection wrappers.
    ///
    /// Returns the rendered root when this application owns a model.
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(crate) fn inspected_view(&self) -> Option<Element> {
        let root = self.model.as_ref().map(|model| {
            model.render(self.environment.clone(), self.interaction_snapshot.clone())
        })?;
        #[cfg(any(feature = "inspect", all(feature = "webview", target_os = "linux")))]
        let mut root = root;
        #[cfg(all(feature = "webview", target_os = "linux"))]
        if let Some(host) = self.window.as_ref().and_then(|window| window.gtk()) {
            let radius = host.platform.corner_radius();
            root = Element::container([root])
                .keyed("argui-native-client")
                .width(argui_ui::percent(1.0))
                .height(argui_ui::percent(1.0))
                .radius(argui_paint::CornerRadii {
                    top_left: 0.0,
                    top_right: 0.0,
                    bottom_left: radius,
                    bottom_right: radius,
                })
                .overflow(argui_ui::Axes {
                    x: argui_ui::Overflow::Hidden,
                    y: argui_ui::Overflow::Hidden,
                });
        }
        #[cfg(feature = "inspect")]
        if let (Some(inspector), Some(tree)) = (&self.inspector, self.ui_tree.as_ref()) {
            super::Inspection::apply_overrides(&mut root, tree, inspector);
        }
        Some(root)
    }
}
