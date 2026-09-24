use argui_core::{Point, Size};
use argui_media::{PathCommand, PathError, PathStyle, VectorLibrary, path_asset};
use argui_paint::VectorId;

#[test]
fn filled_path_preserves_geometry_as_tintable_vector() {
    let commands = [
        PathCommand::MoveTo(Point::new(1.0, 1.0)),
        PathCommand::LineTo(Point::new(19.0, 1.0)),
        PathCommand::QuadraticTo {
            control: Point::new(19.0, 10.0),
            end: Point::new(10.0, 19.0),
        },
        PathCommand::CubicTo {
            first_control: Point::new(5.0, 19.0),
            second_control: Point::new(1.0, 15.0),
            end: Point::new(1.0, 1.0),
        },
        PathCommand::Close,
    ];
    let style = PathStyle {
        even_odd: true,
        ..PathStyle::filled()
    };
    let asset = path_asset(VectorId(7), Size::new(20.0, 20.0), &commands, style).unwrap();
    let svg = std::str::from_utf8(&asset.svg).unwrap();
    assert_eq!(asset.id, VectorId(7));
    assert_eq!(asset.size, Size::new(20.0, 20.0));
    assert!(asset.tintable);
    assert!(svg.contains("M1 1L19 1Q19 10 10 19C5 19 1 15 1 1Z"));
    assert!(svg.contains("fill-rule=\"evenodd\""));
}

#[test]
fn stroked_path_supports_multiple_open_subpaths() {
    let commands = [
        PathCommand::MoveTo(Point::new(0.0, 0.0)),
        PathCommand::LineTo(Point::new(5.0, 5.0)),
        PathCommand::MoveTo(Point::new(5.0, 0.0)),
        PathCommand::LineTo(Point::new(0.0, 5.0)),
    ];
    let asset = path_asset(
        VectorId(8),
        Size::new(5.0, 5.0),
        &commands,
        PathStyle::stroked(2.0),
    )
    .unwrap();
    let svg = std::str::from_utf8(&asset.svg).unwrap();
    assert!(svg.contains("fill=\"none\""));
    assert!(svg.contains("stroke=\"currentColor\" stroke-width=\"2\""));
}

#[test]
fn invalid_geometry_and_style_are_rejected() {
    let move_to = PathCommand::MoveTo(Point::new(0.0, 0.0));
    let line_to = PathCommand::LineTo(Point::new(1.0, 1.0));
    let size = Size::new(10.0, 10.0);
    let make =
        |size, commands: &[PathCommand], style| path_asset(VectorId(9), size, commands, style);

    assert!(matches!(
        make(
            Size::new(0.0, 10.0),
            &[move_to, line_to],
            PathStyle::filled()
        ),
        Err(PathError::InvalidSize)
    ));
    for bad_size in [
        Size::new(10.0, 0.0),
        Size::new(f32::NAN, 10.0),
        Size::new(10.0, f32::INFINITY),
    ] {
        assert!(matches!(
            make(bad_size, &[move_to, line_to], PathStyle::filled()),
            Err(PathError::InvalidSize)
        ));
    }
    assert!(matches!(
        make(size, &[move_to], PathStyle::filled()),
        Err(PathError::Empty)
    ));
    assert!(matches!(
        make(size, &[line_to], PathStyle::filled()),
        Err(PathError::InvalidCommand(0))
    ));
    assert!(matches!(
        make(
            size,
            &[move_to, PathCommand::Close, line_to],
            PathStyle::filled()
        ),
        Err(PathError::InvalidCommand(2))
    ));
    assert!(matches!(
        make(
            size,
            &[move_to, PathCommand::LineTo(Point::new(f32::NAN, 1.0))],
            PathStyle::filled()
        ),
        Err(PathError::InvalidCoordinate(1))
    ));
    for bad_command in [
        PathCommand::MoveTo(Point::new(0.0, f32::INFINITY)),
        PathCommand::QuadraticTo {
            control: Point::new(f32::NAN, 1.0),
            end: Point::new(1.0, 1.0),
        },
        PathCommand::CubicTo {
            first_control: Point::new(1.0, 1.0),
            second_control: Point::new(2.0, 2.0),
            end: Point::new(3.0, f32::NEG_INFINITY),
        },
    ] {
        let commands = if matches!(bad_command, PathCommand::MoveTo(_)) {
            vec![bad_command, line_to]
        } else {
            vec![move_to, bad_command]
        };
        assert!(matches!(
            make(size, &commands, PathStyle::filled()),
            Err(PathError::InvalidCoordinate(_))
        ));
    }
    for width in [-1.0, 0.0, f32::INFINITY, f32::NAN] {
        assert!(matches!(
            make(size, &[move_to, line_to], PathStyle::stroked(width)),
            Err(PathError::InvalidStroke)
        ));
    }
    assert!(matches!(
        make(
            size,
            &[move_to, line_to],
            PathStyle {
                fill: false,
                stroke_width: None,
                even_odd: false
            }
        ),
        Err(PathError::EmptyStyle)
    ));
}

#[test]
fn library_registers_only_valid_paths() {
    let mut library = VectorLibrary::new();
    let commands = [
        PathCommand::MoveTo(Point::new(0.0, 0.0)),
        PathCommand::LineTo(Point::new(4.0, 4.0)),
    ];
    assert!(
        library
            .insert_path(Size::new(0.0, 4.0), &commands, PathStyle::filled())
            .is_err()
    );
    assert!(library.assets().is_empty());

    let id = library
        .insert_path(Size::new(4.0, 4.0), &commands, PathStyle::stroked(1.0))
        .unwrap();
    assert_eq!(library.assets().len(), 1);
    assert_eq!(library.assets()[0].id, id);
}
