use crate::{Button, ButtonBehavior, DialogAction, Sheet, SheetSide, WidgetTheme};
use argui_ui::{Element, Role, Semantics, UiEvent, length};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SidebarAction {
    SetCollapsed(bool),
    SetOpen(bool),
}

/// Reusable desktop navigation rail and modal mobile navigation, controlled by the caller.
#[derive(Clone, Debug)]
pub struct Sidebar {
    pub key: String,
    pub label: String,
    pub content: Element,
    pub rail: Option<Element>,
    pub header: Option<Element>,
    pub footer: Option<Element>,
    pub collapsed: bool,
    pub mobile: bool,
    pub open: bool,
    pub toggle_label: String,
}

impl Sidebar {
    /// Creates a labelled sidebar around `content`.
    /// `key` identifies the sidebar and `label` names its trigger.
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, content: Element) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            content,
            rail: None,
            header: None,
            footer: None,
            collapsed: false,
            mobile: false,
            open: false,
            toggle_label: "Toggle navigation".into(),
        }
    }

    #[must_use]
    /// Returns an open or close action when `event` targets the sidebar.
    pub fn action(&self, event: &UiEvent) -> Option<SidebarAction> {
        if self.mobile {
            return crate::DialogBehavior::new(&self.key, &self.label, self.open)
                .action(event)
                .map(|action| SidebarAction::SetOpen(action == DialogAction::Open));
        }
        ButtonBehavior::new(format!("{}::toggle", self.key), &self.toggle_label)
            .action(event)
            .map(|_| SidebarAction::SetCollapsed(!self.collapsed))
    }

    #[must_use]
    /// Builds the sidebar using `theme` for its control and surface styling.
    ///
    /// # Panics
    ///
    /// Panics if the generated toggle control is missing its expected semantic state.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let mut toggle = Button::new(
            format!("{}::toggle", self.key),
            &self.toggle_label,
            theme.ghost_button(),
        )
        .content(
            Element::column((0..3).map(|_| {
                Element::container([])
                    .width(length(16.0))
                    .height(length(2.0))
                    .background(theme.foreground)
            }))
            .gap(3.0),
        )
        .build();
        toggle
            .semantics
            .as_mut()
            .expect("sidebar toggle")
            .state
            .expanded = Some(if self.mobile {
            self.open
        } else {
            !self.collapsed
        });
        let compact = self.collapsed && !self.mobile;
        let content = if compact {
            self.rail.unwrap_or_else(|| Element::container([]))
        } else {
            self.content
        };
        let children = self
            .header
            .filter(|_| !compact)
            .into_iter()
            .chain([content])
            .chain(self.footer.filter(|_| !compact));
        let content = Element::column(children)
            .gap(12.0)
            .semantics(Semantics::new(Role::Navigation).label(&self.label));
        if self.mobile {
            let close = Button::new(
                format!("{}::close", self.key),
                "Close navigation",
                theme.outline_button(),
            )
            .build();
            let mut sheet = Sheet::new(
                self.key,
                self.label,
                self.open,
                toggle,
                Element::column([close, content]).gap(12.0),
            );
            sheet.side = SheetSide::Left;
            sheet.build(theme)
        } else {
            Element::column([toggle, content])
                .keyed(self.key)
                .width(length(if compact { 72.0 } else { 256.0 }))
                .padding(argui_ui::Sides::length(8.0))
                .gap(12.0)
                .background(theme.card)
        }
    }
}
