#[cfg(target_arch = "wasm32")]
use argui_platform::Clipboard;
use argui_ui::ClipboardRequest;
use winit::{event_loop::ActiveEventLoop, window::Window};

use crate::app::Application;
#[cfg(target_arch = "wasm32")]
use crate::event::UserEvent;

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn clipboard_request(
        &mut self,
        request: ClipboardRequest,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        match request {
            ClipboardRequest::Read { target } => {
                if let Ok(text) = self.clipboard.read_text()
                    && let Some(ui) = &mut self.ui_tree
                {
                    let update = ui.paste_text(target, &text);
                    self.apply_ui_update(update, window, event_loop);
                }
            }
            ClipboardRequest::Write(text) => {
                let _ = self.clipboard.write_text(text);
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub(super) fn clipboard_request(
        &mut self,
        request: ClipboardRequest,
        _window: &Window,
        _event_loop: &ActiveEventLoop,
    ) {
        match request {
            ClipboardRequest::Read { target } => {
                let Some(proxy) = self.event_proxy.clone() else {
                    return;
                };
                let window = self.window_key.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let mut clipboard = Clipboard::new();
                    if let Ok(text) = clipboard.read_text().await {
                        let _ = proxy.send_event(UserEvent::ClipboardText {
                            window,
                            target,
                            text,
                        });
                    }
                });
            }
            ClipboardRequest::Write(text) => {
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = Clipboard::new().write_text(text).await;
                });
            }
        }
    }
}
