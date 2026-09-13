#![cfg(feature = "animated-text")]

use argui_animation::{Duration, Frame, Time};
use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_runtime::{Context, Entity, Render, WindowEnvironment};
use argui_text::{FontFamily, TextEngine, TextStyle};
use argui_ui::{Element, ElementKind, TreeUpdate, UiTree};
use argui_widgets::{AnimatedText, TextAnimation, shadcn};

fn texts(element: &Element) -> Vec<String> {
    let mut values = Vec::new();
    if let ElementKind::Text { content, .. } = &element.kind {
        values.push(content.as_str().into());
    }
    values.extend(element.children.iter().flat_map(texts));
    values
}

#[test]
fn only_changed_digits_roll_and_intermediate_frames_reuse_layout() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Dark);
    let mut label = AnimatedText::new("count", "10").font_size(32.0);
    assert!(!label.is_animating());
    label.set_text("10");
    assert!(!label.is_animating());
    label.set_text("11");
    let start = label.build(theme);
    assert_eq!(texts(&start.children[0]), ["1"]);
    assert_eq!(texts(&start.children[1]), ["0", "1"]);
    let mut tree = UiTree::new(start.clone());
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::new();
    let before = engine
        .compute(&mut tree, &mut text, Size::new(300.0, 100.0))
        .unwrap();
    let width = before.nodes[0].bounds.size.width;
    let semantic = tree.semantic_tree(&before.semantic_bounds, 1.0);
    assert_eq!(semantic.nodes.len(), 1);
    assert_eq!(semantic.nodes[0].semantics.label.as_deref(), Some("11"));
    assert!(!label.advance(Duration::ZERO));
    label.advance(Duration::from_millis(150));
    let middle = label.build(theme);
    assert!(middle.children[0].ptr_eq(&start.children[0]));
    assert!(middle.children[1].children[0].transform.translation.y < 0.0);
    assert_eq!(
        tree.update(middle),
        TreeUpdate::Paint,
        "Animation must not invalidate layout"
    );
    label.advance(Duration::from_millis(300));
    assert!(!label.is_animating());
    tree.update(label.build(theme));
    let after = engine
        .compute(&mut tree, &mut text, Size::new(300.0, 100.0))
        .unwrap();
    assert_eq!(after.nodes[0].bounds.size.width, width);
    assert!(!label.advance(Duration::from_secs(10)));
}

#[test]
fn carries_reverse_rolls_unicode_and_queued_updates_finish_without_stale_text() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Light);
    let mut label = AnimatedText::new("count", "99");
    label.set_text("100");
    assert_eq!(label.build(theme).children.len(), 3);
    label.set_text("101");
    label.set_text("105");
    label.advance(Duration::from_secs(1));
    assert!(label.is_animating());
    assert_eq!(
        texts(&label.build(theme).children[2]),
        ["0", "1", "2", "3", "4", "5"]
    );
    label.advance(Duration::from_secs(1));
    assert_eq!(texts(&label.build(theme)).join(""), "105");
    label.set_text("104");
    let down = label.build(theme);
    assert!(down.children[2].children[0].transform.translation.y < 0.0);
    label.advance(Duration::from_secs(1));
    label.set_text("");
    label.advance(Duration::from_secs(1));
    assert!(label.build(theme).children.is_empty());
    let mut unicode = AnimatedText::new("status", "e\u{301}🌍").align_end(false);
    unicode.set_text("e\u{301}🌟");
    let view = unicode.build(theme);
    assert_eq!(view.children.len(), 2);
    assert_eq!(texts(&view.children[0]), ["e\u{301}"]);
    assert_eq!(texts(&view.children[1]), ["🌍", "🌟"]);
}

#[test]
fn slide_fade_and_reduced_motion_respect_timing_and_stop_at_rest() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Dark);
    for animation in [TextAnimation::Slide, TextAnimation::Fade] {
        let mut label = AnimatedText::new("value", "1")
            .animation(animation)
            .duration(Duration::from_millis(400))
            .text_style(TextStyle {
                font_size: 24.0,
                line_height: 30.0,
                family: FontFamily::Monospace,
                ..TextStyle::default()
            });
        label.set_text("9");
        assert_eq!(texts(&label.build(theme)), ["1", "9"]);
        label.advance(Duration::from_millis(200));
        let middle = label.build(theme);
        if animation == TextAnimation::Fade {
            let reel = &middle.children[0].children[0];
            let alphas = reel
                .children
                .iter()
                .map(|glyph| {
                    let ElementKind::Text { style, .. } = &glyph.kind else {
                        panic!("text")
                    };
                    assert!(glyph.layer.is_none());
                    style.color.to_linear_rgba()[3]
                })
                .collect::<Vec<_>>();
            assert_eq!(alphas, [0.125, 0.875]);
            assert_eq!(reel.children[1].transform.translation.y, -30.0);
        }
        label.set_reduced_motion(true);
        assert!(!label.is_animating());
        assert_eq!(texts(&label.build(theme)), ["9"]);
        label.set_text("12");
        assert_eq!(texts(&label.build(theme)).join(""), "12");
    }
    let mut immediate = AnimatedText::new("now", "0").duration(Duration::ZERO);
    immediate.set_text("1");
    assert!(!immediate.is_animating());
    let entity = Entity::new(AnimatedText::new("mounted", "0"));
    entity.update(|label, _| label.set_text("1"));
    let _ = entity.render_in(WindowEnvironment {
        reduced_motion: true,
        ..Default::default()
    });
    entity.read(|label| assert!(!label.wants_animation_frame()));
}

#[test]
fn changing_character_widths_converge_before_transition_cleanup() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Dark);
    let font = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
    let mut text =
        TextEngine::from_embedded_fonts([font.as_slice()], "Noto Sans", "Noto Sans", "Noto Sans");
    for animation in [
        TextAnimation::Roll,
        TextAnimation::Slide,
        TextAnimation::Fade,
    ] {
        for (from, to) in [
            ("100", "99"),
            ("99", "100"),
            ("Saved", "Draft"),
            ("Draft", "Saved"),
            ("", "1"),
            ("1", ""),
        ] {
            let mut label = AnimatedText::new("value", from)
                .font_size(44.0)
                .animation(animation);
            let mut engine = LayoutEngine::new();
            let mut tree = UiTree::new(Element::row([label.build(theme)]));
            let mut bounds = |label: &mut AnimatedText| {
                tree.update(Element::row([label.build(theme)]));
                engine
                    .compute(&mut tree, &mut text, Size::new(400.0, 100.0))
                    .unwrap()
                    .nodes[1]
                    .bounds
            };
            let before = bounds(&mut label);
            label.set_text(to);
            let start = bounds(&mut label);
            assert_eq!(
                start.size.width, before.size.width,
                "{animation:?}: {from} → {to} starts at the displayed width"
            );
            label.advance(Duration::from_millis(70));
            let middle = bounds(&mut label);
            label.advance(Duration::from_millis(349));
            let near_end = bounds(&mut label);
            label.advance(Duration::from_millis(1));
            let end = bounds(&mut label);
            assert!(
                (near_end.size.width - end.size.width).abs() <= 1.0,
                "{animation:?}: {from} → {to} must not jump on cleanup: {near_end:?} → {end:?}"
            );
            if (end.size.width - before.size.width).abs() > 4.0 {
                assert!(middle.size.width > before.size.width.min(end.size.width));
                assert!(middle.size.width < before.size.width.max(end.size.width));
            }
        }
    }
}

#[test]
fn an_idle_window_does_not_skip_the_first_animation_frame() {
    let mut label = AnimatedText::new("count", "10");
    let mut cx = Context::default();
    label.render(&mut cx);
    label.set_text("11");
    let frame = Frame {
        now: Time::from_nanos(10_000_000_000),
        elapsed: Duration::from_secs(10),
    };
    label.animation_frame(frame, &mut cx);
    assert!(label.wants_animation_frame());
    label.animation_frame(
        Frame {
            elapsed: Duration::from_millis(500),
            ..frame
        },
        &mut cx,
    );
    assert!(!label.wants_animation_frame());
    label.animation_frame(frame, &mut cx);
    assert_eq!(label.value(), "11");
}

#[test]
fn a_status_fade_reverses_immediately_without_an_opacity_jump_or_queued_transition() {
    fn alphas(element: &Element) -> Vec<(String, f32)> {
        let mut result = match &element.kind {
            ElementKind::Text { content, style } => {
                vec![(content.as_str().into(), style.color.to_linear_rgba()[3])]
            }
            _ => Vec::new(),
        };
        result.extend(element.children.iter().flat_map(alphas));
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Dark);
    let mut label = AnimatedText::new("status", "Draft")
        .animation(TextAnimation::Fade)
        .align_end(false);
    label.set_text("Saved");
    label.advance(Duration::from_millis(100));
    let before = alphas(&label.build(theme));
    label.set_text("Draft");
    let reversed = alphas(&label.build(theme));
    for ((before_text, before_alpha), (after_text, after_alpha)) in before.iter().zip(&reversed) {
        assert_eq!(before_text, after_text);
        assert!((before_alpha - after_alpha).abs() < 0.00001);
    }
    label.advance(Duration::from_millis(200));
    assert!(
        alphas(&label.build(theme))
            .iter()
            .find(|(text, _)| text == "D")
            .unwrap()
            .1
            > reversed.iter().find(|(text, _)| text == "D").unwrap().1
    );
    label.advance(Duration::from_millis(220));
    assert!(!label.is_animating());
    assert_eq!(texts(&label.build(theme)).join(""), "Draft");
    label.set_text("Saved");
    label.set_text("Draft");
    assert!(
        !label.is_animating(),
        "a reversal before the first frame cancels immediately"
    );
}
