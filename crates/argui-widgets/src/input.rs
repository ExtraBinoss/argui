use argui_paint::{Color, PaintStyle};
use argui_text::{TextColor, TextOverflow, TextStyle, TextWrap};
use argui_ui::{
    AlignItems, CaretStyle, Element, EventType, LayoutStyle, Role, StateSelector, StylePatch,
    StyleTransition, TextEdit, TextEditorSpec, TextInputFilter, ValueHandler, VisualState, percent,
};

#[cfg(feature = "textarea")]
use argui_text::TextContent;
#[cfg(feature = "textarea")]
use argui_ui::{Axes, Overflow, ScrollConfig, ScrollPropagation, ScrollbarGutter, ScrollbarStyle};

use crate::{TEXT_FIELD_SCOPE, TextFieldBehavior, TextFieldPart};

#[cfg(feature = "input")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum InputKind {
    #[default]
    Text,
    Search,
    Number,
    Arithmetic,
    Password,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InputStyle {
    pub layout: LayoutStyle,
    pub paint: PaintStyle,
    pub hovered: StylePatch,
    pub focused: StylePatch,
    pub transition: StyleTransition,
    pub text: TextStyle,
    pub placeholder: TextStyle,
    pub selection: Color,
    pub caret: CaretStyle,
}

impl InputStyle {
    /// Creates an input style from base paint and text typography.
    #[must_use]
    pub fn new(paint: PaintStyle, mut text: TextStyle) -> Self {
        text.wrap = TextWrap::None;
        let mut placeholder = text.clone();
        placeholder.color = TextColor::srgba(0.55, 0.60, 0.68, 1.0);
        placeholder.overflow = TextOverflow::Ellipsis(argui_text::EllipsisPosition::End);
        Self {
            layout: LayoutStyle {
                size: argui_ui::Dimensions {
                    width: percent(1.0),
                    height: argui_ui::auto(),
                },
                padding: argui_ui::sides(13.0, 10.0),
                align_items: Some(AlignItems::CENTER),
                flex_shrink: 0.0,
                ..LayoutStyle::default()
            },
            hovered: StylePatch::from_quad(paint.quad.clone()),
            focused: StylePatch::from_quad(paint.quad.clone()),
            transition: crate::theme::instant_hover(
                StyleTransition::default(),
                StateSelector::scope(TEXT_FIELD_SCOPE, VisualState::Hovered),
            ),
            paint,
            text,
            placeholder,
            selection: Color::srgba(0.20, 0.68, 0.94, 0.38),
            caret: CaretStyle::default(),
        }
    }

    #[must_use]
    /// Sets the style patch applied while the pointer hovers over the field.
    pub fn hovered(mut self, style: impl Into<StylePatch>) -> Self {
        self.hovered = style.into();
        self
    }

    #[must_use]
    /// Sets the style patch applied while the field has visible focus.
    pub fn focused(mut self, style: impl Into<StylePatch>) -> Self {
        self.focused = style.into();
        self
    }

    #[must_use]
    /// Sets the style and text transition behavior.
    pub fn transition(mut self, transition: StyleTransition) -> Self {
        self.transition = transition;
        self
    }

    #[must_use]
    /// Sets the text-selection highlight color.
    pub const fn selection(mut self, color: Color) -> Self {
        self.selection = color;
        self
    }

    #[must_use]
    /// Sets the caret appearance.
    pub fn caret(mut self, caret: CaretStyle) -> Self {
        self.caret = caret;
        self
    }
}

#[cfg(feature = "input")]
#[derive(Clone, Debug, PartialEq)]
pub struct Input {
    key: String,
    value: String,
    placeholder: String,
    style: InputStyle,
    kind: InputKind,
    enabled: bool,
    read_only: bool,
    label: Option<String>,
    description: Option<String>,
    invalid: bool,
    leading: Option<(Element, f32)>,
    input_handlers: Vec<ValueHandler<String>>,
    edit_handlers: Vec<ValueHandler<TextEdit>>,
    submit_handlers: Vec<ValueHandler<String>>,
}

#[cfg(feature = "input")]
impl Input {
    /// Creates an editable single-line input.
    ///
    /// `key` identifies the field, `value` is its controlled text, `placeholder` is shown
    /// when empty, and `style` configures its appearance.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        value: impl Into<String>,
        placeholder: impl Into<String>,
        style: InputStyle,
    ) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            placeholder: placeholder.into(),
            style,
            kind: InputKind::Text,
            enabled: true,
            read_only: false,
            label: None,
            description: None,
            invalid: false,
            leading: None,
            input_handlers: Vec::new(),
            edit_handlers: Vec::new(),
            submit_handlers: Vec::new(),
        }
    }

    #[must_use]
    /// Sets the input purpose and corresponding text filter; `kind` selects its semantics.
    pub const fn kind(mut self, kind: InputKind) -> Self {
        self.kind = kind;
        self
    }

    #[must_use]
    /// Sets whether the input accepts user editing; `enabled` controls interaction availability.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    /// Sets whether the text can be edited while remaining focusable; `read_only` controls editability.
    pub const fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    #[must_use]
    /// Sets the accessible name of the input; `label` supplies that name.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    #[must_use]
    /// Sets supplementary accessible description text.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    /// Sets whether the input is marked invalid.
    pub const fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    #[must_use]
    /// Adds a leading element and reserves `slot_width` logical pixels for it; `content` is rendered before the field.
    pub fn leading(mut self, content: Element, slot_width: f32) -> Self {
        self.leading = Some((content, slot_width.max(0.0)));
        self
    }

    /// Adds a callback receiving the newly edited controlled value.
    ///
    /// `handler` is normally created with `Context::input_callback`. Handlers
    /// run in declaration order through the normal input event pipeline.
    #[must_use]
    pub fn on_input(mut self, handler: ValueHandler<String>) -> Self {
        self.input_handlers.push(handler);
        self
    }

    /// Adds a callback receiving only the accepted UTF-8 range replacement.
    ///
    /// `handler` is normally created with `Context::edit_callback`. Prefer this
    /// over [`Self::on_input`] for large document buffers.
    #[must_use]
    pub fn on_edit(mut self, handler: ValueHandler<TextEdit>) -> Self {
        self.edit_handlers.push(handler);
        self
    }

    /// Adds a callback receiving the submitted value.
    ///
    /// `handler` is normally created with `Context::submit_callback`. Handlers
    /// run in declaration order through the normal submit event pipeline.
    #[must_use]
    pub fn on_submit(mut self, handler: ValueHandler<String>) -> Self {
        self.submit_handlers.push(handler);
        self
    }

    #[must_use]
    /// Builds the configured input element.
    pub fn build(self) -> Element {
        let mut style = self.style;
        if let Some((_, slot_width)) = self.leading.as_ref() {
            style.layout.padding.left = argui_ui::length(*slot_width);
        }
        let privacy = if self.kind == InputKind::Password {
            argui_ui::TextPrivacy::Password
        } else {
            argui_ui::TextPrivacy::Public
        };
        editor(
            EditorSpec {
                key: self.key,
                value: self.value,
                placeholder: self.placeholder,
                style,
                multiline: false,
                role: match self.kind {
                    InputKind::Text | InputKind::Password => Role::TextInput,
                    InputKind::Search => Role::SearchInput,
                    InputKind::Number | InputKind::Arithmetic => Role::TextInput,
                },
                filter: match self.kind {
                    InputKind::Text | InputKind::Search | InputKind::Password => {
                        TextInputFilter::Any
                    }
                    InputKind::Number => TextInputFilter::Decimal,
                    InputKind::Arithmetic => TextInputFilter::Arithmetic,
                },
                enabled: self.enabled,
                read_only: self.read_only,
                label: self.label,
                description: self.description,
                invalid: self.invalid,
                input_handlers: self.input_handlers,
                edit_handlers: self.edit_handlers,
                submit_handlers: self.submit_handlers,
            },
            self.leading,
        )
        .text_privacy(privacy)
    }
}

#[cfg(feature = "textarea")]
#[derive(Clone, Debug, PartialEq)]
pub struct TextArea {
    key: String,
    value: String,
    placeholder: String,
    style: InputStyle,
    scroll: ScrollConfig,
    content: Option<TextContent>,
    enabled: bool,
    read_only: bool,
    input_handlers: Vec<ValueHandler<String>>,
    edit_handlers: Vec<ValueHandler<TextEdit>>,
    submit_handlers: Vec<ValueHandler<String>>,
}

#[cfg(feature = "textarea")]
impl TextArea {
    /// Creates a multiline input with wrapping text.
    ///
    /// `key` identifies the field, `value` is its controlled text, `placeholder` is shown
    /// when empty, and `style` configures its appearance.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        value: impl Into<String>,
        placeholder: impl Into<String>,
        mut style: InputStyle,
    ) -> Self {
        style.text.wrap = TextWrap::WordOrGlyph;
        style.placeholder.wrap = TextWrap::WordOrGlyph;
        style.placeholder.overflow = TextOverflow::Clip;
        style.layout.align_items = Some(AlignItems::START);
        Self {
            key: key.into(),
            value: value.into(),
            placeholder: placeholder.into(),
            style,
            scroll: ScrollConfig::default().propagation(ScrollPropagation::Contain),
            content: None,
            enabled: true,
            read_only: false,
            input_handlers: Vec::new(),
            edit_handlers: Vec::new(),
            submit_handlers: Vec::new(),
        }
    }

    #[must_use]
    /// Sets the scroll behavior for overflowing text.
    pub fn scroll_config(mut self, scroll: ScrollConfig) -> Self {
        self.scroll = scroll;
        self
    }

    /// Sets how multiline content wraps inside the editor viewport.
    ///
    /// `wrap` controls both entered text and the placeholder. Code editors commonly
    /// use [`TextWrap::None`], while prose editors normally keep the default
    /// [`TextWrap::WordOrGlyph`].
    #[must_use]
    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.style.text.wrap = wrap;
        self.style.placeholder.wrap = wrap;
        self
    }

    /// Supplies rich text styling for the current controlled value.
    ///
    /// `content` must concatenate to the same text as the value passed to [`Self::new`].
    /// A mismatched value safely falls back to plain text until refreshed content arrives.
    #[must_use]
    pub fn rich_text(mut self, content: TextContent) -> Self {
        self.content = Some(content);
        self
    }

    #[must_use]
    /// Sets the scrollbar style for the text area.
    pub fn scrollbar(mut self, scrollbar: ScrollbarStyle) -> Self {
        self.scroll = self.scroll.scrollbar(scrollbar);
        self
    }

    #[must_use]
    /// Sets whether the text area accepts user editing; `enabled` controls interaction availability.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    /// Sets whether the text remains focusable but cannot be edited; `read_only` controls editability.
    pub const fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Adds a callback receiving the newly edited multiline value.
    ///
    /// `handler` is normally created with `Context::input_callback`.
    #[must_use]
    pub fn on_input(mut self, handler: ValueHandler<String>) -> Self {
        self.input_handlers.push(handler);
        self
    }

    /// Adds a callback receiving only the accepted UTF-8 range replacement.
    ///
    /// `handler` is normally created with `Context::edit_callback`. This avoids
    /// cloning the complete controlled value on every edit.
    #[must_use]
    pub fn on_edit(mut self, handler: ValueHandler<TextEdit>) -> Self {
        self.edit_handlers.push(handler);
        self
    }

    /// Adds a callback receiving the submitted multiline value.
    ///
    /// `handler` is normally created with `Context::submit_callback`.
    #[must_use]
    pub fn on_submit(mut self, handler: ValueHandler<String>) -> Self {
        self.submit_handlers.push(handler);
        self
    }

    #[must_use]
    /// Builds the configured text area.
    pub fn build(self) -> Element {
        let horizontal_overflow = if self.style.text.wrap == TextWrap::None {
            Overflow::Auto
        } else {
            Overflow::Hidden
        };
        let content = self.content;
        let mut element = editor(
            EditorSpec {
                key: self.key,
                value: self.value,
                placeholder: self.placeholder,
                style: self.style,
                multiline: true,
                role: Role::TextArea,
                filter: TextInputFilter::Any,
                enabled: self.enabled,
                read_only: self.read_only,
                label: None,
                description: None,
                invalid: false,
                input_handlers: self.input_handlers,
                edit_handlers: self.edit_handlers,
                submit_handlers: self.submit_handlers,
            },
            None,
        );
        if let Some(content) = content {
            element = element.text_editor_content(content);
        }
        element
            .overflow(Axes {
                x: horizontal_overflow,
                y: Overflow::Auto,
            })
            .scrollbar_gutter(ScrollbarGutter::Stable)
            .scroll_config(self.scroll)
    }
}

struct EditorSpec {
    key: String,
    value: String,
    placeholder: String,
    style: InputStyle,
    multiline: bool,
    role: Role,
    filter: TextInputFilter,
    enabled: bool,
    read_only: bool,
    label: Option<String>,
    description: Option<String>,
    invalid: bool,
    input_handlers: Vec<ValueHandler<String>>,
    edit_handlers: Vec<ValueHandler<TextEdit>>,
    submit_handlers: Vec<ValueHandler<String>>,
}

fn editor(spec: EditorSpec, leading: Option<(Element, f32)>) -> Element {
    let label = spec.label.unwrap_or_else(|| spec.placeholder.clone());
    let mut behavior = TextFieldBehavior::new(&spec.key, label, &spec.value, spec.role)
        .enabled(spec.enabled)
        .read_only(spec.read_only)
        .invalid(spec.invalid);
    if let Some(description) = spec.description {
        behavior = behavior.description(description);
    }
    let element = Element::text_editor(TextEditorSpec {
        value: spec.value,
        placeholder: spec.placeholder,
        multiline: spec.multiline,
        read_only: spec.read_only,
        filter: spec.filter,
        text: spec.style.text,
        placeholder_text: spec.style.placeholder,
        selection: spec.style.selection,
        caret: spec.style.caret,
    })
    .layout_style(spec.style.layout)
    .paint_style(spec.style.paint)
    .when(
        StateSelector::scope(TEXT_FIELD_SCOPE, VisualState::Hovered),
        spec.style.hovered,
    )
    .when(
        StateSelector::scope(TEXT_FIELD_SCOPE, VisualState::Focused),
        spec.style.focused,
    )
    .transition(spec.style.transition);
    let mut editor = behavior.decorate(TextFieldPart::Editor, element);
    for handler in spec.input_handlers {
        editor = editor.on(handler.direct_listener(EventType::Input));
    }
    for handler in spec.edit_handlers {
        editor = editor.on(handler.direct_listener(EventType::TextEdit));
    }
    for handler in spec.submit_handlers {
        editor = editor.on(handler.direct_listener(EventType::Submit));
    }
    let Some((leading, slot_width)) = leading else {
        return behavior.decorate(TextFieldPart::Root, editor);
    };
    let slot = behavior.decorate(
        TextFieldPart::Decoration,
        Element::row([leading])
            .absolute(argui_ui::Sides {
                left: argui_ui::length(0.0),
                right: argui_ui::auto(),
                top: argui_ui::length(0.0),
                bottom: argui_ui::auto(),
            })
            .width(argui_ui::length(slot_width))
            .height(percent(1.0))
            .align_items(AlignItems::CENTER)
            .justify_content(argui_ui::JustifyContent::CENTER),
    );
    behavior.decorate(
        TextFieldPart::Root,
        Element::container([editor, slot]).width(percent(1.0)),
    )
}
