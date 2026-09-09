use crate::{Button, WidgetTheme};
use argui_ui::{ActionInvocation, ActionState, Element, LiveRegion, Role, Semantics};
use std::{collections::VecDeque, time::Duration};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastVariant {
    #[default]
    Information,
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: String,
    pub title: String,
    pub description: String,
    pub variant: ToastVariant,
    /// None means persistent until explicitly closed.
    pub duration: Option<Duration>,
    pub actions: Vec<(ActionInvocation, ActionState)>,
}

impl Toast {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: String::new(),
            variant: ToastVariant::default(),
            duration,
            actions: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastPause {
    Hover,
    Focus,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastInsertError {
    DuplicateId,
    Capacity,
}

#[derive(Clone, Debug)]
struct Entry {
    toast: Toast,
    remaining: Option<Duration>,
    hover: bool,
    focus: bool,
}

/// Component-owned queue. Call `advance` at `next_deadline`, and cancel that wakeup
/// when the host is unmounted. Queued notifications begin timing when made visible.
#[derive(Clone, Debug)]
pub struct ToastState {
    entries: VecDeque<Entry>,
    visible_limit: usize,
    capacity: usize,
    last: Duration,
}

impl ToastState {
    pub fn new(visible_limit: usize, capacity: usize, now: Duration) -> Self {
        assert!(visible_limit > 0 && capacity >= visible_limit);
        Self {
            entries: VecDeque::new(),
            visible_limit,
            capacity,
            last: now,
        }
    }

    pub fn visible(&self) -> impl Iterator<Item = &Toast> {
        self.entries
            .iter()
            .take(self.visible_limit)
            .map(|entry| &entry.toast)
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn insert(&mut self, toast: Toast, now: Duration) -> Result<(), ToastInsertError> {
        self.advance(now);
        if self.entries.iter().any(|entry| entry.toast.id == toast.id) {
            return Err(ToastInsertError::DuplicateId);
        }
        if self.entries.len() == self.capacity {
            return Err(ToastInsertError::Capacity);
        }
        self.entries.push_back(Entry {
            remaining: toast.duration,
            toast,
            hover: false,
            focus: false,
        });
        Ok(())
    }

    /// Replace content and restart this notification's duration, keeping its queue position.
    pub fn update(&mut self, toast: Toast, now: Duration) -> bool {
        self.advance(now);
        let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.toast.id == toast.id)
        else {
            return false;
        };
        entry.remaining = toast.duration;
        entry.toast = toast;
        true
    }

    pub fn close(&mut self, id: &str, now: Duration) -> bool {
        self.advance(now);
        let Some(index) = self.entries.iter().position(|entry| entry.toast.id == id) else {
            return false;
        };
        self.entries.remove(index);
        true
    }

    pub fn pause(&mut self, id: &str, reason: ToastPause, paused: bool, now: Duration) -> bool {
        self.advance(now);
        let Some(entry) = self.entries.iter_mut().find(|entry| entry.toast.id == id) else {
            return false;
        };
        let flag = match reason {
            ToastPause::Hover => &mut entry.hover,
            ToastPause::Focus => &mut entry.focus,
        };
        let changed = *flag != paused;
        *flag = paused;
        changed
    }

    pub fn next_deadline(&self) -> Option<Duration> {
        self.entries
            .iter()
            .take(self.visible_limit)
            .filter(|entry| !entry.hover && !entry.focus)
            .filter_map(|entry| {
                entry
                    .remaining
                    .and_then(|remaining| self.last.checked_add(remaining))
            })
            .min()
    }

    pub fn advance(&mut self, now: Duration) -> bool {
        let elapsed = now.saturating_sub(self.last);
        self.last = now.max(self.last);
        let mut expired = Vec::new();
        for (index, entry) in self.entries.iter_mut().take(self.visible_limit).enumerate() {
            if !entry.hover
                && !entry.focus
                && let Some(remaining) = &mut entry.remaining
            {
                *remaining = remaining.saturating_sub(elapsed);
                if remaining.is_zero() {
                    expired.push(index);
                }
            }
        }
        let changed = !expired.is_empty();
        for index in expired.into_iter().rev() {
            self.entries.remove(index);
        }
        changed
    }
}

pub struct ToastHost<'a> {
    pub key: &'a str,
    pub state: &'a ToastState,
    pub close_label: &'a str,
}

impl ToastHost<'_> {
    pub fn close_key(&self, id: &str) -> String {
        format!("{}::close::{id}", self.key)
    }
    pub fn close_action<'a>(&self, event: &'a argui_ui::UiEvent) -> Option<&'a str> {
        if !matches!(event.kind, argui_ui::UiEventKind::Click(_)) {
            return None;
        }
        let id = event
            .target_key()?
            .strip_prefix(&format!("{}::close::", self.key))?;
        self.state
            .visible()
            .any(|toast| toast.id == id)
            .then_some(id)
    }
    pub fn pause_action(&self, event: &argui_ui::UiEvent) -> Option<(String, ToastPause, bool)> {
        let key = event.target_key()?;
        for toast in self.state.visible() {
            let root = format!("{}::toast::{}", self.key, toast.id);
            match &event.kind {
                argui_ui::UiEventKind::Pointer(pointer) if key == root => {
                    let paused = match pointer.phase {
                        argui_core::PointerPhase::Entered => true,
                        argui_core::PointerPhase::Left => false,
                        _ => return None,
                    };
                    return Some((toast.id.clone(), ToastPause::Hover, paused));
                }
                argui_ui::UiEventKind::Focused | argui_ui::UiEventKind::Blurred
                    if key == self.close_key(&toast.id)
                        || toast.actions.iter().enumerate().any(|(index, _)| {
                            key == format!("{}::action::{}::{index}", self.key, toast.id)
                        }) =>
                {
                    return Some((
                        toast.id.clone(),
                        ToastPause::Focus,
                        matches!(event.kind, argui_ui::UiEventKind::Focused),
                    ));
                }
                _ => {}
            }
        }
        None
    }

    pub fn build(&self, theme: &WidgetTheme) -> Element {
        Element::column(self.state.visible().map(|toast| {
            let mut semantics = Semantics::new(if toast.variant == ToastVariant::Error {
                Role::Alert
            } else {
                Role::Status
            })
            .label(&toast.title)
            .description(&toast.description);
            semantics.live = if toast.variant == ToastVariant::Error {
                LiveRegion::Assertive
            } else {
                LiveRegion::Polite
            };
            let actions = toast
                .actions
                .iter()
                .enumerate()
                .map(|(index, (invocation, state))| {
                    Button::new(
                        format!("{}::action::{}::{index}", self.key, toast.id),
                        &state.label,
                        theme.outline_button(),
                    )
                    .enabled(state.enabled)
                    .build()
                    .action_from(*invocation)
                });
            Element::column([
                Element::text(toast.title.as_str()),
                Element::text(toast.description.as_str()),
                Element::row(
                    actions.chain(std::iter::once(
                        Button::new(
                            self.close_key(&toast.id),
                            self.close_label,
                            theme.ghost_button(),
                        )
                        .build(),
                    )),
                ),
            ])
            .keyed(format!("{}::toast::{}", self.key, toast.id))
            .interaction(argui_ui::Interaction::default())
            .semantics(semantics)
            .padding(argui_ui::Sides::length(12.0))
            .background(theme.background)
        }))
        .keyed(self.key)
        .gap(8.0)
    }
}
