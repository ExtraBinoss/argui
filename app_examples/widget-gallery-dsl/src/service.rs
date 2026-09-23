//! Shared AOT/live host services for the DSL gallery.

use std::{cell::RefCell, rc::Rc, time::Duration};

use argui::{
    platform::file_picker::{FileDialog, FileFilter, FilePickerMode},
    runtime::{
        Context, Entity, Render,
        tasks::{self, TaskSlot},
    },
    ui::Element,
};
use argui_example_widget_gallery_dsl::Main;
#[cfg(feature = "argui-live")]
use argui_example_widget_gallery_dsl::MainLive;

/// One synchronous DSL callback request handed to the host task owner.
pub(crate) enum Request {
    File(String),
    Search(String),
    CancelSearch,
}

/// Named host API shared by generated AOT and live gallery components.
pub(crate) trait GalleryView: Render {
    /// Installs typed callbacks which queue requests for the task owner.
    ///
    /// `requests` receives emitted work. Returns a live ABI/binding error.
    fn bind_requests(&mut self, requests: Rc<RefCell<Vec<Request>>>) -> Result<(), String>;

    /// Returns the currently mounted page, or a live ABI error.
    fn selected_page(&self) -> Result<String, String>;

    /// Sets the visible file-picker result; returns an ABI/property error.
    fn set_file_result(&mut self, value: String) -> Result<(), String>;

    /// Sets search status, rows, and loading state; returns an ABI/property error.
    fn set_search_state(
        &mut self,
        status: String,
        rows: Vec<String>,
        loading: bool,
    ) -> Result<(), String>;
}

impl GalleryView for Main {
    fn bind_requests(&mut self, requests: Rc<RefCell<Vec<Request>>>) -> Result<(), String> {
        let file = Rc::clone(&requests);
        self.on_request_file(move |mode| {
            file.borrow_mut().push(Request::File(mode));
            "Opening system dialog…".into()
        });
        let search = Rc::clone(&requests);
        self.on_request_search(move |query| search.borrow_mut().push(Request::Search(query)));
        self.on_cancel_search(move || requests.borrow_mut().push(Request::CancelSearch));
        Ok(())
    }

    fn selected_page(&self) -> Result<String, String> {
        Ok(self.selected())
    }

    fn set_file_result(&mut self, value: String) -> Result<(), String> {
        Main::set_file_result(self, value);
        Ok(())
    }

    fn set_search_state(
        &mut self,
        status: String,
        rows: Vec<String>,
        loading: bool,
    ) -> Result<(), String> {
        self.set_search_status(status);
        self.set_search_results(argui::reactive::Model::new(rows));
        self.set_search_loading(loading);
        Ok(())
    }
}

#[cfg(feature = "argui-live")]
impl GalleryView for argui_dsl_runtime::LiveRuntime {
    fn bind_requests(&mut self, requests: Rc<RefCell<Vec<Request>>>) -> Result<(), String> {
        let facade = MainLive::attach(self).map_err(|error| error.to_string())?;
        let file = Rc::clone(&requests);
        facade
            .on_request_file(self, move |mode| {
                file.borrow_mut().push(Request::File(mode));
                "Opening system dialog…".into()
            })
            .map_err(|error| error.to_string())?;
        let search = Rc::clone(&requests);
        facade
            .on_request_search(self, move |query| {
                search.borrow_mut().push(Request::Search(query));
            })
            .map_err(|error| error.to_string())?;
        facade
            .on_cancel_search(self, move || {
                requests.borrow_mut().push(Request::CancelSearch);
            })
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn selected_page(&self) -> Result<String, String> {
        MainLive::attach(self)
            .map_err(|error| error.to_string())?
            .selected(self)
            .map_err(|error| error.to_string())
    }

    fn set_file_result(&mut self, value: String) -> Result<(), String> {
        MainLive::attach(self)
            .map_err(|error| error.to_string())?
            .set_file_result(self, value)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn set_search_state(
        &mut self,
        status: String,
        rows: Vec<String>,
        loading: bool,
    ) -> Result<(), String> {
        let facade = MainLive::attach(self).map_err(|error| error.to_string())?;
        facade
            .set_search_status(self, status)
            .map_err(|error| error.to_string())?;
        facade
            .set_search_results(self, argui::reactive::Model::new(rows))
            .map_err(|error| error.to_string())?;
        facade
            .set_search_loading(self, loading)
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

/// Task owner around the generated view; dropping it cancels pending deliveries.
pub(crate) struct GalleryHost<V: GalleryView> {
    view: Entity<V>,
    requests: Rc<RefCell<Vec<Request>>>,
    search: TaskSlot,
    file: TaskSlot,
}

impl<V: GalleryView> GalleryHost<V> {
    /// Binds the generated view and creates its retained task owner.
    ///
    /// `view` is an AOT or live root already mounted. Returns a binding/ABI error.
    pub(crate) fn new(mut view: V) -> Result<Self, String> {
        let requests = Rc::new(RefCell::new(Vec::new()));
        view.bind_requests(Rc::clone(&requests))?;
        Ok(Self {
            view: Entity::new(view),
            requests,
            search: TaskSlot::default(),
            file: TaskSlot::default(),
        })
    }

    /// Updates a mounted generated view and requests a new frame.
    ///
    /// `update` receives the business surface. A failed live ABI update becomes
    /// an error on stderr; no task writes a dead view.
    fn update_view(&self, update: impl FnOnce(&mut V) -> Result<(), String>) {
        self.view.update(|view, cx| {
            if let Err(error) = update(view) {
                eprintln!("gallery host update failed: {error}");
            }
            cx.notify();
        });
    }

    /// Starts the latest search and cancels delivery from any earlier request.
    ///
    /// `query` is the user input; `cx` owns the task and UI-thread completion.
    fn start_search(&mut self, query: String, cx: &mut Context<Self>) {
        self.update_view(|view| view.set_search_state("Searching…".into(), Vec::new(), true));
        if let Err(error) = cx.spawn_latest(
            &mut self.search,
            search_drafts(query),
            |host, result, cx| {
                if host
                    .view
                    .read(|view| view.selected_page().ok().as_deref() == Some("Async tasks"))
                {
                    let (status, rows) = match result {
                        Ok(Ok(rows)) => (format!("{} matching drafts", rows.len()), rows),
                        Ok(Err(error)) => (format!("Invalid query: {error}"), Vec::new()),
                        Err(error) => (format!("Search failed: {error}"), Vec::new()),
                    };
                    host.update_view(|view| view.set_search_state(status, rows, false));
                    cx.notify();
                }
            },
        ) {
            self.update_view(|view| {
                view.set_search_state(format!("Search failed: {error}"), Vec::new(), false)
            });
        }
    }

    /// Starts the latest file dialog without blocking the UI event handler.
    ///
    /// `mode` selects the dialog; `cx` owns cancellation and result delivery.
    fn start_file(&mut self, mode: String, cx: &mut Context<Self>) {
        self.update_view(|view| view.set_file_result("Opening system dialog…".into()));
        if let Err(error) = cx.spawn_latest(
            &mut self.file,
            open_file_dialog(mode),
            |host, result, cx| {
                if host
                    .view
                    .read(|view| view.selected_page().ok().as_deref() == Some("File picker"))
                {
                    let status = result.unwrap_or_else(|error| format!("Dialog failed: {error}"));
                    host.update_view(|view| view.set_file_result(status));
                    cx.notify();
                }
            },
        ) {
            self.update_view(|view| view.set_file_result(format!("Dialog failed: {error}")));
        }
    }
}

impl<V: GalleryView> Render for GalleryHost<V> {
    fn tasks_ready(&mut self, cx: &mut Context<Self>) {
        let selected = self
            .view
            .read(|view| view.selected_page().unwrap_or_default());
        if selected != "Async tasks" {
            self.search.cancel();
        }
        if selected != "File picker" {
            self.file.cancel();
        }
        let requests = self.requests.borrow_mut().drain(..).collect::<Vec<_>>();
        for request in requests {
            match request {
                Request::Search(query) if selected == "Async tasks" => self.start_search(query, cx),
                Request::File(mode) if selected == "File picker" => self.start_file(mode, cx),
                Request::CancelSearch => {
                    self.search.cancel();
                    self.update_view(|view| {
                        view.set_search_state("Cancelled".into(), Vec::new(), false)
                    });
                }
                _ => {}
            }
        }
    }

    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        cx.entity(&self.view)
    }

    fn image_assets(&self) -> Vec<argui::paint::ImageAsset> {
        self.view.read(Render::image_assets)
    }

    fn vector_assets(&self) -> Vec<argui::paint::VectorAsset> {
        self.view.read(Render::vector_assets)
    }

    fn effect_definitions(&self) -> Vec<argui::render::EffectDefinition> {
        self.view.read(Render::effect_definitions)
    }
}

/// Searches draft titles asynchronously and returns checked filter errors.
///
/// `query` is a title fragment or `id:N` minimum index. The result contains
/// matching titles in source order, or a parse error for a malformed index.
async fn search_drafts(query: String) -> Result<Vec<String>, String> {
    tasks::sleep(Duration::from_millis(200)).await;
    let minimum = query
        .strip_prefix("id:")
        .map(|value| value.parse::<usize>().map_err(|error| error.to_string()))
        .transpose()?;
    let words = [
        "Architecture",
        "Design review",
        "Release notes",
        "Meeting agenda",
        "Travel plans",
    ];
    let lower = query.to_lowercase();
    let mut rows = Vec::new();
    for index in 0..10_000 {
        let title = format!("Draft {index:05} · {}", words[index % words.len()]);
        if minimum.map_or_else(
            || title.to_lowercase().contains(&lower),
            |minimum| index >= minimum,
        ) {
            rows.push(title);
        }
        if index % 128 == 0 {
            tasks::yield_now().await;
        }
    }
    Ok(rows)
}

/// Opens the selected native dialog asynchronously and formats its outcome.
///
/// `mode` is file, files, folder, folders, or save. Returns a visible error
/// string for an unsupported mode or platform failure.
async fn open_file_dialog(mode: String) -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dialog = match mode.as_str() {
            "file" => FileDialog::new(FilePickerMode::File)
                .title("Open a document")
                .filter(FileFilter::new("Documents", ["txt", "md", "pdf"])),
            "files" => FileDialog::new(FilePickerMode::Files)
                .title("Select images")
                .filter(FileFilter::new("Images", ["png", "jpg", "jpeg", "webp"])),
            "folder" => FileDialog::new(FilePickerMode::Folder).title("Choose project folder"),
            "folders" => FileDialog::new(FilePickerMode::Folders).title("Choose source folders"),
            "save" => FileDialog::new(FilePickerMode::Save)
                .title("Choose export destination")
                .file_name("report.txt"),
            _ => return "Unknown file request.".into(),
        };
        match dialog.open().await {
            Ok(Some(files)) if !files.is_empty() => format!(
                "Selected {}:\n{}",
                files.len(),
                files
                    .iter()
                    .map(|file| file.path().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            Ok(_) => "No new selection.".into(),
            Err(error) => format!("Dialog failed: {error}"),
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = mode;
        "Use the native gallery for system file dialogs.".into()
    }
}
