use std::{error::Error, fmt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClipboardError(String);

impl fmt::Display for ClipboardError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for ClipboardError {}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct Clipboard {
    inner: Option<arboard::Clipboard>,
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg_attr(coverage_nightly, coverage(off))]
impl Clipboard {
    #[must_use]
    pub const fn new() -> Self {
        Self { inner: None }
    }

    pub fn read_text(&mut self) -> Result<String, ClipboardError> {
        self.inner()?.get_text().map_err(error)
    }

    pub fn write_text(&mut self, text: String) -> Result<(), ClipboardError> {
        self.inner()?.set_text(text).map_err(error)
    }

    fn inner(&mut self) -> Result<&mut arboard::Clipboard, ClipboardError> {
        if self.inner.is_none() {
            self.inner = Some(arboard::Clipboard::new().map_err(error)?);
        }
        Ok(self.inner.as_mut().expect("clipboard was initialized"))
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn error(error: arboard::Error) -> ClipboardError {
    ClipboardError(error.to_string())
}

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
pub struct Clipboard;

#[cfg(target_arch = "wasm32")]
#[cfg_attr(coverage_nightly, coverage(off))]
impl Clipboard {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    pub async fn read_text(&mut self) -> Result<String, ClipboardError> {
        let clipboard = web_sys::window()
            .ok_or_else(|| ClipboardError("browser window is unavailable".into()))?
            .navigator()
            .clipboard();
        let value = wasm_bindgen_futures::JsFuture::from(clipboard.read_text())
            .await
            .map_err(|value| ClipboardError(format!("{value:?}")))?;
        value
            .as_string()
            .ok_or_else(|| ClipboardError("clipboard returned non-text data".into()))
    }

    pub async fn write_text(&mut self, text: String) -> Result<(), ClipboardError> {
        let clipboard = web_sys::window()
            .ok_or_else(|| ClipboardError("browser window is unavailable".into()))?
            .navigator()
            .clipboard();
        wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&text))
            .await
            .map_err(|value| ClipboardError(format!("{value:?}")))?;
        Ok(())
    }
}
