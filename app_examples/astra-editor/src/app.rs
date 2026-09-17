use std::{cell::RefCell, collections::BTreeSet, time::Duration};

use argui::{
    animation::Frame,
    core::{Color, ColorScheme, Key, KeyState, Size},
    paint::VectorAsset,
    runtime::{Context, LayoutSnapshot, Render, ThemeRequest, tasks::TaskSlot},
    theme::ThemeMode,
    ui::{Element, EventType, TextEdit, UiEvent, UiEventKind},
    widgets::{TablerIcon, TreeNode, TreeViewCache, WidgetAssets, shadcn},
};
use web_time::Instant;

use crate::{
    syntax,
    workspace::{EntryKind, Project, TextMatch},
};

mod editor;
mod io;
mod search;
mod sidebar;
mod view;

const TREE_ANIMATION_SECONDS: f32 = 0.12;
const HIGHLIGHT_DEBOUNCE: Duration = Duration::from_millis(48);
const SEARCH_LIMIT: usize = 300;

/// Kind of instant workspace navigation currently shown by the search overlay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SearchMode {
    /// Search every indexed source line.
    Text,
    /// Search project paths for quick opening.
    Files,
}

impl SearchMode {
    /// Returns the accessible title for this search mode.
    const fn title(self) -> &'static str {
        match self {
            Self::Text => "Search across files",
            Self::Files => "Open a file",
        }
    }

    /// Returns the mode-specific input placeholder.
    const fn placeholder(self) -> &'static str {
        match self {
            Self::Text => "Search every file in the workspace…",
            Self::Files => "Type a file name or path…",
        }
    }
}

/// Retained global-search model and its virtual viewport state.
struct SearchState {
    mode: SearchMode,
    open: bool,
    query: String,
    text_matches: Vec<TextMatch>,
    file_matches: Vec<usize>,
    offset: f32,
    elapsed_micros: u128,
    progress: f32,
}

impl Default for SearchState {
    /// Creates a closed, empty global-search overlay.
    fn default() -> Self {
        Self {
            mode: SearchMode::Text,
            open: false,
            query: String::new(),
            text_matches: Vec::new(),
            file_matches: Vec::new(),
            offset: 0.0,
            elapsed_micros: 0,
            progress: 0.0,
        }
    }
}

/// Product-shaped Argui text editor example.
pub struct AstraEditor {
    project: Project,
    open_documents: Vec<usize>,
    active_document: usize,
    selected_tree_key: String,
    collapsed: BTreeSet<String>,
    tree_nodes: Vec<TreeNode>,
    tree_cache: RefCell<TreeViewCache>,
    explorer_offset: f32,
    explorer_width: f32,
    explorer_open: bool,
    compact: bool,
    explorer_revealed: bool,
    viewport: Size,
    theme_mode: ThemeMode,
    assets: WidgetAssets,
    search: SearchState,
    tree_reveal_root: Option<String>,
    tree_reveal_started_at: Instant,
    tree_reveal_complete: bool,
    reduced_motion: bool,
    focus_initialized: bool,
    loading_project: bool,
    picker_task: TaskSlot,
    highlight_task: TaskSlot,
    #[cfg(not(target_arch = "wasm32"))]
    scan_task: Option<argui::runtime::tasks::TaskHandle>,
    #[cfg(not(target_arch = "wasm32"))]
    save_task: Option<argui::runtime::tasks::TaskHandle>,
    notice: Option<String>,
}

impl Default for AstraEditor {
    /// Creates the editor around an embedded project without touching disk.
    fn default() -> Self {
        let mut editor = Self::with_project(Project::demo());
        editor.open_documents = vec![0, 3];
        editor.notice = Some("Bundled workspace · open a folder to edit your project".into());
        editor
    }
}

impl AstraEditor {
    /// Creates an editor around a preloaded nonempty project.
    ///
    /// `project` supplies the documents, navigator entries, and optional native root.
    /// The first document is active and at most two initial tabs are opened.
    ///
    /// # Panics
    ///
    /// Panics when `project` contains no editable documents.
    #[must_use]
    pub fn with_project(project: Project) -> Self {
        assert!(
            !project.documents.is_empty(),
            "Astra Editor requires at least one document"
        );
        let active_document = 0;
        let selected_tree_key = project.documents[active_document].path.clone();
        let assets = WidgetAssets::tabler_subset(
            Color::WHITE,
            [
                TablerIcon::Search,
                TablerIcon::Sun,
                TablerIcon::Moon,
                TablerIcon::Sidebar,
                TablerIcon::Close,
                TablerIcon::ChevronRight,
                TablerIcon::Folder,
                TablerIcon::File,
                TablerIcon::Rust,
                TablerIcon::Terminal,
                TablerIcon::ArrowDown,
                TablerIcon::Check,
                TablerIcon::Loader,
            ],
        );
        let tree_nodes = project_tree_nodes(&project, &assets);
        let open_documents = (0..project.documents.len().min(2)).collect();
        Self {
            project,
            open_documents,
            active_document,
            selected_tree_key,
            collapsed: BTreeSet::new(),
            tree_nodes,
            tree_cache: RefCell::new(TreeViewCache::default()),
            explorer_offset: 0.0,
            explorer_width: 248.0,
            explorer_open: true,
            compact: false,
            explorer_revealed: false,
            viewport: Size::new(1280.0, 800.0),
            theme_mode: ThemeMode::Light,
            assets,
            search: SearchState::default(),
            tree_reveal_root: None,
            tree_reveal_started_at: Instant::now(),
            tree_reveal_complete: true,
            reduced_motion: false,
            focus_initialized: false,
            loading_project: false,
            picker_task: TaskSlot::default(),
            highlight_task: TaskSlot::default(),
            #[cfg(not(target_arch = "wasm32"))]
            scan_task: None,
            #[cfg(not(target_arch = "wasm32"))]
            save_task: None,
            notice: Some("Preloaded workspace".into()),
        }
    }
}

impl AstraEditor {
    /// Returns the active immutable document.
    fn active_document(&self) -> &crate::workspace::Document {
        &self.project.documents[self.active_document]
    }

    /// Opens `document`, reusing an existing tab when possible.
    fn open_document(&mut self, document: usize, cx: &mut Context<Self>) {
        if document >= self.project.documents.len() {
            return;
        }
        if !self.open_documents.contains(&document) {
            self.open_documents.push(document);
        }
        self.active_document = document;
        self.selected_tree_key = self.project.documents[document].path.clone();
        if self.compact {
            self.explorer_revealed = false;
        }
        self.schedule_derived_refresh(document, Duration::ZERO, cx);
        cx.request_focus("code-editor");
        cx.notify();
    }

    /// Closes one tab while retaining unsaved in-memory document contents.
    fn close_document(&mut self, document: usize, cx: &mut Context<Self>) {
        let Some(position) = self
            .open_documents
            .iter()
            .position(|candidate| *candidate == document)
        else {
            return;
        };
        if self.open_documents.len() == 1 {
            return;
        }
        self.open_documents.remove(position);
        if self.active_document == document {
            let next = position.min(self.open_documents.len() - 1);
            self.active_document = self.open_documents[next];
            self.selected_tree_key = self.project.documents[self.active_document].path.clone();
            self.schedule_derived_refresh(self.active_document, Duration::ZERO, cx);
        }
        cx.notify();
    }

    /// Applies a text edit immediately and defers derived syntax/search work.
    fn edit_active(&mut self, edit: TextEdit, cx: &mut Context<Self>) {
        let was_dirty = self.project.documents[self.active_document].is_dirty();
        if self.project.documents[self.active_document]
            .apply_edit(&edit)
            .is_err()
        {
            self.notice = Some("Ignored a stale text edit; the editor was resynchronized".into());
            cx.notify();
            return;
        }
        self.schedule_derived_refresh(self.active_document, HIGHLIGHT_DEBOUNCE, cx);
        if was_dirty != self.project.documents[self.active_document].is_dirty() {
            cx.notify();
        }
    }

    /// Coalesces derived document work so it never runs in the synchronous typing path.
    fn schedule_derived_refresh(
        &mut self,
        document: usize,
        delay: Duration,
        cx: &mut Context<Self>,
    ) {
        if !self.project.documents[document].needs_derived_refresh() {
            return;
        }
        if !delay.is_zero() {
            let started = cx.spawn_latest(
                &mut self.highlight_task,
                async move {
                    argui::runtime::tasks::sleep(delay).await;
                    document
                },
                |editor, result, cx| {
                    let Ok(document) = result else {
                        return;
                    };
                    editor.schedule_derived_refresh(document, Duration::ZERO, cx);
                },
            );
            if started.is_err() {
                cx.notify();
            }
            return;
        }
        let (revision, path, source) = self.project.documents[document].derived_snapshot();
        let started = cx.spawn_latest(
            &mut self.highlight_task,
            async move {
                let search_content = source.to_lowercase();
                let highlighted = syntax::highlight(&path, &source);
                (document, revision, search_content, highlighted)
            },
            |editor, result, cx| {
                let Ok((document, revision, search_content, highlighted)) = result else {
                    return;
                };
                let Some(document) = editor.project.documents.get_mut(document) else {
                    return;
                };
                if document.apply_derived(revision, search_content, highlighted) {
                    if editor.search.open {
                        editor.refresh_search();
                    }
                    cx.notify();
                }
            },
        );
        if started.is_err() {
            cx.notify();
        }
    }

    /// Rebuilds the active in-memory search result set and records its elapsed time.
    fn refresh_search(&mut self) {
        let started = Instant::now();
        match self.search.mode {
            SearchMode::Text => {
                self.search.text_matches = self.project.search(&self.search.query, SEARCH_LIMIT);
                self.search.file_matches.clear();
            }
            SearchMode::Files => {
                self.search.file_matches = self
                    .project
                    .matching_files(&self.search.query, SEARCH_LIMIT);
                self.search.text_matches.clear();
            }
        }
        self.search.elapsed_micros = started.elapsed().as_micros();
        self.search.offset = 0.0;
    }

    /// Opens the requested search surface and focuses its query field.
    fn show_search(&mut self, mode: SearchMode, cx: &mut Context<Self>) {
        self.search.mode = mode;
        self.search.open = true;
        self.search.query.clear();
        if self.reduced_motion {
            self.search.progress = 1.0;
        }
        self.refresh_search();
        cx.request_focus("workspace-search");
        cx.notify();
    }

    /// Closes the search surface and restores focus to the editor.
    fn hide_search(&mut self, cx: &mut Context<Self>) {
        self.search.open = false;
        if self.reduced_motion {
            self.search.progress = 0.0;
        }
        cx.request_focus("code-editor");
        cx.notify();
    }

    /// Replaces the active project and resets project-scoped presentation state.
    fn install_project(&mut self, project: Project, cx: &mut Context<Self>) {
        self.highlight_task.cancel();
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(task) = self.save_task.take() {
            task.cancel();
        }
        self.project = project;
        self.tree_nodes = project_tree_nodes(&self.project, &self.assets);
        self.tree_cache = RefCell::new(TreeViewCache::default());
        self.open_documents = (0..self.project.documents.len().min(2)).collect();
        self.active_document = 0;
        self.selected_tree_key = self.project.documents[0].path.clone();
        self.collapsed.clear();
        self.explorer_offset = 0.0;
        self.loading_project = false;
        self.explorer_revealed = false;
        self.tree_reveal_root = None;
        self.tree_reveal_started_at = Instant::now();
        self.tree_reveal_complete = true;
        self.notice = Some(format!(
            "Indexed {} files · ready for instant search",
            self.project.documents.len()
        ));
        self.schedule_derived_refresh(self.active_document, Duration::ZERO, cx);
        cx.request_focus("code-editor");
        cx.notify();
    }

    /// Toggles between the deliberately supported light and dark palettes.
    fn toggle_theme(&mut self, cx: &mut Context<Self>) {
        self.theme_mode = match self.theme_mode {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark | ThemeMode::System => ThemeMode::Light,
        };
        cx.set_theme(self.theme_request());
        cx.notify();
    }

    /// Toggles the explorer in the active responsive presentation.
    fn toggle_explorer(&mut self, cx: &mut Context<Self>) {
        if self.compact {
            self.explorer_revealed = !self.explorer_revealed;
        } else {
            self.explorer_open = !self.explorer_open;
            cx.request_focus("code-editor");
        }
        cx.notify();
    }

    /// Returns the runtime theme request for the selected editor palette.
    fn theme_request(&self) -> ThemeRequest {
        ThemeRequest {
            color_scheme: Some(match self.theme_mode {
                ThemeMode::Dark => ColorScheme::Dark,
                ThemeMode::Light | ThemeMode::System => ColorScheme::Light,
            }),
            primary: Some(Color::from_srgb8(43, 110, 242)),
        }
    }

    /// Interprets application-wide keyboard shortcuts before focused controls.
    fn handle_shortcut(&mut self, event: &UiEvent, cx: &mut Context<Self>) -> bool {
        let UiEventKind::KeyInput(input) = &event.kind else {
            return false;
        };
        if input.state != KeyState::Pressed {
            return false;
        }
        if input.key == Key::Escape && self.search.open {
            self.hide_search(cx);
            return true;
        }
        if !input.modifiers.command() {
            return false;
        }
        match &input.key {
            Key::Character(value) if value.eq_ignore_ascii_case("s") => self.save_active(cx),
            Key::Character(value) if input.modifiers.shift && value.eq_ignore_ascii_case("f") => {
                self.show_search(SearchMode::Text, cx);
            }
            Key::Character(value) if !input.modifiers.shift && value.eq_ignore_ascii_case("p") => {
                self.show_search(SearchMode::Files, cx);
            }
            Key::Character(value) if value.eq_ignore_ascii_case("b") => {
                self.toggle_explorer(cx);
            }
            Key::Character(value) if value.eq_ignore_ascii_case("j") => {
                self.launch_terminal(cx);
            }
            _ => return false,
        }
        true
    }

    /// Tracks the virtual scroll offsets owned by explorer and search result lists.
    fn handle_scroll(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        let UiEventKind::Scrolled { offset, .. } = event.kind else {
            return;
        };
        let rerender = match event.target_key() {
            Some("project-tree") => {
                let previous = self.explorer_offset;
                self.explorer_offset = offset.y.max(0.0);
                self.project_tree_window_changed(previous, self.explorer_offset)
            }
            Some("workspace-search-results") => {
                let previous = self.search.offset;
                self.search.offset = offset.y.max(0.0);
                let row_height = if self.search.mode == SearchMode::Text {
                    60.0
                } else {
                    46.0
                };
                self.search_window_changed(row_height, previous, self.search.offset)
            }
            _ => return,
        };
        if rerender {
            cx.notify();
        }
    }

    /// Dispatches captured global key and virtual-scroll events.
    fn handle_event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if self.handle_shortcut(event, cx) {
            let _ = event.prevent_default();
            event.stop_propagation();
            return;
        }
        self.handle_scroll(event, cx);
    }

    /// Returns eased progress for the global-search overlay transition.
    fn search_progress(&self) -> f32 {
        let progress = self.search.progress;
        progress * progress * (3.0 - 2.0 * progress)
    }

    /// Returns eased progress for the current explorer reveal transition.
    fn tree_open_progress(&self) -> f32 {
        if self.reduced_motion || self.tree_reveal_root.is_none() {
            return 1.0;
        }
        let progress = (self.tree_reveal_started_at.elapsed().as_secs_f32()
            / TREE_ANIMATION_SECONDS)
            .clamp(0.0, 1.0);
        progress * progress * (3.0 - 2.0 * progress)
    }
}

impl Render for AstraEditor {
    /// Builds the responsive editor tree and installs captured global listeners.
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.reduced_motion = cx.environment().reduced_motion;
        if !self.focus_initialized {
            self.focus_initialized = true;
            cx.request_focus("code-editor");
        }
        if self.reduced_motion {
            self.search.progress = if self.search.open { 1.0 } else { 0.0 };
        }
        if self.tree_reveal_root.is_some() && self.tree_open_progress() >= 1.0 {
            self.tree_reveal_root = None;
            self.tree_reveal_complete = true;
        }
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let mut root = self
            .view(theme, cx)
            .safe_area(cx.environment().safe_area_insets);
        for event_type in [EventType::Key, EventType::Scroll] {
            root = root.on(cx.listener(event_type, Self::handle_event).capture(true));
        }
        root
    }

    /// Advances overlay presence and the brief descendant reveal while either is active.
    fn animation_frame(&mut self, frame: Frame, cx: &mut Context<Self>) {
        let search_target = if self.search.open { 1.0 } else { 0.0 };
        if self.search.progress != search_target {
            let direction = if self.search.open { 1.0 } else { -1.0 };
            self.search.progress = (self.search.progress
                + direction * frame.elapsed.as_secs_f64() as f32 / 0.14)
                .clamp(0.0, 1.0);
            cx.notify();
        }
        let presenting_tree = self.tree_reveal_root.is_some() && !self.tree_reveal_complete;
        if presenting_tree {
            cx.notify();
        } else if self.search.progress == search_target {
            cx.request_paint();
        }
    }

    /// Returns whether the retained presentation currently needs display-linked frames.
    fn wants_animation_frame(&self) -> bool {
        self.search.progress != if self.search.open { 1.0 } else { 0.0 }
            || (self.tree_reveal_root.is_some() && !self.tree_reveal_complete)
    }

    /// Updates responsive breakpoints and the available split-pane dimensions.
    fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
        let viewport = layout.viewport_size();
        let compact = viewport.width < 720.0;
        if self.viewport != viewport || self.compact != compact {
            self.viewport = viewport;
            self.compact = compact;
            if !compact {
                self.explorer_revealed = false;
            }
            cx.notify();
        }
    }

    /// Registers the embedded Tabler SVG subset used by the editor chrome.
    fn vector_assets(&self) -> Vec<VectorAsset> {
        self.assets.assets().to_vec()
    }
}

/// Builds stable tree rows once per installed project, including file-type SVG icons.
fn project_tree_nodes(project: &Project, assets: &WidgetAssets) -> Vec<TreeNode> {
    project
        .entries
        .iter()
        .map(|entry| TreeNode {
            key: entry.key.clone(),
            label: entry.label.clone(),
            depth: entry.depth,
            icon: Some(assets.vector_id(match entry.kind {
                EntryKind::Directory => TablerIcon::Folder,
                EntryKind::File(document) if project.documents[document].extension() == "rs" => {
                    TablerIcon::Rust
                }
                EntryKind::File(_) => TablerIcon::File,
            })),
        })
        .collect()
}
