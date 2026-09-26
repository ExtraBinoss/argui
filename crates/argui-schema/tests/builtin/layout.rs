use argui_core::{Color, Size, Transform2D};
use argui_paint::{Border, BorderWidths, CornerRadii, Shadow};
use argui_schema::{ContainerRule, ContainerRuleStyle, NativeElementInput, SchemaValue, builtin};
use argui_ui::{
    ContainerQuery, ContainerScopeId, GridPlacement, GridTemplateComponent, LayoutInsets,
    LengthPercentageAuto, Line, TreeUpdate, UiTree, WritingDirection, auto, fr, length, minmax,
};

#[test]
fn direction_scope_is_exposed_to_all_native_layout_containers() {
    let registry = builtin::registry().unwrap();
    for native_type in [
        builtin::CONTAINER,
        builtin::ROW,
        builtin::COLUMN,
        builtin::GRID,
    ] {
        let element = registry
            .construct(
                native_type,
                &NativeElementInput::new()
                    .property(builtin::DIRECTION_SCOPE, SchemaValue::String("rtl".into())),
            )
            .unwrap();
        assert_eq!(element.direction_scope, Some(WritingDirection::Rtl));
    }
    assert!(
        registry
            .construct(
                builtin::ROW,
                &NativeElementInput::new().property(
                    builtin::DIRECTION_SCOPE,
                    SchemaValue::String("sideways".into()),
                ),
            )
            .is_err()
    );
}

#[test]
fn grid_tracks_placement_and_common_layout_reach_the_element() {
    let registry = builtin::registry().unwrap();
    let grid = registry
        .construct(
            builtin::GRID,
            &NativeElementInput::new()
                .property(
                    builtin::GRID_COLUMNS,
                    SchemaValue::GridTracks(vec![
                        GridTemplateComponent::Single(minmax(length(120.0), fr(1.0))),
                        GridTemplateComponent::Single(fr(2.0)),
                    ]),
                )
                .property(
                    builtin::GRID_ROWS,
                    SchemaValue::GridTracks(vec![
                        GridTemplateComponent::Single(auto()),
                        GridTemplateComponent::Single(length(36.0)),
                    ]),
                )
                .property(
                    builtin::MAX_WIDTH,
                    SchemaValue::Constraint(LengthPercentageAuto::length(800.0)),
                )
                .property(builtin::ASPECT_RATIO, SchemaValue::Float(1.5))
                .property(
                    builtin::PADDING,
                    SchemaValue::Insets(LayoutInsets {
                        left: 14.0,
                        ..LayoutInsets::default()
                    }),
                )
                .property(
                    builtin::MARGIN,
                    SchemaValue::Insets(LayoutInsets {
                        top: 7.0,
                        ..LayoutInsets::default()
                    }),
                )
                .property(builtin::COLUMN_GAP, SchemaValue::Float(12.0))
                .property(
                    builtin::TRANSFORM,
                    SchemaValue::Transform(
                        Transform2D::IDENTITY.translate(0.0, 9.0).scale(1.25, 1.0),
                    ),
                ),
        )
        .unwrap();
    assert_eq!(grid.style.grid_template_columns.len(), 2);
    assert_eq!(grid.style.grid_template_rows.len(), 2);
    assert_eq!(
        grid.style.max_size.width,
        LengthPercentageAuto::length(800.0)
    );
    assert_eq!(grid.style.aspect_ratio, Some(1.5));
    assert_eq!(grid.style.padding.left, length(14.0));
    assert_eq!(grid.style.margin.top, length(7.0));
    assert_eq!(grid.style.gap.width, length(12.0));
    assert_eq!(grid.transform.scale.x, 1.25);
    assert_eq!(grid.transform.translation.y, 9.0);
    assert!(matches!(
        grid.style.grid_template_columns[0],
        GridTemplateComponent::Single(_)
    ));
}

#[test]
fn independent_layout_limits_and_basis_reach_the_element() {
    let panel = builtin::registry()
        .unwrap()
        .construct(
            builtin::CONTAINER,
            &NativeElementInput::new()
                .property(
                    builtin::MAX_HEIGHT,
                    SchemaValue::Constraint(LengthPercentageAuto::length(240.0)),
                )
                .property(builtin::FLEX_BASIS, SchemaValue::Dimension(length(40.0)))
                .property(builtin::ROW_GAP, SchemaValue::Float(11.0))
                .property(builtin::SHRINK, SchemaValue::Float(0.0)),
        )
        .unwrap();
    assert_eq!(
        panel.style.max_size.height,
        LengthPercentageAuto::length(240.0)
    );
    assert_eq!(panel.style.flex_basis, length(40.0));
    assert_eq!(panel.style.gap.height, length(11.0));
    assert_eq!(panel.style.flex_shrink, 0.0);
}

#[test]
fn min_max_constraints_accept_percent_and_auto_and_reject_negative_values() {
    let registry = builtin::registry().unwrap();
    let element = registry
        .construct(
            builtin::ROW,
            &NativeElementInput::new()
                .property(
                    builtin::MIN_WIDTH,
                    SchemaValue::Constraint(LengthPercentageAuto::percent(0.5)),
                )
                .property(
                    builtin::MAX_WIDTH,
                    SchemaValue::Constraint(LengthPercentageAuto::auto()),
                ),
        )
        .unwrap();
    assert_eq!(
        element.style.min_size.width,
        LengthPercentageAuto::percent(0.5)
    );
    assert_eq!(element.style.max_size.width, LengthPercentageAuto::auto());
    assert!(
        registry
            .construct(
                builtin::ROW,
                &NativeElementInput::new().property(
                    builtin::MIN_WIDTH,
                    SchemaValue::Constraint(LengthPercentageAuto::length(-1.0)),
                ),
            )
            .is_err()
    );
}

#[test]
fn grid_child_placement_is_explicit_and_checked() {
    let registry = builtin::registry().unwrap();
    let cell = registry
        .construct(
            builtin::CONTAINER,
            &NativeElementInput::new()
                .property(builtin::GRID_ROW_START, SchemaValue::Int(2))
                .property(builtin::GRID_ROW_SPAN, SchemaValue::Int(3))
                .property(builtin::GRID_COLUMN_START, SchemaValue::Int(1)),
        )
        .unwrap();
    assert_eq!(
        cell.style.grid_row,
        Line {
            start: argui_ui::line(2),
            end: GridPlacement::Span(3)
        }
    );
    assert_eq!(cell.style.grid_column.start, argui_ui::line(1));
    for value in [0, i64::MAX] {
        assert!(
            registry
                .construct(
                    builtin::CONTAINER,
                    &NativeElementInput::new()
                        .property(builtin::GRID_ROW_SPAN, SchemaValue::Int(value))
                )
                .is_err()
        );
    }
    assert!(
        registry
            .construct(
                builtin::GRID,
                &NativeElementInput::new()
                    .property(builtin::GRID_COLUMNS, SchemaValue::String("bogus".into()))
            )
            .is_err()
    );
}

#[test]
fn container_breakpoint_registers_a_layout_query() {
    let registry = builtin::registry().unwrap();
    let grid = registry
        .construct(
            builtin::GRID,
            &NativeElementInput::new()
                .property(
                    builtin::QUERY_SCOPE,
                    SchemaValue::String("dashboard".into()),
                )
                .property(
                    builtin::CONTAINER_RULES,
                    SchemaValue::ContainerRules(vec![ContainerRule {
                        conditions: vec![ContainerQuery::min_width(
                            ContainerScopeId::from_owned("dashboard".into()),
                            520.0,
                        )],
                        style: ContainerRuleStyle {
                            grid_columns: Some(vec![
                                GridTemplateComponent::Single(fr(1.0)),
                                GridTemplateComponent::Single(fr(2.0)),
                            ]),
                            ..ContainerRuleStyle::default()
                        },
                    }]),
                ),
        )
        .unwrap();
    let mut tree = UiTree::new(grid);
    assert_eq!(
        tree.resolve_container_queries(&[Size::new(600.0, 400.0)]),
        TreeUpdate::Layout
    );
    assert_eq!(
        tree.resolve_container_queries(&[Size::new(600.0, 400.0)]),
        TreeUpdate::None
    );
}

#[test]
fn independent_edges_and_clip_reach_paint_and_layout() {
    let registry = builtin::registry().unwrap();
    let panel = registry
        .construct(
            builtin::CONTAINER,
            &NativeElementInput::new()
                .property(
                    builtin::RADII,
                    SchemaValue::Radii(CornerRadii {
                        top_left: 12.0,
                        top_right: 4.0,
                        bottom_right: 4.0,
                        bottom_left: 4.0,
                    }),
                )
                .property(
                    builtin::BORDER,
                    SchemaValue::Border(Border {
                        widths: BorderWidths {
                            left: 3.0,
                            top: 2.0,
                            ..BorderWidths::default()
                        },
                        color: Color::BLACK,
                    }),
                )
                .property(builtin::CLIP, SchemaValue::Bool(true))
                .property(
                    builtin::SHADOW,
                    SchemaValue::Shadow(Shadow::drop([5.0, 0.0], 8.0, Color::BLACK).spread(2.0)),
                )
                .property(builtin::ALIGN_SELF, SchemaValue::String("center".into())),
        )
        .unwrap();
    assert_eq!(panel.paint.quad.radii.top_left, 12.0);
    assert_eq!(panel.paint.quad.radii.top_right, 4.0);
    assert_eq!(panel.paint.quad.border.unwrap().widths.left, 3.0);
    assert_eq!(panel.paint.quad.border.unwrap().widths.top, 2.0);
    assert_eq!(panel.style.overflow.x, argui_ui::Overflow::Hidden);
    assert_eq!(panel.style.align_self, Some(argui_ui::AlignSelf::CENTER));
    let shadow = &panel.layer.as_ref().unwrap().shadows[0];
    assert_eq!(shadow.offset, [5.0, 0.0]);
    assert_eq!(shadow.spread, 2.0);
}

#[test]
fn positioning_and_standalone_rotation_reach_the_element() {
    let registry = builtin::registry().unwrap();
    let panel = registry
        .construct(
            builtin::CONTAINER,
            &NativeElementInput::new()
                .property(builtin::POSITION, SchemaValue::String("absolute".into()))
                .property(
                    builtin::INSET,
                    SchemaValue::PositionInsets(argui_ui::PositionInsets {
                        left: Some(12.0),
                        top: Some(8.0),
                        ..argui_ui::PositionInsets::default()
                    }),
                )
                .property(builtin::Z_INDEX, SchemaValue::Int(4))
                .property(builtin::ROTATION, SchemaValue::Float(30.0)),
        )
        .unwrap();
    assert_eq!(panel.style.position, argui_ui::Position::Absolute);
    assert_eq!(panel.style.inset.left, length(12.0));
    assert_eq!(panel.style.inset.top, length(8.0));
    assert_eq!(panel.z_index, 4);
    assert_ne!(panel.transform, argui_core::Transform2D::IDENTITY);
    assert!(
        registry
            .construct(
                builtin::CONTAINER,
                &NativeElementInput::new().property(builtin::Z_INDEX, SchemaValue::Int(i64::MAX)),
            )
            .is_err()
    );
}
