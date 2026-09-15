use base64::{Engine as _, engine::general_purpose::STANDARD};
use winit::{dpi::PhysicalSize, platform::web::WindowExtWebSys, window::Window};

use crate::{ApplicationIdentity, WindowKey};

const CANVAS_STYLE: &str = r#"
canvas[data-argui-window] {
  border: 0;
  outline: none;
}
html.argui-show-canvas-focus-ring canvas[data-argui-window]:focus-visible {
  outline: 2px solid Highlight;
  outline-offset: -2px;
}
"#;

/// Applies application identity metadata to the current browser document.
/// `identity` supplies the document title, application identifier, and icons.
///
/// # Errors
/// Returns an error if the browser document cannot be accessed or modified.
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

/// Attaches the Winit canvas to the configured browser parent, if present.
/// `window` supplies the canvas, `key` labels it, and `parent_id` selects its DOM parent when set.
///
/// # Errors
/// Returns an error if the document, canvas, or configured parent element is unavailable.
pub fn attach_web_canvas(
    window: &Window,
    key: &WindowKey,
    parent_id: Option<&str>,
) -> Result<(), String> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| "browser document is unavailable".to_owned())?;
    install_canvas_style(&document)?;
    let canvas = window
        .canvas()
        .ok_or_else(|| "winit did not create a canvas".to_owned())?;
    canvas
        .set_attribute("data-argui-window", key.as_str())
        .map_err(js_error)?;
    if let Some(parent_id) = parent_id {
        let parent = document
            .get_element_by_id(parent_id)
            .ok_or_else(|| format!("web parent element not found: {parent_id}"))?;
        parent.append_child(&canvas).map_err(js_error)?;
    }
    Ok(())
}

/// Returns the canvas backing extent in physical pixels.
///
/// Browser device emulation can report a CSS-sized Winit `inner_size` while
/// exposing a larger device pixel ratio. Measuring the attached canvas keeps
/// layout in CSS pixels and gives WGPU the full-resolution backing surface.
pub fn web_drawable_size(window: &Window) -> PhysicalSize<u32> {
    let fallback = window.inner_size();
    let Some(canvas) = window.canvas() else {
        return fallback;
    };
    let bounds = canvas.get_bounding_client_rect();
    let scale = web_sys::window().map_or_else(
        || window.scale_factor(),
        |browser| browser.device_pixel_ratio(),
    );
    let width = physical_extent(bounds.width(), scale);
    let height = physical_extent(bounds.height(), scale);
    if width == 0 || height == 0 {
        fallback
    } else {
        PhysicalSize::new(width, height)
    }
}

fn physical_extent(css_extent: f64, scale: f64) -> u32 {
    if !css_extent.is_finite() || !scale.is_finite() || css_extent <= 0.0 || scale <= 0.0 {
        return 0;
    }
    (css_extent * scale).round().clamp(1.0, f64::from(u32::MAX)) as u32
}

fn install_canvas_style(document: &web_sys::Document) -> Result<(), String> {
    if document
        .query_selector("style[data-argui-canvas-style]")
        .map_err(js_error)?
        .is_some()
    {
        return Ok(());
    }
    let style = document.create_element("style").map_err(js_error)?;
    style
        .set_attribute("data-argui-canvas-style", "")
        .map_err(js_error)?;
    style.set_text_content(Some(CANVAS_STYLE));
    document
        .head()
        .ok_or_else(|| "browser document has no head".to_owned())?
        .append_child(&style)
        .map_err(js_error)?;
    Ok(())
}

fn js_error(error: wasm_bindgen::JsValue) -> String {
    error.as_string().unwrap_or_else(|| format!("{error:?}"))
}
