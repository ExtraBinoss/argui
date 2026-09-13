use argui_animation::{Duration, Frame};
use argui_core::Transform2D;
use argui_runtime::{Context, Render};
use argui_text::{FontFamily, TextStyle, TextWrap};
use argui_ui::{AlignItems, Axes, Element, Overflow, Role, Semantics, length};
use unicode_segmentation::UnicodeSegmentation;

use crate::{WidgetTheme, shadcn};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextAnimation {
    /// Digits roll through intermediate values; other graphemes slide once.
    #[default]
    Roll,
    Slide,
    Fade,
}

/// A retained, single-line text transition. Mount as an entity, or call `advance`
/// and `build` from a host. Only changed graphemes animate; one semantic text node
/// exposes the requested value, without announcing intermediate reel characters.
/// Updates during a transition coalesce into the latest value for the next roll.
#[derive(Clone, Debug)]
pub struct AnimatedText {
    key: String,
    value: String,
    from: String,
    to: String,
    progress: f32,
    first_frame: bool,
    duration: Duration,
    animation: TextAnimation,
    align_end: bool,
    reduced_motion: bool,
    font_size: f32,
    style: Option<TextStyle>,
    cached_style: Option<TextStyle>,
    columns: Vec<Column>,
}

#[derive(Clone, Debug)]
struct Column {
    content: Element,
    distance: f32,
}

impl AnimatedText {
    #[must_use]
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            key: key.into(),
            from: value.clone(),
            to: value.clone(),
            value,
            progress: 1.0,
            first_frame: false,
            duration: Duration::from_millis(420),
            animation: TextAnimation::Roll,
            align_end: true,
            reduced_motion: false,
            font_size: 16.0,
            style: None,
            cached_style: None,
            columns: Vec::new(),
        }
    }

    #[must_use]
    pub fn animation(mut self, animation: TextAnimation) -> Self {
        self.animation = animation;
        self.cached_style = None;
        self
    }

    #[must_use]
    pub const fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Align numeric places from the right (default), or text positions from the left.
    #[must_use]
    pub fn align_end(mut self, align_end: bool) -> Self {
        self.align_end = align_end;
        self.cached_style = None;
        self
    }

    /// Default text uses the theme foreground and a monospace family for stable digits.
    #[must_use]
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.style = Some(style);
        self.cached_style = None;
        self
    }

    #[must_use]
    pub fn font_size(mut self, size: f32) -> Self {
        assert!(
            size.is_finite() && size > 0.0,
            "font size must be finite and positive"
        );
        self.font_size = size;
        self.cached_style = None;
        self
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_text(&mut self, value: impl Into<String>) {
        let value = value.into();
        if self.value == value {
            return;
        }
        self.value = value;
        if self.reduced_motion || self.duration == Duration::ZERO {
            self.finish();
        } else if !self.is_animating() {
            self.start();
        }
    }

    pub fn set_reduced_motion(&mut self, reduced: bool) {
        self.reduced_motion = reduced;
        if reduced && (self.is_animating() || self.to != self.value) {
            self.finish();
        }
    }

    #[must_use]
    pub fn is_animating(&self) -> bool {
        self.progress < 1.0
    }

    /// Advance only while animating. No clocks, timers or background work remain at rest.
    pub fn advance(&mut self, elapsed: Duration) -> bool {
        if !self.is_animating() || elapsed == Duration::ZERO {
            return false;
        }
        self.progress =
            (self.progress + (elapsed.as_secs_f64() / self.duration.as_secs_f64()) as f32).min(1.0);
        if !self.is_animating() {
            if self.to != self.value {
                self.start();
            } else {
                self.finish();
            }
        }
        true
    }

    fn start(&mut self) {
        self.from.clone_from(&self.to);
        self.to.clone_from(&self.value);
        self.progress = 0.0;
        self.first_frame = true;
        self.cached_style = None;
    }

    fn finish(&mut self) {
        self.from.clone_from(&self.value);
        self.to.clone_from(&self.value);
        self.progress = 1.0;
        self.first_frame = false;
        self.cached_style = None;
    }

    #[must_use]
    pub fn build(&mut self, theme: &WidgetTheme) -> Element {
        let mut style = self.style.clone().unwrap_or_else(|| TextStyle {
            font_size: self.font_size,
            line_height: self.font_size * 1.25,
            color: theme.foreground,
            family: FontFamily::Monospace,
            ..TextStyle::default()
        });
        style.wrap = TextWrap::None;
        if self.cached_style.as_ref() != Some(&style) {
            self.columns = self.columns(&style);
            self.cached_style = Some(style);
        }
        let eased = 1.0 - (1.0 - self.progress).powi(3);
        Element::row(self.columns.iter().map(|column| {
            if column.distance == 0.0 {
                return column.content.clone();
            }
            let mut content = column.content.clone();
            let position = if self.animation == TextAnimation::Fade {
                f32::from(self.progress >= 0.5)
            } else {
                eased
            };
            content.children[0].transform = Transform2D::IDENTITY.translate(
                0.0,
                if column.distance < 0.0 {
                    column.distance * (1.0 - position)
                } else {
                    -column.distance * position
                },
            );
            if self.animation == TextAnimation::Fade {
                content = content.opacity((2.0 * self.progress - 1.0).abs());
            }
            content
        }))
        .keyed(self.key.clone())
        .align_items(AlignItems::CENTER)
        .semantics(Semantics::new(Role::Text).label(&self.value))
    }

    fn columns(&self, style: &TextStyle) -> Vec<Column> {
        let before: Vec<_> = self.from.graphemes(true).collect();
        let after: Vec<_> = self.to.graphemes(true).collect();
        let count = before.len().max(after.len());
        let decreasing = matches!((self.from.parse::<f64>(), self.to.parse::<f64>()),
            (Ok(from), Ok(to)) if to < from);
        let glyph = |value: &str| {
            Element::text(value)
                .text_style(style.clone())
                .height(length(style.line_height))
                .shrink(0.0)
        };
        (0..count)
            .map(|index| {
                let at = |len| index.checked_sub(if self.align_end { count - len } else { 0 });
                let from = at(before.len())
                    .and_then(|index| before.get(index))
                    .copied()
                    .unwrap_or("");
                let to = at(after.len())
                    .and_then(|index| after.get(index))
                    .copied()
                    .unwrap_or("");
                let slot = if self.align_end {
                    count - index - 1
                } else {
                    index
                };
                let key = format!("{}::glyph::{slot}", self.key);
                if from == to {
                    return Column {
                        content: glyph(to).keyed(key).semantic_hidden(true),
                        distance: 0.0,
                    };
                }
                let mut reel = vec![from.to_owned(), to.to_owned()];
                if self.animation == TextAnimation::Roll
                    && from.len() == 1
                    && to.len() == 1
                    && let (Ok(mut digit), Ok(target)) = (from.parse::<u8>(), to.parse::<u8>())
                {
                    reel.truncate(1);
                    while digit != target {
                        digit = (digit + if decreasing { 9 } else { 1 }) % 10;
                        reel.push(digit.to_string());
                    }
                }
                let distance = (reel.len() - 1) as f32 * style.line_height;
                let backwards = decreasing && self.animation == TextAnimation::Roll;
                if backwards {
                    reel.reverse();
                }
                let content =
                    Element::column([
                        Element::column(reel.iter().map(|text| glyph(text))).shrink(0.0)
                    ])
                    .keyed(key)
                    .height(length(style.line_height))
                    .shrink(0.0)
                    .overflow(Axes {
                        x: Overflow::Hidden,
                        y: Overflow::Hidden,
                    })
                    .semantic_hidden(true);
                Column {
                    content,
                    distance: if backwards { -distance } else { distance },
                }
            })
            .collect()
    }
}

impl Render for AnimatedText {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.set_reduced_motion(cx.environment().reduced_motion);
        let themes = shadcn(cx.environment());
        self.build(themes.resolve(cx.environment().color_scheme))
    }

    fn animation_frame(&mut self, frame: Frame, cx: &mut Context<Self>) {
        if std::mem::take(&mut self.first_frame) {
            return;
        }
        if self.advance(frame.elapsed) {
            cx.notify();
        }
    }

    fn wants_animation_frame(&self) -> bool {
        self.is_animating()
    }
}
