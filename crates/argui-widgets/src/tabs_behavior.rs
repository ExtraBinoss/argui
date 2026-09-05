use argui_ui::{
    CursorIcon, Element, GestureSet, Interaction, KeyboardActivation, Orientation, Role,
    SemanticAction, SemanticState, Semantics, StateName, StateScopeId, UiEvent, UiEventKind,
    UserSelect,
};

pub const TABS_SCOPE: StateScopeId = StateScopeId::new("tabs");
pub const TAB_SELECTED: StateName = StateName::new("selected");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TabsPart {
    Root,
    List,
    Trigger(usize),
    Label,
    Panel(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TabsAction {
    Select(usize),
}

#[derive(Clone, Debug)]
pub struct TabsBehavior {
    key: String,
    labels: Vec<String>,
    enabled: Vec<bool>,
    selected: usize,
}

impl TabsBehavior {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        tabs: impl IntoIterator<Item = (String, bool)>,
        selected: usize,
    ) -> Self {
        let (labels, enabled) = tabs.into_iter().unzip();
        Self {
            key: key.into(),
            labels,
            enabled,
            selected,
        }
    }

    #[must_use]
    pub fn trigger_key(&self, index: usize) -> String {
        format!("{}::tab::{index}", self.key)
    }

    #[must_use]
    pub fn decorate(&self, part: TabsPart, element: Element) -> Element {
        match part {
            TabsPart::Root => element.state_scope(TABS_SCOPE),
            TabsPart::List => element
                .semantics(Semantics::new(Role::TabList).orientation(Orientation::Horizontal)),
            TabsPart::Label => element.semantic_hidden(true),
            TabsPart::Panel(index) => element
                .keyed(format!("{}::panel::{index}", self.key))
                .semantics(
                    Semantics::new(Role::TabPanel)
                        .label(self.labels.get(index).cloned().unwrap_or_default()),
                ),
            TabsPart::Trigger(index) => {
                let enabled = self.enabled.get(index).copied().unwrap_or(false);
                let selected = index == self.selected;
                element
                    .keyed(self.trigger_key(index))
                    .user_select(UserSelect::None)
                    .active_state(TAB_SELECTED, selected)
                    .interaction(
                        Interaction::default()
                            .enabled(enabled)
                            .focusable(enabled)
                            .cursor(if enabled {
                                CursorIcon::Pointer
                            } else {
                                CursorIcon::NotAllowed
                            })
                            .gestures(GestureSet::default().tap(argui_ui::TapGesture::default()))
                            .keyboard_activation(KeyboardActivation::EnterOrSpace),
                    )
                    .semantics(
                        Semantics::new(Role::Tab)
                            .label(self.labels.get(index).cloned().unwrap_or_default())
                            .state(SemanticState {
                                selected,
                                disabled: !enabled,
                                ..SemanticState::default()
                            })
                            .position_in_set((index + 1) as u32, self.labels.len() as u32)
                            .action(SemanticAction::Click)
                            .action(SemanticAction::Focus),
                    )
            }
        }
    }

    #[must_use]
    pub fn action(&self, event: &UiEvent) -> Option<TabsAction> {
        if !matches!(event.kind, UiEventKind::Click(_)) {
            return None;
        }
        let index = event
            .target_key()?
            .strip_prefix(&format!("{}::tab::", self.key))?
            .parse::<usize>()
            .ok()?;
        self.enabled
            .get(index)
            .copied()
            .unwrap_or(false)
            .then_some(TabsAction::Select(index))
    }
}
