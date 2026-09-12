use crate::navigation::Page;
use argui::{
    runtime::{Context, Render, tasks::TaskSlot},
    ui::{Element, EventType, UiEvent, UiEventKind},
    widgets::{HoverCardState, MessageScrollState, QuestionnaireState, shadcn},
};
use std::collections::BTreeSet;

mod conversation;
mod forms;
mod navigation;
mod surfaces;

pub(crate) const PAGES: [Page; 26] = [
    Page::Accordion,
    Page::AlertDialog,
    Page::Attachment,
    Page::Bubble,
    Page::ButtonGroup,
    Page::Carousel,
    Page::Chart,
    Page::Combobox,
    Page::Direction,
    Page::Drawer,
    Page::Field,
    Page::HoverCard,
    Page::InputGroup,
    Page::InputOtp,
    Page::Item,
    Page::Marker,
    Page::Message,
    Page::MessageScroller,
    Page::NativeSelect,
    Page::NavigationMenu,
    Page::Questionnaire,
    Page::ScrollArea,
    Page::Sheet,
    Page::Sidebar,
    Page::Toggle,
    Page::ToggleGroup,
];

/// Each page retains its own small demonstration state while navigating the gallery.
pub(crate) struct CatalogueDemo {
    page: Page,
    text: String,
    status: String,
    open: bool,
    selected: usize,
    highlighted: Option<usize>,
    active: Option<String>,
    choices: BTreeSet<String>,
    collapsed: bool,
    offset: f32,
    hover: HoverCardState,
    timer: TaskSlot,
    origin: web_time::Instant,
    questionnaire: QuestionnaireState,
    messages: usize,
    message_scroll: MessageScrollState,
    scroll_maximum: f32,
    append_count: Option<usize>,
}

impl CatalogueDemo {
    pub(crate) fn reset_transient(&mut self) {
        self.timer.cancel();
        self.hover.reset();
        if matches!(
            self.page,
            Page::Sheet | Page::Drawer | Page::AlertDialog | Page::Combobox | Page::NativeSelect
        ) {
            self.open = false;
        }
        if self.page == Page::NavigationMenu {
            self.active = None;
        }
        self.offset = 0.0;
    }

    pub(crate) fn new(page: Page) -> Self {
        Self {
            page,
            text: String::new(),
            status: String::new(),
            open: false,
            selected: 0,
            highlighted: None,
            active: None,
            choices: BTreeSet::new(),
            collapsed: false,
            offset: 0.0,
            hover: HoverCardState::new("profile"),
            timer: TaskSlot::default(),
            origin: web_time::Instant::now(),
            questionnaire: QuestionnaireState::default(),
            messages: 8,
            message_scroll: MessageScrollState::default(),
            scroll_maximum: 0.0,
            append_count: Some(0),
        }
    }

    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        let palette = shadcn(cx.environment().primary);
        let theme = palette.resolve(cx.environment().color_scheme);
        let handled = match self.page {
            Page::Field
            | Page::InputGroup
            | Page::InputOtp
            | Page::Combobox
            | Page::NativeSelect
            | Page::Questionnaire => self.forms_event(event, theme, cx),
            Page::Accordion
            | Page::Toggle
            | Page::ToggleGroup
            | Page::NavigationMenu
            | Page::Sidebar
            | Page::ButtonGroup
            | Page::Direction => self.navigation_event(event, theme, cx),
            Page::Sheet
            | Page::AlertDialog
            | Page::Drawer
            | Page::Carousel
            | Page::Chart
            | Page::HoverCard => self.surfaces_event(event, theme, cx),
            _ => self.conversation_event(event, theme, cx),
        };
        if handled {
            event.stop_propagation();
            if self.page != Page::MessageScroller
                && matches!(&event.kind, UiEventKind::KeyInput(input) if input.key != argui::core::Key::Tab)
            {
                let _ = event.prevent_default();
            }
            cx.notify();
        }
    }
}

impl Render for CatalogueDemo {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let palette = shadcn(cx.environment().primary);
        let theme = palette.resolve(cx.environment().color_scheme);
        let view = match self.page {
            Page::Field
            | Page::InputGroup
            | Page::InputOtp
            | Page::Combobox
            | Page::NativeSelect
            | Page::Questionnaire => self.forms_view(theme),
            Page::Accordion
            | Page::Toggle
            | Page::ToggleGroup
            | Page::NavigationMenu
            | Page::Sidebar
            | Page::ButtonGroup
            | Page::Direction => self.navigation_view(theme),
            Page::Sheet
            | Page::AlertDialog
            | Page::Drawer
            | Page::Carousel
            | Page::Chart
            | Page::HoverCard => self.surfaces_view(theme, cx),
            _ => self.conversation_view(theme),
        };
        let mut root = Element::column([
            view,
            crate::app::text(&self.status, 14.0, theme.muted_foreground, 400),
        ])
        .keyed("catalogue-demo")
        .gap(18.0)
        .max_width(argui::ui::percent(1.0));
        for kind in [
            EventType::Click,
            EventType::Key,
            EventType::Input,
            EventType::Gesture,
            EventType::Dismiss,
            EventType::PointerOutside,
            EventType::Scroll,
            EventType::SelectionChange,
        ] {
            root = root.on(cx.listener(kind, Self::event));
        }
        root
    }

    fn layout_changed(&mut self, layout: &argui::runtime::LayoutSnapshot, cx: &mut Context<Self>) {
        if self.page != Page::MessageScroller {
            return;
        }
        if let (Some(viewport), Some(content)) =
            (layout.bounds("chat"), layout.bounds("chat-messages"))
        {
            self.scroll_maximum = (content.size.height - viewport.size.height).max(0.0);
            let unread = self.message_scroll.unread;
            if let Some(count) = self.append_count.take()
                && let Some(request) =
                    self.message_scroll
                        .appended("chat", count, self.scroll_maximum)
            {
                cx.scroll(request);
            }
            if self.message_scroll.unread != unread {
                cx.notify();
            }
        }
    }
}
