use argui_ui::Element;

use crate::WebViewState;

/// A layout slot; its retained session is registered with the native host separately.
pub struct WebView<'a> {
    state: &'a WebViewState,
}

impl<'a> WebView<'a> {
    #[must_use]
    pub const fn new(state: &'a WebViewState) -> Self {
        Self { state }
    }

    #[must_use]
    pub fn build(self) -> Element {
        Element::container([])
            .native_content(argui_ui::NativeContent::new(
                self.state.id().0,
                self.state.clone(),
            ))
            .keyed(format!("argui-webview-{}", self.state.id().0))
            .min_width(argui_ui::length(0.0))
            .min_height(argui_ui::length(0.0))
            .user_select(argui_ui::UserSelect::None)
    }
}
