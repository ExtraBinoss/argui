use argui_paint::{Color, PaintStyle};
use argui_text::{TextColor, TextOverflow, TextStyle, TextWrap};
use argui_ui::{
    AlignItems, Axes, CaretStyle, Element, LayoutStyle, Overflow, Role, ScrollConfig,
    ScrollPropagation, ScrollbarGutter, ScrollbarStyle, StateSelector, StylePatch, StyleTransition,
    TextEditorSpec, TextInputFilter, VisualState, percent,
};

use crate::{TEXT_FIELD_SCOPE, TextFieldBehavior, TextFieldPart};

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
    pub fn hovered(mut self, style: impl Into<StylePatch>) -> Self {
        self.hovered = style.into();
        self
    }

    #[must_use]
    pub fn focused(mut self, style: impl Into<StylePatch>) -> Self {
        self.focused = style.into();
        self
    }

    #[must_use]
    pub fn transition(mut self, transition: StyleTransition) -> Self {
        self.transition = transition;
        self
    }

    #[must_use]
    pub const fn selection(mut self, color: Color) -> Self {
        self.selection = color;
        self
    }

    #[must_use]
    pub fn caret(mut self, caret: CaretStyle) -> Self {
        self.caret = caret;
        self
    }
}

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
}

impl Input {
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
        }
    }

    #[must_use]
    pub const fn kind(mut self, kind: InputKind) -> Self {
        self.kind = kind;
        self
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub const fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    #[must_use]
    pub const fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    #[must_use]
    pub fn leading(mut self, content: Element, slot_width: f32) -> Self {
        self.leading = Some((content, slot_width.max(0.0)));
        self
    }

    #[must_use]
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
            },
            self.leading,
        )
        .text_privacy(privacy)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextArea {
    key: String,
    value: String,
    placeholder: String,
    style: InputStyle,
    scroll: ScrollConfig,
    enabled: bool,
    read_only: bool,
}

impl TextArea {
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
            enabled: true,
            read_only: false,
        }
    }

    #[must_use]
    pub fn scroll_config(mut self, scroll: ScrollConfig) -> Self {
        self.scroll = scroll;
        self
    }

    #[must_use]
    pub fn scrollbar(mut self, scrollbar: ScrollbarStyle) -> Self {
        self.scroll = self.scroll.scrollbar(scrollbar);
        self
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub const fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    #[must_use]
    pub fn build(self) -> Element {
        editor(
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
            },
            None,
        )
        .overflow(Axes {
            x: Overflow::Hidden,
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
    let editor = behavior.decorate(TextFieldPart::Editor, element);
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
