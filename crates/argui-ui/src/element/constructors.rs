use crate::{AlignItems, Display, FlexDirection, JustifyItems, LayoutStyle};
use argui_core::{Transform2D, TransformOrigin};
use argui_paint::PaintStyle;
use argui_text::{TextContent, TextStyle};

use super::{Element, ElementKind, TextEditorSpec};

impl Element {
    /// Creates a generic container from its child elements.
    ///
    /// * `children` — elements to retain as this container's children.
    #[must_use]
    pub fn container(children: impl IntoIterator<Item = Self>) -> Self {
        Self(std::rc::Rc::new(super::kind::ElementNode {
            inspectable: true,
            layout_boundary: false,
            retained_identity: None,
            native_content: None,
            desktop_backdrop: None,
            key: None,
            kind: ElementKind::Container,
            style: LayoutStyle::default(),
            direction_scope: None,
            paint: PaintStyle::default(),
            transform: Transform2D::IDENTITY,
            transform_origin: TransformOrigin::CENTER,
            interaction: None,
            hit_test: crate::HitTestStyle::default(),
            conditional_styles: crate::state::ConditionalStyles::default(),
            style_transition: None,
            state_scope: None,
            container_scope: None,
            active_states: Vec::new(),
            semantics: None,
            tooltip: None,
            semantic_hidden: false,
            semantic_scope: false,
            semantic_bindings: crate::SemanticBindings::default(),
            bindings: Vec::new(),
            layer: None,
            effects: Vec::new(),
            scroll: None,
            virtual_item: None,
            portal: None,
            focus_scope: None,
            text_privacy: crate::TextPrivacy::Public,
            text_history: None,
            action_scope: None,
            action: None,
            event_listeners: Vec::new(),
            user_select: crate::UserSelect::Auto,
            selection_style: None,
            selection_highlight: None,
            z_index: 0,
            children: children.into_iter().collect(),
        }))
    }

    /// Creates a flex container whose children are arranged in a row.
    ///
    /// * `children` — elements to place in the row.
    #[must_use]
    pub fn row(children: impl IntoIterator<Item = Self>) -> Self {
        let mut element = Self::container(children);
        element.style.display = Display::Flex;
        element.style.flex_direction = FlexDirection::Row;
        element
    }

    /// Creates a flex container whose children are arranged in a column.
    ///
    /// * `children` — elements to place in the column.
    #[must_use]
    pub fn column(children: impl IntoIterator<Item = Self>) -> Self {
        let mut element = Self::container(children);
        element.style.display = Display::Flex;
        element.style.flex_direction = FlexDirection::Column;
        element
    }

    /// Creates a grid container whose children stretch to their grid area.
    ///
    /// * `children` — elements to place in the grid.
    #[must_use]
    pub fn grid(children: impl IntoIterator<Item = Self>) -> Self {
        let mut element = Self::container(children);
        element.style.display = Display::Grid;
        element.style.align_items = Some(AlignItems::STRETCH);
        element.style.justify_items = Some(JustifyItems::STRETCH);
        element
    }

    /// Creates a text element with the default text style.
    ///
    /// * `value` — text content to display.
    #[must_use]
    pub fn text(value: impl Into<TextContent>) -> Self {
        Self::container([]).with_kind(ElementKind::Text {
            content: value.into(),
            style: TextStyle::default(),
        })
    }

    /// Creates a text editor from its value, placeholder, and style configuration.
    ///
    /// * `spec` — configuration for the editor and its displayed text.
    #[must_use]
    pub fn text_editor(spec: TextEditorSpec) -> Self {
        Self::container([]).with_kind(ElementKind::TextEditor {
            value: spec.value,
            placeholder: spec.placeholder,
            styled: None,
            multiline: spec.multiline,
            read_only: spec.read_only,
            filter: spec.filter,
            text: spec.text,
            placeholder_text: Box::new(spec.placeholder_text),
            selection: spec.selection,
            caret: spec.caret,
        })
    }

    /// Supplies styled content for a text editor whose plain text matches its value.
    ///
    /// * `content` — rich text runs used while their concatenated text equals the
    ///   controlled editor value. A mismatch safely falls back to plain text.
    #[must_use]
    pub fn text_editor_content(mut self, content: TextContent) -> Self {
        if let ElementKind::TextEditor { styled, .. } = &mut self.kind {
            *styled = Some(Box::new(content));
        }
        self
    }

    fn with_kind(mut self, kind: ElementKind) -> Self {
        self.kind = kind;
        self
    }
}
