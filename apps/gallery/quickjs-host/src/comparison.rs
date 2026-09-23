//! A frozen Rust presentation of the exact Solid Animation Lab host tree.

use std::{cell::RefCell, rc::Rc};

use argui_host::Host;
use argui_platform::WindowKey;
use argui_runtime::{
    AppEvent, AppModel, AppUpdate, NativeHostAssets, WindowEnvironment, WireOperation,
};
use argui_ui::{Element, percent};
use serde_json::{Value, json};

use crate::QuickJsGallery;

/// Rust-owned immutable view and media for the same Animation Lab scene as QuickJS.
pub struct AnimationSnapshot {
    root: Element,
    assets: NativeHostAssets,
    wire_batches: Vec<Vec<WireOperation>>,
}

impl AnimationSnapshot {
    /// Mounts the bundled Solid app, opens Animation Lab, and freezes its validated root.
    /// `source` and `contract_json` must match the native host's generated ABI.
    /// `assets` are the decoded media used by both comparison variants.
    /// Returns a Rust model with no JavaScript runtime or event actor after construction.
    ///
    /// # Errors
    /// Returns an error if QuickJS, navigation, wire decoding, or a host commit fails.
    pub fn build(
        source: &str,
        contract_json: &str,
        assets: NativeHostAssets,
    ) -> Result<Self, String> {
        let batches = Rc::new(RefCell::new(Vec::<String>::new()));
        let captured = Rc::clone(&batches);
        let gallery = QuickJsGallery::new(source, contract_json, "mountGallery", move |batch| {
            captured.borrow_mut().push(batch);
            String::new()
        })?;
        let initial = batches.borrow().clone();
        let callback = navigation_callback(&initial)?;
        gallery.deliver(&callback.to_string())?;
        let mut host = Host::with_builtins().map_err(|error| error.to_string())?;
        let mut wire_batches = Vec::new();
        for batch in batches.borrow().iter() {
            let operations: Vec<WireOperation> = serde_json::from_str(batch)
                .map_err(|error| format!("invalid gallery operation batch: {error}"))?;
            wire_batches.push(operations.clone());
            let operations = operations
                .into_iter()
                .map(WireOperation::into_native)
                .collect::<Result<Vec<_>, _>>()?;
            host.commit(&operations)
                .map_err(|error| error.to_string())?;
        }
        let root = host
            .root_element()
            .ok_or("Animation Lab has no visible root")?;
        Ok(Self {
            root,
            assets,
            wire_batches,
        })
    }

    /// Returns a shared clone of the frozen root, for exact-scene assertions.
    pub fn root(&self) -> Element {
        self.root.clone()
    }

    /// Returns the exact mount and navigation transactions that formed the frozen scene.
    /// These are useful for matching a live host run without comparing callback closures.
    pub fn wire_batches(&self) -> &[Vec<WireOperation>] {
        &self.wire_batches
    }
}

impl AppModel for AnimationSnapshot {
    fn view(&self, window: &WindowKey, environment: WindowEnvironment) -> Option<Element> {
        if *window != WindowKey::main() {
            return None;
        }
        let mut root = self.root.clone();
        if root.paint.quad.background.is_none() && root.children.len() == 1 {
            root.paint.quad.background = root.children[0].paint.quad.background.clone();
        }
        Some(
            root.safe_area(environment.safe_area_insets)
                .width(percent(1.0))
                .height(percent(1.0)),
        )
    }

    fn update(&mut self, _event: &AppEvent) -> AppUpdate {
        AppUpdate::none()
    }

    fn image_assets(&self) -> Vec<argui_paint::ImageAsset> {
        self.assets.images.clone()
    }

    fn vector_assets(&self) -> Vec<argui_paint::VectorAsset> {
        self.assets.vectors.clone()
    }
}

/// Locates the initial gallery navigation callback without depending on generated numeric IDs.
pub(crate) fn navigation_callback(batches: &[String]) -> Result<Value, String> {
    let operations = batches
        .iter()
        .map(|batch| serde_json::from_str::<Vec<Value>>(batch))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let node = &operations
        .iter()
        .find(|operation| {
            operation["kind"] == "setProperty"
                && operation["value"]["value"] == "page-animation-lab"
        })
        .ok_or("Animation Lab navigation key is absent")?["id"];
    let callback = &operations
        .iter()
        .find(|operation| {
            operation["kind"] == "setListener"
                && operation["id"] == *node
                && operation["event"] == 1
        })
        .ok_or("Animation Lab navigation callback is absent")?["callback"];
    Ok(json!({"node": node, "callback": callback, "payload": {"kind": "click"}}))
}
