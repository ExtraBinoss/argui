use std::time::Duration;

use argui_core::{Key, KeyState, PointerKind, PointerPhase};
use argui_paint::{Border, CornerRadii, LayerStyle, PaintStyle, QuadStyle};
use argui_text::TextStyle;
use argui_ui::{
    Element, FloatingPlacement, FocusPolicy, Interaction, Placement, Role, Semantics, Sides,
    UiEvent, UiEventKind, WindowLayer, length,
};

use crate::WidgetTheme;

mod host;
pub use host::TooltipHost;

/// A non-interactive description anchored to a trigger. Visibility is controlled by the caller.
#[derive(Clone, Debug)]
pub struct Tooltip {
    key: String,
    surface: Option<argui_ui::OverlaySurface>,
    description: String,
    open: bool,
    trigger: Element,
    placement: FloatingPlacement,
    max_width: f32,
    paint: Option<PaintStyle>,
    layer: Option<LayerStyle>,
}

impl Tooltip {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        description: impl Into<String>,
        open: bool,
        trigger: Element,
    ) -> Self {
        Self {
            key: key.into(),
            surface: None,
            description: description.into(),
            open,
            trigger,
            placement: FloatingPlacement::new(Placement::Top),
            max_width: 280.0,
            paint: None,
            layer: None,
        }
    }

    #[must_use]
    pub const fn placement(mut self, placement: FloatingPlacement) -> Self {
        self.placement = placement;
        self
    }

    #[must_use]
    pub const fn max_width(mut self, width: f32) -> Self {
        self.max_width = width;
        self
    }

    #[must_use]
    pub fn paint(mut self, paint: PaintStyle) -> Self {
        self.paint = Some(paint);
        self
    }

    /// Supply any foreground/backdrop filter, including a registered custom effect.
    #[must_use]
    pub fn layer(mut self, layer: LayerStyle) -> Self {
        self.layer = Some(layer);
        self
    }

    #[must_use]
    pub const fn surface(mut self, surface: argui_ui::OverlaySurface) -> Self {
        self.surface = Some(surface);
        self
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let content_key = format!("{}::content", self.key);
        let mut trigger = self.trigger.keyed(self.key.clone());
        // A controlled tooltip owns this trigger; do not also create automatic help.
        trigger.tooltip = None;
        trigger
            .interaction
            .get_or_insert_with(|| Interaction::default().focus_policy(FocusPolicy::TabStop));
        let semantics = trigger
            .semantics
            .get_or_insert_with(|| Semantics::new(Role::Group));
        semantics.description = Some(self.description.clone());
        if self.open {
            trigger = trigger.described_by([content_key.clone()]);
        }
        let content = self.open.then(|| {
            Element::column([Element::text(self.description.clone())
                .shrink(0.0)
                .max_width(length((self.max_width - 20.0).max(0.0)))
                .text_style(TextStyle {
                    font_size: 14.0,
                    line_height: 20.0,
                    color: theme.foreground,
                    ..TextStyle::default()
                })])
            .overflow(argui_ui::Axes {
                x: argui_ui::Overflow::Auto,
                y: argui_ui::Overflow::Auto,
            })
            .scroll_config(
                argui_ui::ScrollConfig::default()
                    .propagation(argui_ui::ScrollPropagation::Contain)
                    .scrollbar(theme.scrollbar.clone()),
            )
            .keyed(content_key)
            .max_width(length(self.max_width.max(0.0)))
            .padding(Sides {
                left: length(10.0),
                right: length(10.0),
                top: length(7.0),
                bottom: length(7.0),
            })
            .paint_style(self.paint.unwrap_or_else(|| {
                PaintStyle::new(
                    QuadStyle::solid(theme.popover)
                        .border(Border::all(1.0, theme.popover_border))
                        .radius(CornerRadii::all(6.0)),
                )
            }))
            .layer(self.layer.unwrap_or_else(|| theme.overlay_layer(6.0, 0.0)))
            .interaction(Interaction::default())
            .semantics(Semantics::new(Role::Tooltip).label(self.description))
            .anchored_portal(WindowLayer::Popover, self.key, self.placement)
        });
        Element::container(std::iter::once(trigger).chain(content.map(|content| {
            if let Some(surface) = self.surface {
                content.portal_surface(surface)
            } else {
                content
            }
        })))
    }
}

/// Event-driven hover/focus visibility. Schedule `advance` at `next_deadline`; no idle polling.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TooltipState {
    key: String,
    content_key: String,
    hovered: [bool; 2],
    focused: bool,
    hover_dismissed: bool,
    focus_dismissed: bool,
    open: bool,
    delay: Duration,
    deadline: Option<(Duration, bool)>,
}

impl TooltipState {
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        let key = key.into();
        Self {
            content_key: format!("{key}::content"),
            key,
            hovered: [false; 2],
            focused: false,
            hover_dismissed: false,
            focus_dismissed: false,
            open: false,
            delay: Duration::from_millis(350),
            deadline: None,
        }
    }

    #[must_use]
    pub const fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    #[must_use]
    pub fn next_deadline(&self) -> Option<Duration> {
        self.deadline.map(|(time, _)| time)
    }

    /// Clear transient interaction state when the trigger is removed or its view is hidden.
    pub fn reset(&mut self) {
        self.hovered = [false; 2];
        self.focused = false;
        self.hover_dismissed = false;
        self.focus_dismissed = false;
        self.open = false;
        self.deadline = None;
    }

    pub fn update(&mut self, event: &UiEvent, now: Duration) -> bool {
        let previous = (
            self.hovered,
            self.focused,
            self.hover_dismissed,
            self.focus_dismissed,
            self.open,
            self.deadline,
        );
        let trigger = event.target_key() == Some(self.key.as_str());
        let content = event.target_key() == Some(self.content_key.as_str());
        match &event.kind {
            UiEventKind::Pointer(pointer)
                if (trigger || content) && pointer.kind != PointerKind::Touch =>
            {
                match pointer.phase {
                    PointerPhase::Entered => {
                        if trigger && !self.hovered[0] {
                            self.hover_dismissed = false;
                        }
                        self.hovered[usize::from(content)] = true;
                    }
                    PointerPhase::Left | PointerPhase::Cancelled => {
                        self.hovered[usize::from(content)] = false;
                        if trigger {
                            self.hover_dismissed = false;
                        }
                    }
                    PointerPhase::Pressed => self.dismiss(),
                    _ => return false,
                }
            }
            UiEventKind::DismissRequested if content => self.dismiss(),
            UiEventKind::Focused if trigger => self.focused = true,
            UiEventKind::Blurred if trigger => {
                self.focused = false;
                self.focus_dismissed = false;
            }
            UiEventKind::Click(_) if trigger => self.dismiss(),
            UiEventKind::KeyInput(input)
                if (self.open || self.deadline.is_some())
                    && input.state == KeyState::Pressed
                    && input.key == Key::Escape =>
            {
                self.dismiss();
            }
            _ => return false,
        }
        let focused = self.focused && !self.focus_dismissed;
        let hovered = !self.hover_dismissed && self.hovered.iter().any(|hovered| *hovered);
        if self.hover_dismissed && self.focus_dismissed {
            self.open = false;
            self.deadline = None;
        } else if focused || hovered {
            if focused || self.open || self.delay.is_zero() {
                self.open = true;
                self.deadline = None;
            } else if self.deadline.is_none() {
                self.deadline = Some((now.saturating_add(self.delay), true));
            }
        } else if self.open {
            // Allow the pointer to cross the anchor gap into the tooltip itself.
            self.deadline = Some((now.saturating_add(Duration::from_millis(100)), false));
        } else {
            self.deadline = None;
        }
        (
            self.hovered,
            self.focused,
            self.hover_dismissed,
            self.focus_dismissed,
            self.open,
            self.deadline,
        ) != previous
    }

    fn dismiss(&mut self) {
        self.hover_dismissed = true;
        self.focus_dismissed = true;
    }

    pub fn advance(&mut self, now: Duration) -> bool {
        let Some((deadline, open)) = self.deadline else {
            return false;
        };
        if now < deadline {
            return false;
        }
        self.deadline = None;
        let changed = self.open != open;
        self.open = open;
        changed
    }
}
