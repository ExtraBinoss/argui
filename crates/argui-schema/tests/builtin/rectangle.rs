use argui_core::Color;
use argui_paint::{Fill, Filter};
use argui_schema::{NativeElementInput, NativeSlotValue, SchemaValue, builtin};
use argui_ui::{Element, ExpandedLengthPercentageAuto, Overflow, Position, length};

#[test]
fn rectangle_paints_brush_border_and_clips_rounded_children() {
    let registry = builtin::registry().unwrap();
    let background = Fill::Solid(Color::srgba(0.2, 0.4, 0.6, 1.0));
    let rectangle = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new()
                .property(builtin::BACKGROUND, SchemaValue::Brush(background.clone()))
                .property(builtin::BORDER_WIDTH, SchemaValue::Float(2.0))
                .property(builtin::BORDER_COLOR, SchemaValue::Color(Color::BLACK))
                .property(builtin::RADIUS, SchemaValue::Float(8.0))
                .property(builtin::CLIP, SchemaValue::Bool(true))
                .property(
                    builtin::BACKDROP_FILTER,
                    SchemaValue::String("blur(8px)".into()),
                )
                .property(builtin::X, SchemaValue::Dimension(length(12.0)))
                .property(builtin::Y, SchemaValue::Dimension(length(4.0)))
                .slot(NativeSlotValue::new(
                    builtin::CHILDREN,
                    [Element::text("child")],
                )),
        )
        .unwrap();

    assert_eq!(rectangle.paint.quad.background, Some(background));
    assert_eq!(rectangle.paint.quad.border.unwrap().widths.left, 2.0);
    assert_eq!(rectangle.paint.quad.radii.top_left, 8.0);
    assert_eq!(rectangle.style.overflow.x, Overflow::Hidden);
    assert_eq!(rectangle.style.overflow.y, Overflow::Hidden);
    assert_eq!(rectangle.children.len(), 1);
    assert_eq!(
        rectangle.layer.as_ref().unwrap().backdrop_filters,
        vec![Filter::Blur(8.0)]
    );
    assert_eq!(rectangle.style.position, Position::Absolute);
    assert_eq!(
        rectangle.style.inset.left.expand(),
        ExpandedLengthPercentageAuto::Length(12.0)
    );
    assert_eq!(
        rectangle.style.inset.top.expand(),
        ExpandedLengthPercentageAuto::Length(4.0)
    );
}

#[test]
fn rectangle_rejects_invalid_border_width_and_radius() {
    let registry = builtin::registry().unwrap();
    for (property, value) in [(builtin::BORDER_WIDTH, f32::NAN), (builtin::RADIUS, -1.0)] {
        let error = registry
            .construct(
                builtin::RECTANGLE,
                &NativeElementInput::new().property(property, SchemaValue::Float(value)),
            )
            .unwrap_err();
        assert!(error.to_string().contains("finite nonnegative"));
    }
}

#[test]
fn rectangle_rejects_invalid_backdrop_filter() {
    let registry = builtin::registry().unwrap();
    let error = registry
        .construct(
            builtin::RECTANGLE,
            &NativeElementInput::new().property(
                builtin::BACKDROP_FILTER,
                SchemaValue::String("blur(-2px)".into()),
            ),
        )
        .unwrap_err();
    assert!(error.to_string().contains("invalid backdrop_filter"));
}
