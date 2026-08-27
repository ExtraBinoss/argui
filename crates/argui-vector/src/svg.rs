use argui_core::{Color, Size};
use argui_paint::{VectorAsset, VectorId, VectorVertex};
use lyon_path::{Path, math::point};
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillRule, FillTessellator, FillVertex, LineCap, LineJoin,
    StrokeOptions, StrokeTessellator, StrokeVertex, VertexBuffers,
};
use usvg::{Node, Paint, tiny_skia_path::PathSegment};

#[derive(Debug, thiserror::Error)]
pub enum VectorError {
    #[error("SVG parsing failed: {0}")]
    Svg(#[from] usvg::Error),
    #[error("SVG paint type is not supported by the vector pipeline yet")]
    UnsupportedPaint,
    #[error("SVG images and text are not vector path primitives")]
    UnsupportedNode,
    #[error("SVG feature is not supported by the vector pipeline yet: {0}")]
    UnsupportedFeature(&'static str),
    #[error("path tessellation failed")]
    Tessellation,
    #[error("morph paths need corresponding commands and an affine-compatible mapping")]
    IncompatibleMorph,
}

pub fn parse_svg(id: VectorId, svg: &[u8]) -> Result<VectorAsset, VectorError> {
    let tree = usvg::Tree::from_data(svg, &usvg::Options::default())?;
    let size = Size::new(tree.size().width(), tree.size().height());
    let mut geometry = VertexBuffers::<VectorVertex, u32>::new();
    visit(tree.root(), &mut geometry, 1.0)?;
    if geometry.indices.is_empty() {
        return Err(VectorError::UnsupportedNode);
    }
    Ok(VectorAsset {
        id,
        size,
        vertices: geometry.vertices.into(),
        indices: geometry.indices.into(),
    })
}

pub fn morph_svg(id: VectorId, from: &[u8], to: &[u8]) -> Result<VectorAsset, VectorError> {
    let mut first = parse_svg(id, from)?;
    let second = parse_svg(id, to)?;
    if first.size != second.size {
        return Err(VectorError::IncompatibleMorph);
    }
    let from_tree = usvg::Tree::from_data(from, &usvg::Options::default())?;
    let to_tree = usvg::Tree::from_data(to, &usvg::Options::default())?;
    let transform = morph_transform(&from_tree, &to_tree)?;
    let target_color = uniform_color(&second)?;
    let mut vertices = first.vertices.to_vec();
    for vertex in &mut vertices {
        vertex.to = transform.apply(vertex.from);
        vertex.color_to = target_color;
    }
    first.vertices = vertices.into();
    Ok(first)
}

#[derive(Clone, Copy)]
struct MorphTransform {
    matrix: [f32; 4],
    translation: [f32; 2],
}

impl MorphTransform {
    fn apply(self, point: [f32; 2]) -> [f32; 2] {
        [
            self.matrix[0] * point[0] + self.matrix[2] * point[1] + self.translation[0],
            self.matrix[1] * point[0] + self.matrix[3] * point[1] + self.translation[1],
        ]
    }
}

fn morph_transform(from: &usvg::Tree, to: &usvg::Tree) -> Result<MorphTransform, VectorError> {
    let mut from_commands = Vec::new();
    let mut to_commands = Vec::new();
    collect_commands(from.root(), &mut from_commands)?;
    collect_commands(to.root(), &mut to_commands)?;
    if from_commands.len() != to_commands.len()
        || from_commands
            .iter()
            .zip(&to_commands)
            .any(|(from, to)| from.kind != to.kind)
    {
        return Err(VectorError::IncompatibleMorph);
    }
    let from_points = from_commands
        .iter()
        .filter_map(|command| command.point)
        .collect::<Vec<_>>();
    let to_points = to_commands
        .iter()
        .filter_map(|command| command.point)
        .collect::<Vec<_>>();
    let transform = affine_from_points(&from_points, &to_points)?;
    if from_points.iter().zip(to_points).any(|(from, to)| {
        let mapped = transform.apply(*from);
        (mapped[0] - to[0]).abs() > 0.01 || (mapped[1] - to[1]).abs() > 0.01
    }) {
        return Err(VectorError::IncompatibleMorph);
    }
    Ok(transform)
}

#[derive(Clone, Copy)]
struct MorphCommand {
    kind: u8,
    point: Option<[f32; 2]>,
}

fn collect_commands(
    group: &usvg::Group,
    commands: &mut Vec<MorphCommand>,
) -> Result<(), VectorError> {
    for node in group.children() {
        match node {
            Node::Group(group) => collect_commands(group, commands)?,
            Node::Path(path) if path.is_visible() => {
                let transform = path.abs_transform();
                for segment in path.data().segments() {
                    let mut add = |kind, point| {
                        commands.push(MorphCommand {
                            kind,
                            point: Some(map_point(transform, point)),
                        });
                    };
                    match segment {
                        PathSegment::MoveTo(point) => add(0, point),
                        PathSegment::LineTo(point) => add(1, point),
                        PathSegment::QuadTo(control, point) => {
                            add(2, control);
                            add(3, point);
                        }
                        PathSegment::CubicTo(first, second, point) => {
                            add(4, first);
                            add(5, second);
                            add(6, point);
                        }
                        PathSegment::Close => commands.push(MorphCommand {
                            kind: 7,
                            point: None,
                        }),
                    }
                }
            }
            Node::Path(_) => {}
            Node::Image(_) | Node::Text(_) => return Err(VectorError::UnsupportedNode),
        }
    }
    Ok(())
}

fn affine_from_points(from: &[[f32; 2]], to: &[[f32; 2]]) -> Result<MorphTransform, VectorError> {
    if from.len() != to.len() || from.len() < 3 {
        return Err(VectorError::IncompatibleMorph);
    }
    let origin = from[0];
    for second in 1..from.len() - 1 {
        for third in second + 1..from.len() {
            let a = subtract(from[second], origin);
            let b = subtract(from[third], origin);
            let determinant = a[0] * b[1] - a[1] * b[0];
            if determinant.abs() <= 0.000_1 {
                continue;
            }
            let c = subtract(to[second], to[0]);
            let d = subtract(to[third], to[0]);
            let inverse = [
                b[1] / determinant,
                -a[1] / determinant,
                -b[0] / determinant,
                a[0] / determinant,
            ];
            let matrix = [
                c[0] * inverse[0] + d[0] * inverse[1],
                c[1] * inverse[0] + d[1] * inverse[1],
                c[0] * inverse[2] + d[0] * inverse[3],
                c[1] * inverse[2] + d[1] * inverse[3],
            ];
            return Ok(MorphTransform {
                matrix,
                translation: [
                    to[0][0] - matrix[0] * origin[0] - matrix[2] * origin[1],
                    to[0][1] - matrix[1] * origin[0] - matrix[3] * origin[1],
                ],
            });
        }
    }
    Err(VectorError::IncompatibleMorph)
}

fn subtract(left: [f32; 2], right: [f32; 2]) -> [f32; 2] {
    [left[0] - right[0], left[1] - right[1]]
}

fn uniform_color(asset: &VectorAsset) -> Result<Color, VectorError> {
    let Some(color) = asset.vertices.first().map(|vertex| vertex.color_from) else {
        return Err(VectorError::IncompatibleMorph);
    };
    asset
        .vertices
        .iter()
        .all(|vertex| vertex.color_from == color)
        .then_some(color)
        .ok_or(VectorError::IncompatibleMorph)
}

fn visit(
    group: &usvg::Group,
    geometry: &mut VertexBuffers<VectorVertex, u32>,
    inherited_opacity: f32,
) -> Result<(), VectorError> {
    if group.clip_path().is_some() || group.mask().is_some() || !group.filters().is_empty() {
        return Err(VectorError::UnsupportedFeature("group clip/mask/filter"));
    }
    let opacity = inherited_opacity * group.opacity().get();
    for node in group.children() {
        match node {
            Node::Group(group) => visit(group, geometry, opacity)?,
            Node::Path(path) if path.is_visible() => tessellate(path, geometry, opacity)?,
            Node::Path(_) => {}
            Node::Image(_) | Node::Text(_) => return Err(VectorError::UnsupportedNode),
        }
    }
    Ok(())
}

fn tessellate(
    source: &usvg::Path,
    geometry: &mut VertexBuffers<VectorVertex, u32>,
    inherited_opacity: f32,
) -> Result<(), VectorError> {
    let path = lyon_path(source);
    match source.paint_order() {
        usvg::PaintOrder::FillAndStroke => {
            tessellate_fill(source, &path, geometry, inherited_opacity)?;
            tessellate_stroke(source, &path, geometry, inherited_opacity)?;
        }
        usvg::PaintOrder::StrokeAndFill => {
            tessellate_stroke(source, &path, geometry, inherited_opacity)?;
            tessellate_fill(source, &path, geometry, inherited_opacity)?;
        }
    }
    Ok(())
}

fn tessellate_fill(
    source: &usvg::Path,
    path: &Path,
    geometry: &mut VertexBuffers<VectorVertex, u32>,
    inherited_opacity: f32,
) -> Result<(), VectorError> {
    if let Some(fill) = source.fill() {
        let color = color(fill.paint(), fill.opacity().get() * inherited_opacity)?;
        let rule = match fill.rule() {
            usvg::FillRule::NonZero => FillRule::NonZero,
            usvg::FillRule::EvenOdd => FillRule::EvenOdd,
        };
        FillTessellator::new()
            .tessellate_path(
                path,
                &FillOptions::default().with_fill_rule(rule),
                &mut BuffersBuilder::new(geometry, |vertex: FillVertex<'_>| {
                    vector_vertex(vertex.position(), color)
                }),
            )
            .map_err(|_| VectorError::Tessellation)?;
    }
    Ok(())
}

fn tessellate_stroke(
    source: &usvg::Path,
    path: &Path,
    geometry: &mut VertexBuffers<VectorVertex, u32>,
    inherited_opacity: f32,
) -> Result<(), VectorError> {
    if let Some(stroke) = source.stroke() {
        if stroke.dasharray().is_some() {
            return Err(VectorError::UnsupportedFeature("dashed stroke"));
        }
        let color = color(stroke.paint(), stroke.opacity().get() * inherited_opacity)?;
        let options = StrokeOptions::default()
            .with_line_width(stroke.width().get())
            .with_miter_limit(stroke.miterlimit().get())
            .with_start_cap(line_cap(stroke.linecap()))
            .with_end_cap(line_cap(stroke.linecap()))
            .with_line_join(line_join(stroke.linejoin()));
        StrokeTessellator::new()
            .tessellate_path(
                path,
                &options,
                &mut BuffersBuilder::new(geometry, |vertex: StrokeVertex<'_, '_>| {
                    vector_vertex(vertex.position(), color)
                }),
            )
            .map_err(|_| VectorError::Tessellation)?;
    }
    Ok(())
}

fn lyon_path(source: &usvg::Path) -> Path {
    let transform = source.abs_transform();
    let map = |p| point_from_array(map_point(transform, p));
    let mut builder = Path::builder();
    let mut open = false;
    for segment in source.data().segments() {
        match segment {
            PathSegment::MoveTo(p) => {
                if open {
                    builder.end(false);
                }
                builder.begin(map(p));
                open = true;
            }
            PathSegment::LineTo(p) => {
                builder.line_to(map(p));
            }
            PathSegment::QuadTo(a, b) => {
                builder.quadratic_bezier_to(map(a), map(b));
            }
            PathSegment::CubicTo(a, b, c) => {
                builder.cubic_bezier_to(map(a), map(b), map(c));
            }
            PathSegment::Close => {
                builder.close();
                open = false;
            }
        }
    }
    if open {
        builder.end(false);
    }
    builder.build()
}

fn map_point(transform: usvg::Transform, point: usvg::tiny_skia_path::Point) -> [f32; 2] {
    [
        point.x * transform.sx + point.y * transform.kx + transform.tx,
        point.x * transform.ky + point.y * transform.sy + transform.ty,
    ]
}

fn point_from_array(value: [f32; 2]) -> lyon_path::math::Point {
    point(value[0], value[1])
}

fn color(paint: &Paint, opacity: f32) -> Result<Color, VectorError> {
    let Paint::Color(color) = paint else {
        return Err(VectorError::UnsupportedPaint);
    };
    Ok(Color::rgba(
        srgb(color.red),
        srgb(color.green),
        srgb(color.blue),
        opacity,
    ))
}

fn srgb(channel: u8) -> f32 {
    let value = f32::from(channel) / 255.0;
    if value <= 0.040_45 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn vector_vertex(position: lyon_path::math::Point, color: Color) -> VectorVertex {
    VectorVertex {
        from: position.to_array(),
        to: position.to_array(),
        color_from: color,
        color_to: color,
    }
}

const fn line_cap(cap: usvg::LineCap) -> LineCap {
    match cap {
        usvg::LineCap::Butt => LineCap::Butt,
        usvg::LineCap::Round => LineCap::Round,
        usvg::LineCap::Square => LineCap::Square,
    }
}

const fn line_join(join: usvg::LineJoin) -> LineJoin {
    match join {
        usvg::LineJoin::Miter | usvg::LineJoin::MiterClip => LineJoin::Miter,
        usvg::LineJoin::Round => LineJoin::Round,
        usvg::LineJoin::Bevel => LineJoin::Bevel,
    }
}
