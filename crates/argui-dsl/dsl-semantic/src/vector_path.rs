//! Static, typed path geometry shared by semantic checking and IR lowering.

use argui_dsl_syntax::{SyntaxKind, SyntaxNode, TextRange};

/// One absolute command in an authored vector path.
#[derive(Clone, Debug, PartialEq)]
pub enum PathCommandSpec {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadraticTo(f32, f32, f32, f32),
    CubicTo(f32, f32, f32, f32, f32, f32),
    Close,
}

/// Literal geometry and paint style for one compiled vector asset.
#[derive(Clone, Debug, PartialEq)]
pub struct PathSpec {
    pub width: f32,
    pub height: f32,
    pub commands: Vec<PathCommandSpec>,
    pub fill: bool,
    /// Zero disables the stroke; positive values enable it.
    pub stroke_width: f32,
    pub even_odd: bool,
}

/// Source-located static path syntax or type failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathSyntaxError {
    pub message: String,
    pub range: TextRange,
}

/// Parses a `path(width, height, commands, fill, stroke_width, even_odd)` call.
///
/// `node` is the complete parsed call expression. Returns literal geometry
/// shared by both backends. Every coordinate and style value must be static.
///
/// # Errors
///
/// Returns the exact offending syntax range for invalid arity, commands,
/// units, or runtime-dependent geometry.
pub fn parse(node: &SyntaxNode) -> Result<PathSpec, PathSyntaxError> {
    let arguments = arguments(node);
    if arguments.len() != 6 {
        return Err(error(
            node,
            "path() expects width, height, commands, fill, stroke_width, and even_odd",
        ));
    }
    let width = number(&arguments[0], "path width")?;
    let height = number(&arguments[1], "path height")?;
    let commands = commands(&arguments[2])?;
    let fill = boolean(&arguments[3], "path fill")?;
    let stroke_width = number(&arguments[4], "path stroke_width")?;
    let even_odd = boolean(&arguments[5], "path even_odd")?;
    Ok(PathSpec {
        width,
        height,
        commands,
        fill,
        stroke_width,
        even_odd,
    })
}

/// Parses a literal array of command constructor calls.
fn commands(node: &SyntaxNode) -> Result<Vec<PathCommandSpec>, PathSyntaxError> {
    let value = inner(node);
    if value.kind() != SyntaxKind::ArrayExpr {
        return Err(error(node, "path commands must be a literal command array"));
    }
    value
        .children()
        .filter(|child| child.kind() == SyntaxKind::Expr)
        .map(|child| command(&child))
        .collect()
}

/// Parses one command and checks its numeric argument count.
fn command(node: &SyntaxNode) -> Result<PathCommandSpec, PathSyntaxError> {
    let value = inner(node);
    if value.kind() != SyntaxKind::CallExpr {
        return Err(error(
            node,
            "path commands must be literal constructor calls",
        ));
    }
    let name = value
        .children()
        .find(|child| child.kind() == SyntaxKind::PathExpr)
        .and_then(|child| {
            child
                .children_with_tokens()
                .filter_map(|part| part.into_token())
                .find(|token| token.kind() == SyntaxKind::Ident)
        })
        .map(|token| token.text().to_string())
        .unwrap_or_default();
    let expected = match name.as_str() {
        "move_to" | "line_to" => 2,
        "quadratic_to" => 4,
        "cubic_to" => 6,
        "close_path" => 0,
        _ => {
            return Err(error(
                node,
                "unknown path command; use move_to, line_to, quadratic_to, cubic_to, or close_path",
            ));
        }
    };
    let arguments = arguments(&value);
    if arguments.len() != expected {
        return Err(error(
            node,
            format!("{name}() expects {expected} numeric coordinates"),
        ));
    }
    let values = arguments
        .iter()
        .map(|argument| number(argument, "path coordinate"))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(match name.as_str() {
        "move_to" => PathCommandSpec::MoveTo(values[0], values[1]),
        "line_to" => PathCommandSpec::LineTo(values[0], values[1]),
        "quadratic_to" => PathCommandSpec::QuadraticTo(values[0], values[1], values[2], values[3]),
        "cubic_to" => PathCommandSpec::CubicTo(
            values[0], values[1], values[2], values[3], values[4], values[5],
        ),
        "close_path" => PathCommandSpec::Close,
        _ => unreachable!("command name was checked above"),
    })
}

/// Reads a finite, unitless numeric literal, including an optional unary sign.
fn number(node: &SyntaxNode, role: &str) -> Result<f32, PathSyntaxError> {
    let value = inner(node);
    let (negative, literal) = if value.kind() == SyntaxKind::UnaryExpr {
        let negative = value
            .children_with_tokens()
            .filter_map(|part| part.into_token())
            .any(|token| token.kind() == SyntaxKind::Minus);
        let Some(operand) = value
            .children()
            .find(|child| child.kind() == SyntaxKind::LiteralExpr)
        else {
            return Err(error(
                node,
                format!("{role} must be a numeric literal; dynamic path geometry is unsupported"),
            ));
        };
        (negative, operand)
    } else {
        (false, value)
    };
    if literal.kind() != SyntaxKind::LiteralExpr {
        return Err(error(
            node,
            format!("{role} must be a numeric literal; dynamic path geometry is unsupported"),
        ));
    }
    let raw = literal
        .children_with_tokens()
        .filter_map(|part| part.into_token())
        .find(|token| token.kind() == SyntaxKind::Number)
        .map(|token| token.text().to_string())
        .ok_or_else(|| error(node, format!("{role} must be a numeric literal")))?;
    let parsed = raw.parse::<f32>().map_err(|_| {
        error(
            node,
            format!("{role} must be a finite, unitless numeric literal"),
        )
    })?;
    let result = if negative { -parsed } else { parsed };
    if !result.is_finite() {
        return Err(error(node, format!("{role} must be finite")));
    }
    Ok(result)
}

/// Reads a boolean literal for one path style field.
fn boolean(node: &SyntaxNode, role: &str) -> Result<bool, PathSyntaxError> {
    let value = inner(node);
    if value.kind() != SyntaxKind::LiteralExpr {
        return Err(error(node, format!("{role} must be a bool literal")));
    }
    match value
        .children_with_tokens()
        .filter_map(|part| part.into_token())
        .find(|token| matches!(token.kind(), SyntaxKind::TrueKw | SyntaxKind::FalseKw))
        .map(|token| token.kind())
    {
        Some(SyntaxKind::TrueKw) => Ok(true),
        Some(SyntaxKind::FalseKw) => Ok(false),
        _ => Err(error(node, format!("{role} must be a bool literal"))),
    }
}

/// Returns direct call arguments in source order.
fn arguments(node: &SyntaxNode) -> Vec<SyntaxNode> {
    node.children()
        .find(|child| child.kind() == SyntaxKind::ArgumentList)
        .map(|list| {
            list.children()
                .filter(|child| child.kind() == SyntaxKind::Expr)
                .collect()
        })
        .unwrap_or_default()
}

/// Removes the parser's single expression wrapper when present.
fn inner(node: &SyntaxNode) -> SyntaxNode {
    if node.kind() == SyntaxKind::Expr {
        node.children().next().unwrap_or_else(|| node.clone())
    } else {
        node.clone()
    }
}

/// Creates one syntax error at `node` with a human-readable `message`.
fn error(node: &SyntaxNode, message: impl Into<String>) -> PathSyntaxError {
    PathSyntaxError {
        message: message.into(),
        range: node.text_range(),
    }
}
