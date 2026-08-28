use base64::{Engine as _, engine::general_purpose::STANDARD};
use winit::{platform::web::WindowExtWebSys, window::Window};

use crate::{ApplicationIdentity, WindowKey};

pub fn apply_web_identity(identity: &ApplicationIdentity) -> Result<(), String> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| "browser document is unavailable".to_owned())?;
    document.set_title(&identity.display_name);
    let existing = document
        .query_selector_all("link[data-argui-favicon]")
        .map_err(js_error)?;
    for index in 0..existing.length() {
        if let Some(node) = existing.item(index) {
            if let Some(parent) = node.parent_node() {
                parent.remove_child(&node).map_err(js_error)?;
            }
        }
    }
    let head = document
        .head()
        .ok_or_else(|| "browser document has no head".to_owned())?;
    for icon in identity.icons.icons() {
        let link = document.create_element("link").map_err(js_error)?;
        link.set_attribute("rel", "icon").map_err(js_error)?;
        link.set_attribute("type", "image/png").map_err(js_error)?;
        link.set_attribute("data-argui-favicon", "")
            .map_err(js_error)?;
        link.set_attribute("sizes", &format!("{}x{}", icon.width, icon.height))
            .map_err(js_error)?;
        let encoded = STANDARD.encode(&icon.png);
        link.set_attribute("href", &format!("data:image/png;base64,{encoded}"))
            .map_err(js_error)?;
        head.append_child(&link).map_err(js_error)?;
    }
    Ok(())
}

pub fn attach_web_canvas(
    window: &Window,
    key: &WindowKey,
    parent_id: Option<&str>,
) -> Result<(), String> {
    let canvas = window
        .canvas()
        .ok_or_else(|| "winit did not create a canvas".to_owned())?;
    canvas
        .set_attribute("data-argui-window", key.as_str())
        .map_err(js_error)?;
    if let Some(parent_id) = parent_id {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| "browser document is unavailable".to_owned())?;
        let parent = document
            .get_element_by_id(parent_id)
            .ok_or_else(|| format!("web parent element not found: {parent_id}"))?;
        parent.append_child(&canvas).map_err(js_error)?;
    }
    Ok(())
}

fn js_error(error: wasm_bindgen::JsValue) -> String {
    error.as_string().unwrap_or_else(|| format!("{error:?}"))
}
