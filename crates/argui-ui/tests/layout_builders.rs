use argui_text::{TextAlign, TextOverflow};
use argui_ui::{
    AlignContent, AlignItems, AlignSelf, BoxSizing, Dimension, Dimensions, Display, Element,
    ElementKind, FlexDirection, FlexWrap, GridAutoFlow, GridPlacement, GridTemplateArea,
    GridTemplateAreas, GridTemplateComponent, JustifyContent, JustifyItems, JustifySelf,
    LayoutStyle, LengthPercentage, LengthPercentageAuto, Line, Overflow, Position, ScrollbarGutter,
    TextEditorSpec, TextInputFilter, TrackSizingFunction, fr, length, sides,
};

#[test]
fn grid_builders_store_templates_tracks_areas_and_placements() {
    let rows = vec![GridTemplateComponent::Single(fr(1.0))];
    let columns = vec![GridTemplateComponent::Single(fr(2.0))];
    let auto_rows = vec![TrackSizingFunction::from(LengthPercentage::length(24.0))];
    let auto_columns = vec![TrackSizingFunction::from(LengthPercentage::percent(0.5))];
    let areas = GridTemplateAreas {
        areas: vec![GridTemplateArea {
            name: "content".to_owned(),
            row_start: 0,
            row_end: 1,
            column_start: 0,
            column_end: 1,
        }],
        row_count: 1,
        column_count: 1,
    };
    let row = Line {
        start: GridPlacement::NamedLine("top".to_owned(), 1),
        end: GridPlacement::Span(2),
    };
    let column = Line {
        start: GridPlacement::Span(1),
        end: GridPlacement::NamedLine("end".to_owned(), 1),
    };

    let element = Element::grid([])
        .grid_template_rows(rows.clone())
        .grid_template_columns(columns.clone())
        .grid_auto_rows(auto_rows.clone())
        .grid_auto_columns(auto_columns.clone())
        .grid_auto_flow(GridAutoFlow::ColumnDense)
        .grid_template_areas(areas.clone())
        .grid_row(row.clone())
        .grid_column(column.clone());

    assert_eq!(element.style.grid_template_rows, rows);
    assert_eq!(element.style.grid_template_columns, columns);
    assert_eq!(element.style.grid_auto_rows, auto_rows);
    assert_eq!(element.style.grid_auto_columns, auto_columns);
    assert_eq!(element.style.grid_auto_flow, GridAutoFlow::ColumnDense);
    assert_eq!(element.style.grid_template_areas, Some(areas));
    assert_eq!(element.style.grid_row, row);
    assert_eq!(element.style.grid_column, column);
}

#[test]
fn layout_builders_cover_optional_alignment_and_position_fields() {
    let layout = Element::container([])
        .display(Display::Flex)
        .box_sizing(BoxSizing::ContentBox)
        .writing_direction(argui_ui::WritingDirection::Rtl)
        .position(Position::Sticky)
        .margin(sides(3.0, 5.0))
        .padding(sides(7.0, 11.0))
        .row_gap(13.0)
        .column_gap(17.0)
        .align_items(AlignItems::END)
        .align_self(AlignSelf::FLEX_END)
        .justify_items(JustifyItems::END)
        .justify_self(JustifySelf::END)
        .align_content(AlignContent::SPACE_AROUND)
        .justify_content(JustifyContent::SPACE_EVENLY)
        .flex_direction(FlexDirection::ColumnReverse)
        .flex_wrap(FlexWrap::WrapReverse)
        .grow(2.0)
        .shrink(0.25)
        .aspect_ratio(1.5);

    assert_eq!(layout.style.display, Display::Flex);
    assert_eq!(layout.style.box_sizing, BoxSizing::ContentBox);
    assert_eq!(
        layout.style.writing_direction,
        argui_ui::WritingDirection::Rtl
    );
    assert_eq!(layout.style.position, Position::Sticky);
    assert_eq!(layout.style.margin, sides(3.0, 5.0));
    assert_eq!(layout.style.padding, sides(7.0, 11.0));
    assert_eq!(layout.style.gap.width, length(17.0));
    assert_eq!(layout.style.gap.height, length(13.0));
    assert_eq!(layout.style.align_items, Some(AlignItems::END));
    assert_eq!(layout.style.align_self, Some(AlignSelf::FLEX_END));
    assert_eq!(layout.style.justify_items, Some(JustifyItems::END));
    assert_eq!(layout.style.justify_self, Some(JustifySelf::END));
    assert_eq!(layout.style.align_content, Some(AlignContent::SPACE_AROUND));
    assert_eq!(
        layout.style.justify_content,
        Some(JustifyContent::SPACE_EVENLY)
    );
    assert_eq!(layout.style.flex_direction, FlexDirection::ColumnReverse);
    assert_eq!(layout.style.flex_wrap, FlexWrap::WrapReverse);
    assert_eq!(layout.style.flex_grow, 2.0);
    assert_eq!(layout.style.flex_shrink, 0.25);
    assert_eq!(layout.style.aspect_ratio, Some(1.5));
}

#[test]
fn textual_builders_update_text_and_editor_styles_but_ignore_other_kinds() {
    let overflow = TextOverflow::Ellipsis(Default::default());
    let text = Element::text("label")
        .text_align(TextAlign::Right)
        .text_overflow(overflow);
    let ElementKind::Text { style, .. } = &text.kind else {
        panic!("expected text element");
    };
    assert_eq!(style.align, TextAlign::Right);
    assert_eq!(style.overflow, overflow);

    let editor = Element::text_editor(TextEditorSpec {
        value: "value".to_owned(),
        placeholder: String::new(),
        multiline: false,
        read_only: false,
        filter: TextInputFilter::Any,
        text: argui_text::TextStyle::default(),
        placeholder_text: argui_text::TextStyle::default(),
        selection: argui_ui::Color::TRANSPARENT,
        caret: argui_ui::CaretStyle::default(),
    })
    .text_align(TextAlign::Center)
    .text_overflow(overflow);
    let ElementKind::TextEditor {
        text,
        placeholder_text,
        ..
    } = &editor.kind
    else {
        panic!("expected text editor");
    };
    assert_eq!(text.align, TextAlign::Center);
    assert_eq!(placeholder_text.align, TextAlign::Center);
    assert_eq!(text.overflow, overflow);
    assert_eq!(placeholder_text.overflow, overflow);

    for element in [Element::container([]), Element::image(argui_ui::ImageId(1))] {
        let before = element.clone().layout_style(LayoutStyle::default());
        let after = element
            .text_align(TextAlign::Justify)
            .text_overflow(TextOverflow::Clip);
        assert_eq!(after.style, before.style);
    }
}

#[test]
fn invalid_scrollbar_width_is_clamped_and_size_builders_preserve_axes() {
    let size = Dimensions {
        width: Dimension::length(320.0),
        height: Dimension::percent(0.5),
    };
    let min = Dimensions {
        width: LengthPercentageAuto::length(20.0),
        height: LengthPercentageAuto::auto(),
    };
    let max = Dimensions {
        width: LengthPercentageAuto::length(640.0),
        height: LengthPercentageAuto::length(480.0),
    };
    let element = Element::container([])
        .overflow(argui_ui::Axes {
            x: Overflow::Clip,
            y: Overflow::Scroll,
        })
        .scrollbar_gutter(ScrollbarGutter::Stable)
        .scrollbar_width(f32::NAN)
        .inset(sides(4.0, 8.0))
        .size(size)
        .min_size(min)
        .max_size(max)
        .flex_basis(Dimension::length(100.0));

    assert_eq!(element.style.overflow.x, Overflow::Clip);
    assert_eq!(element.style.overflow.y, Overflow::Scroll);
    assert_eq!(element.style.scrollbar_gutter, ScrollbarGutter::Stable);
    assert_eq!(element.style.scrollbar_width, 0.0);
    assert_eq!(element.style.inset, sides(4.0, 8.0));
    assert_eq!(element.style.size, size);
    assert_eq!(element.style.min_size, min);
    assert_eq!(element.style.max_size, max);
    assert_eq!(element.style.flex_basis, Dimension::length(100.0));
}
