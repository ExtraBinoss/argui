use argui_core::{Point, Size};

use crate::{
    CursorIcon, Element, GestureKind, GesturePhase, GestureSet, Inset, Interaction, LayoutStyle,
    Length, Position, UiEvent, UiEventKind,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ResizeAxes {
    Horizontal,
    Vertical,
    #[default]
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResizeConfig {
    pub axes: ResizeAxes,
    pub minimum: Size,
    pub maximum: Size,
}

impl ResizeConfig {
    #[must_use]
    pub const fn new(minimum: Size, maximum: Size) -> Self {
        Self {
            axes: ResizeAxes::Both,
            minimum,
            maximum,
        }
    }

    #[must_use]
    pub const fn axes(mut self, axes: ResizeAxes) -> Self {
        self.axes = axes;
        self
    }

    fn clamp(self, size: Size) -> Size {
        Size::new(
            size.width.clamp(self.minimum.width, self.maximum.width),
            size.height.clamp(self.minimum.height, self.maximum.height),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResizeEvent {
    Started(Size),
    Changed(Size),
    Ended(Size),
    Cancelled(Size),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResizeState {
    size: Size,
    start: Option<Size>,
}

impl ResizeState {
    #[must_use]
    pub const fn new(size: Size) -> Self {
        Self { size, start: None }
    }

    #[must_use]
    pub const fn size(self) -> Size {
        self.size
    }

    pub fn update(
        &mut self,
        event: &UiEvent,
        handle_key: &str,
        config: ResizeConfig,
    ) -> Option<ResizeEvent> {
        if event.key.as_deref() != Some(handle_key) {
            return None;
        }
        let UiEventKind::Gesture(gesture) = event.kind else {
            return None;
        };
        let GestureKind::Pan { total, .. } = gesture.kind else {
            return None;
        };
        match gesture.phase {
            GesturePhase::Started => {
                let start = self.size;
                self.start = Some(start);
                let delta = resize_delta(total, config.axes);
                self.size = config.clamp(Size::new(start.width + delta.x, start.height + delta.y));
                Some(ResizeEvent::Started(self.size))
            }
            GesturePhase::Changed => {
                let start = self.start.unwrap_or(self.size);
                let delta = resize_delta(total, config.axes);
                self.size = config.clamp(Size::new(start.width + delta.x, start.height + delta.y));
                Some(ResizeEvent::Changed(self.size))
            }
            GesturePhase::Ended => {
                self.start = None;
                Some(ResizeEvent::Ended(self.size))
            }
            GesturePhase::Cancelled => {
                if let Some(start) = self.start.take() {
                    self.size = start;
                }
                Some(ResizeEvent::Cancelled(self.size))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Resizable {
    layout: LayoutStyle,
    size: Size,
    content: Element,
    handle: Element,
    handle_key: String,
}

impl Resizable {
    #[must_use]
    pub fn new(
        size: Size,
        content: Element,
        handle_key: impl Into<String>,
        handle: Element,
    ) -> Self {
        Self {
            layout: LayoutStyle::default(),
            size,
            content,
            handle,
            handle_key: handle_key.into(),
        }
    }

    #[must_use]
    pub fn layout(mut self, layout: LayoutStyle) -> Self {
        self.layout = layout;
        self
    }

    #[must_use]
    pub fn build(mut self) -> Element {
        self.layout.width = Length::Px(self.size.width);
        self.layout.height = Length::Px(self.size.height);
        self.layout.position = Position::Relative;
        self.handle.style.position = Position::Absolute;
        self.handle.style.inset = Inset {
            right: Length::Px(0.0),
            bottom: Length::Px(0.0),
            ..Inset::default()
        };
        self.handle.key = Some(self.handle_key);
        self.handle.interaction = Some(
            Interaction::default()
                .cursor(CursorIcon::NwseResize)
                .gestures(GestureSet::NONE.pan()),
        );
        Element::container([self.content, self.handle]).layout_style(self.layout)
    }
}

const fn resize_delta(delta: Point, axes: ResizeAxes) -> Point {
    match axes {
        ResizeAxes::Horizontal => Point::new(delta.x, 0.0),
        ResizeAxes::Vertical => Point::new(0.0, delta.y),
        ResizeAxes::Both => delta,
    }
}
