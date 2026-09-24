//! Development bundle selection and native identity remapping for QuickJS reloads.

use std::{
    cell::RefCell,
    fs,
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Sender},
    },
    time::SystemTime,
};

use argui_runtime::{NativeHostControl, WireHostId, WireOperation};

use crate::{
    QuickJsGallery, decode_wire_operations,
    effects::registry_from_json,
    native_metrics::{control_request, parse_control},
    runner::RelayBatch,
    telemetry::JsCounts,
};

/// A development bundle polled only while a runtime file is configured.
pub(crate) struct BundleWatcher {
    path: PathBuf,
    seen: Option<(SystemTime, u64)>,
    pending: Option<(SystemTime, u64)>,
}

impl BundleWatcher {
    /// Watches `path` for a completed bundle. Returns a watcher whose absent file
    /// leaves the embedded bundle active.
    pub(crate) fn new(path: PathBuf) -> Self {
        Self {
            path,
            seen: None,
            pending: None,
        }
    }

    /// Reads the configured path after two polls with identical modification time
    /// and size, then verifies that both remained unchanged during the read.
    /// Returns the bundle text, or `None` when the file is absent or unstable.
    ///
    /// # Errors
    /// Returns a filesystem or UTF-8 error for an unreadable changed file.
    pub(crate) fn changed(&mut self) -> Result<Option<String>, String> {
        let metadata = match fs::metadata(&self.path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("{}: {error}", self.path.display())),
        };
        let signature = (
            metadata.modified().map_err(|error| error.to_string())?,
            metadata.len(),
        );
        if self.seen == Some(signature) {
            return Ok(None);
        }
        if self.pending != Some(signature) {
            self.pending = Some(signature);
            return Ok(None);
        }
        let source = fs::read_to_string(&self.path)
            .map_err(|error| format!("{}: {error}", self.path.display()))?;
        let after = fs::metadata(&self.path)
            .map_err(|error| format!("{}: {error}", self.path.display()))?;
        let after_signature = (
            after.modified().map_err(|error| error.to_string())?,
            after.len(),
        );
        if after_signature != signature {
            self.pending = Some(after_signature);
            return Ok(None);
        }
        self.seen = Some(signature);
        self.pending = None;
        Ok(Some(source))
    }
}

/// One QuickJS session's operation route, staged until its native mount succeeds.
pub(crate) struct Dispatch {
    pub(crate) generation: u32,
    pub(crate) root: Option<WireHostId>,
    pub(crate) pending: Vec<Vec<WireOperation>>,
    sender: Option<Sender<RelayBatch>>,
    batches: Arc<AtomicU64>,
    operations: Arc<AtomicU64>,
}

impl Dispatch {
    /// Creates a staged route whose native node IDs use `generation`.
    /// `batches` and `operations` count QuickJS commits for profiling.
    /// Returns a shared route retained by the QuickJS bridge callback.
    pub(crate) fn new(
        generation: u32,
        batches: Arc<AtomicU64>,
        operations: Arc<AtomicU64>,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            generation,
            root: None,
            pending: Vec::new(),
            sender: None,
            batches,
            operations,
        }))
    }

    /// Decodes `json` and rewrites its node IDs for this session.
    /// Stages the batch until activation, then forwards it to the native host.
    /// Returns an empty string on success and an error string otherwise.
    pub(crate) fn accept(&mut self, json: &str) -> String {
        let mut batch = match decode_wire_operations(json) {
            Ok(batch) => batch,
            Err(error) => return error,
        };
        self.batches.fetch_add(1, Ordering::Relaxed);
        self.operations
            .fetch_add(batch.len() as u64, Ordering::Relaxed);
        for operation in &mut batch {
            remap(operation, self.generation);
            if let WireOperation::SetRoot { id } = operation {
                self.root = *id;
            }
        }
        if let Some(sender) = &self.sender {
            sender
                .send(RelayBatch {
                    operations: batch,
                    controls: Vec::new(),
                    acknowledgement: None,
                })
                .map_or_else(
                    |_| "native UI thread closed".to_string(),
                    |()| String::new(),
                )
        } else {
            self.pending.push(batch);
            String::new()
        }
    }

    /// Uses `sender` for asynchronous commits after a validated native swap.
    pub(crate) fn activate(&mut self, sender: Sender<RelayBatch>) {
        self.sender = Some(sender);
    }

    /// Suppresses disposal transactions after its root has been replaced atomically.
    pub(crate) fn disable(&mut self) {
        self.sender = None;
        self.pending.clear();
    }
}

/// Rewrites every node ID in `operation` to the native `generation` for this session.
fn remap(operation: &mut WireOperation, generation: u32) {
    let assign = |id: &mut WireHostId| id.generation = generation;
    match operation {
        WireOperation::Create { id, .. }
        | WireOperation::SetProperty { id, .. }
        | WireOperation::SetListener { id, .. }
        | WireOperation::Remove { id } => assign(id),
        WireOperation::Insert {
            parent,
            child,
            before,
        } => {
            assign(parent);
            assign(child);
            if let Some(before) = before {
                assign(before);
            }
        }
        WireOperation::SetRoot { id } => {
            if let Some(id) = id {
                assign(id);
            }
        }
    }
}

/// Returns the `ARGUI_GALLERY_BUNDLE` file path for development builds.
/// Release builds always return `None` and keep the embedded source.
pub(crate) fn dev_bundle_path() -> Option<PathBuf> {
    if !cfg!(debug_assertions) {
        return None;
    }
    std::env::var_os("ARGUI_GALLERY_BUNDLE").map(PathBuf::from)
}

/// Builds `source` against `contract_json`, then atomically replaces the native root.
/// `gallery` and `dispatch` are the active session; `sender` sends the swap to
/// the native host, `counts` shares profiling counters, and `profile_enabled`
/// gates candidate renderer samples.
/// The old session remains live when compilation, mounting, or native validation fails.
///
/// # Errors
/// Returns a JavaScript, native commit, or transport error without replacing the old session.
pub(crate) fn reload_gallery(
    gallery: &mut QuickJsGallery,
    dispatch: &mut Rc<RefCell<Dispatch>>,
    source: &str,
    contract_json: &str,
    sender: &Sender<RelayBatch>,
    counts: &JsCounts,
    profile_enabled: &Arc<AtomicBool>,
) -> Result<(), String> {
    let generation = dispatch
        .borrow()
        .generation
        .checked_add(1)
        .ok_or("reload generations exhausted")?;
    let candidate_route = Dispatch::new(
        generation,
        Arc::clone(&counts.batches),
        Arc::clone(&counts.operations),
    );
    let route = Rc::clone(&candidate_route);
    let staged_controls = Rc::new(RefCell::new(Vec::<String>::new()));
    let controls = Rc::clone(&staged_controls);
    let control_sender = Rc::new(RefCell::new(None::<Sender<RelayBatch>>));
    let active_control_sender = Rc::clone(&control_sender);
    let control_enabled = Arc::clone(profile_enabled);
    let candidate = QuickJsGallery::new_with_control(
        source,
        contract_json,
        "mountGallery",
        move |json| route.borrow_mut().accept(&json),
        move |json| {
            if let Some(sender) = active_control_sender.borrow().as_ref() {
                control_request(&json, sender, &control_enabled)
            } else {
                controls.borrow_mut().push(json);
                String::new()
            }
        },
    )?;
    let effects = registry_from_json(&candidate.effect_definitions_json()?)?;
    if candidate_route.borrow().root.is_none() {
        return Err("new bundle did not mount a root".into());
    }
    let old_root = dispatch
        .borrow()
        .root
        .ok_or("mounted gallery has no root")?;
    let mut operations = vec![
        WireOperation::SetRoot { id: None },
        WireOperation::Remove { id: old_root },
    ];
    operations.extend(
        std::mem::take(&mut candidate_route.borrow_mut().pending)
            .into_iter()
            .flatten(),
    );
    let mut swap_controls = vec![NativeHostControl::ReplaceEffects(effects)];
    let mut later_controls = Vec::new();
    for control in std::mem::take(&mut *staged_controls.borrow_mut()) {
        let request: serde_json::Value =
            serde_json::from_str(&control).map_err(|error| error.to_string())?;
        if request.get("kind").and_then(serde_json::Value::as_str) == Some("registerSvg") {
            swap_controls.push(parse_control(&control)?);
        } else {
            later_controls.push(control);
        }
    }
    let (ack_sender, ack) = mpsc::channel();
    sender
        .send(RelayBatch {
            operations,
            controls: swap_controls,
            acknowledgement: Some(ack_sender),
        })
        .map_err(|error| error.to_string())?;
    ack.recv().map_err(|error| error.to_string())??;
    dispatch.borrow_mut().disable();
    if let Err(error) = gallery.dispose() {
        eprintln!("argui-hot-reload: old disposer: {error}");
    }
    if profile_enabled.swap(false, Ordering::Relaxed) {
        let _ = control_request(
            r#"{"kind":"profile","enabled":false}"#,
            sender,
            profile_enabled,
        );
    }
    candidate_route.borrow_mut().activate(sender.clone());
    *control_sender.borrow_mut() = Some(sender.clone());
    *dispatch = candidate_route;
    *gallery = candidate;
    for control in later_controls {
        let error = control_request(&control, sender, profile_enabled);
        if !error.is_empty() {
            eprintln!("argui-hot-reload: staged renderer control: {error}");
        }
    }
    Ok(())
}
