use std::collections::HashMap;

mod builtin;
mod member;
mod vector_path;

use argui_dsl_syntax::{FileId, Span, SyntaxKind, SyntaxNode};

use crate::{
    AssetId, BinaryOperator, CallbackId, ExpressionId, FieldId, IrExpression, IrExpressionKind,
    IrObservation, IrType, IrValue, LocalId, LowerError, PropertyId, SiteId, SourceInfo, TokenId,
    UnaryOperator, id::hash_text,
};

/// Checked output resolved by an explicit visual source identity.
pub(crate) enum ReferenceProperty {
    Observed {
        site: SiteId,
        property: argui_schema::PropertyId,
        observation: IrObservation,
        value_type: IrType,
    },
    Child {
        site: SiteId,
        property: PropertyId,
        value_type: IrType,
    },
}

/// Readable outputs of explicitly identified visual elements.
pub(crate) type References = HashMap<String, HashMap<String, ReferenceProperty>>;

pub(crate) struct Context<'a> {
    pub file: FileId,
    pub module_path: &'a str,
    pub component: Option<crate::ComponentId>,
    pub site: Option<crate::SiteId>,
    pub properties: &'a HashMap<String, (PropertyId, IrType)>,
    pub callbacks: &'a HashMap<String, (CallbackId, Vec<IrType>, IrType)>,
    pub locals: &'a HashMap<String, (LocalId, IrType)>,
    pub tokens: &'a HashMap<String, (TokenId, IrType)>,
    pub fields: &'a HashMap<argui_dsl_semantic::SymbolId, HashMap<String, (FieldId, IrType)>>,
    pub references: Option<&'a References>,
    pub assets: &'a mut HashMap<String, crate::IrAsset>,
    pub errors: &'a mut Vec<LowerError>,
}

/// Lowers one semantic-validated expression into typed, ID-only IR.
pub(crate) fn lower(node: &SyntaxNode, context: &mut Context<'_>) -> IrExpression {
    let source = source(node, context);
    match node.kind() {
        SyntaxKind::Expr => {
            if let Some(child) = expression_child(node) {
                lower(&child, context)
            } else {
                invalid(node, context, "empty expression")
            }
        }
        SyntaxKind::LiteralExpr => literal(node, source, context),
        SyntaxKind::PathExpr => path(node, source, context),
        SyntaxKind::MemberExpr => member::lower(node, source, context),
        SyntaxKind::CallExpr => call(node, source, context),
        SyntaxKind::UnaryExpr => unary(node, source, context),
        SyntaxKind::BinaryExpr => binary(node, source, context),
        SyntaxKind::ConditionalExpr => conditional(node, source, context),
        SyntaxKind::ArrayExpr => array(node, source, context),
        _ => invalid(node, context, "unsupported expression node"),
    }
}

/// Lowers a scalar/unit/color/string literal.
fn literal(node: &SyntaxNode, source: SourceInfo, context: &mut Context<'_>) -> IrExpression {
    let tokens = tokens(node).collect::<Vec<_>>();
    let Some(first) = tokens.first() else {
        return invalid(node, context, "literal contains no token");
    };
    let (value_type, value) = match first.kind() {
        SyntaxKind::String => (IrType::String, IrValue::String(unquote(first.text()))),
        SyntaxKind::TrueKw => (IrType::Bool, IrValue::Bool(true)),
        SyntaxKind::FalseKw => (IrType::Bool, IrValue::Bool(false)),
        SyntaxKind::NullKw => (IrType::Optional(Box::new(IrType::Unknown)), IrValue::Null),
        SyntaxKind::Hash => {
            let digits = tokens.get(1).map_or("", |token| token.text());
            match parse_color(digits) {
                Some(color) => (IrType::Color, IrValue::Color(color)),
                None => return invalid(node, context, "invalid hexadecimal color literal"),
            }
        }
        SyntaxKind::Number => match parse_number(first.text()) {
            Some(value) => value,
            None => return invalid(node, context, "invalid numeric literal"),
        },
        _ => return invalid(node, context, "unknown literal token"),
    };
    IrExpression {
        id: expression_id(&source),
        value_type,
        kind: IrExpressionKind::Constant(value),
        source,
    }
}

/// Resolves a property or scoped local read to its stable ID.
fn path(node: &SyntaxNode, source: SourceInfo, context: &mut Context<'_>) -> IrExpression {
    let name = direct_name(node).unwrap_or_default();
    if let Some((id, value_type)) = context.locals.get(&name) {
        return IrExpression {
            id: expression_id(&source),
            value_type: value_type.clone(),
            kind: IrExpressionKind::LocalRead(*id),
            source,
        };
    }
    if let Some((id, value_type)) = context.properties.get(&name) {
        return IrExpression {
            id: expression_id(&source),
            value_type: value_type.clone(),
            kind: IrExpressionKind::PropertyRead(*id),
            source,
        };
    }
    invalid(
        node,
        context,
        format!("unresolved expression name `{name}`"),
    )
}

/// Resolves built-in, theme, asset, and component-callback calls.
fn call(node: &SyntaxNode, source: SourceInfo, context: &mut Context<'_>) -> IrExpression {
    let callee = node
        .descendants()
        .find(|child| child.kind() == SyntaxKind::PathExpr)
        .and_then(|path| direct_name(&path))
        .unwrap_or_default();
    let argument_nodes = node
        .children()
        .find(|child| child.kind() == SyntaxKind::ArgumentList)
        .map(|arguments| {
            arguments
                .children()
                .filter(|child| child.kind() == SyntaxKind::Expr)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if let Some(expression) = builtin::lower_call(&callee, &argument_nodes, &source, context) {
        return expression;
    }
    match callee.as_str() {
        "var" => {
            let name = argument_nodes
                .first()
                .and_then(|argument| {
                    argument
                        .descendants_with_tokens()
                        .filter_map(|element| element.into_token())
                        .find(|token| token.kind() == SyntaxKind::ThemeName)
                })
                .map(|token| token.text().to_string())
                .unwrap_or_default();
            let Some((token, value_type)) = context.tokens.get(&name) else {
                return invalid(node, context, format!("unresolved theme token `{name}`"));
            };
            IrExpression {
                id: expression_id(&source),
                value_type: value_type.clone(),
                kind: IrExpressionKind::TokenRead(*token),
                source,
            }
        }
        "asset" => {
            let path = argument_nodes
                .first()
                .and_then(|argument| {
                    argument
                        .descendants_with_tokens()
                        .filter_map(|element| element.into_token())
                        .find(|token| token.kind() == SyntaxKind::String)
                })
                .map(|token| unquote(token.text()))
                .unwrap_or_default();
            if path.is_empty() {
                return invalid(node, context, "asset() requires a static string path");
            }
            let path = crate::declaration::resolve_asset_path(context.module_path, &path);
            let id = crate::declaration::register_asset(context.assets, &path);
            IrExpression {
                id: expression_id(&source),
                value_type: IrType::Asset,
                kind: IrExpressionKind::Asset(id),
                source,
            }
        }
        "path" => vector_path::lower(node, source, context),
        _ => {
            let Some((callback, parameters, result)) = context.callbacks.get(&callee).cloned()
            else {
                return invalid(node, context, format!("unresolved function `{callee}`"));
            };
            let arguments = argument_nodes
                .iter()
                .map(|argument| lower(argument, context))
                .collect::<Vec<_>>();
            if arguments.len() != parameters.len() {
                context.errors.push(LowerError::new(
                    format!(
                        "callback `{callee}` expects {} arguments, received {}",
                        parameters.len(),
                        arguments.len()
                    ),
                    Span::new(context.file, node.text_range()),
                ));
            }
            IrExpression {
                id: expression_id(&source),
                value_type: result,
                kind: IrExpressionKind::CallbackCall {
                    callback,
                    arguments,
                },
                source,
            }
        }
    }
}

/// Lowers a unary operation.
fn unary(node: &SyntaxNode, source: SourceInfo, context: &mut Context<'_>) -> IrExpression {
    let Some(operand_node) = expression_children(node).next() else {
        return invalid(node, context, "unary expression has no operand");
    };
    let operand = lower(&operand_node, context);
    let operator = match direct_operator(node) {
        Some(SyntaxKind::Bang) => UnaryOperator::Not,
        Some(SyntaxKind::Minus) => UnaryOperator::Negate,
        Some(SyntaxKind::Plus) => UnaryOperator::Positive,
        _ => return invalid(node, context, "unsupported unary operator"),
    };
    IrExpression {
        id: expression_id(&source),
        value_type: operand.value_type.clone(),
        kind: IrExpressionKind::Unary {
            operator,
            operand: Box::new(operand),
        },
        source,
    }
}

/// Lowers a binary operation with its validated result type.
fn binary(node: &SyntaxNode, source: SourceInfo, context: &mut Context<'_>) -> IrExpression {
    let operands = expression_children(node).collect::<Vec<_>>();
    let Some(left_node) = operands.first() else {
        return invalid(node, context, "binary expression has no left operand");
    };
    let Some(right_node) = operands.get(1) else {
        return invalid(node, context, "binary expression has no right operand");
    };
    let left = lower(left_node, context);
    let right = lower(right_node, context);
    let operator = match direct_operator(node) {
        Some(SyntaxKind::Plus) => BinaryOperator::Add,
        Some(SyntaxKind::Minus) => BinaryOperator::Subtract,
        Some(SyntaxKind::Star) => BinaryOperator::Multiply,
        Some(SyntaxKind::Slash) => BinaryOperator::Divide,
        Some(SyntaxKind::Percent) => BinaryOperator::Remainder,
        Some(SyntaxKind::EqEq) => BinaryOperator::Equal,
        Some(SyntaxKind::BangEq) => BinaryOperator::NotEqual,
        Some(SyntaxKind::Lt) => BinaryOperator::Less,
        Some(SyntaxKind::LtEq) => BinaryOperator::LessEqual,
        Some(SyntaxKind::Gt) => BinaryOperator::Greater,
        Some(SyntaxKind::GtEq) => BinaryOperator::GreaterEqual,
        Some(SyntaxKind::AndAnd) => BinaryOperator::And,
        Some(SyntaxKind::OrOr) => BinaryOperator::Or,
        _ => return invalid(node, context, "unsupported binary operator"),
    };
    let value_type = if matches!(
        operator,
        BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual
            | BinaryOperator::And
            | BinaryOperator::Or
    ) {
        IrType::Bool
    } else if operator == BinaryOperator::Divide
        && left.value_type == IrType::Length
        && right.value_type == IrType::Length
    {
        IrType::Float
    } else if operator == BinaryOperator::Multiply
        && matches!(
            (&left.value_type, &right.value_type),
            (IrType::Float | IrType::Int, IrType::Length)
        )
    {
        IrType::Length
    } else if left.value_type == IrType::Int && right.value_type == IrType::Float
        || left.value_type == IrType::Float && right.value_type == IrType::Int
    {
        IrType::Float
    } else {
        left.value_type.clone()
    };
    IrExpression {
        id: expression_id(&source),
        value_type,
        kind: IrExpressionKind::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        },
        source,
    }
}

/// Lowers a ternary conditional.
fn conditional(node: &SyntaxNode, source: SourceInfo, context: &mut Context<'_>) -> IrExpression {
    let values = expression_children(node).collect::<Vec<_>>();
    if values.len() < 3 {
        return invalid(node, context, "conditional expression is incomplete");
    }
    let condition = lower(&values[0], context);
    let then_value = lower(&values[1], context);
    let else_value = lower(&values[2], context);
    IrExpression {
        id: expression_id(&source),
        value_type: if then_value.value_type == IrType::Int
            && else_value.value_type == IrType::Float
        {
            IrType::Float
        } else {
            then_value.value_type.clone()
        },
        kind: IrExpressionKind::Conditional {
            condition: Box::new(condition),
            then_value: Box::new(then_value),
            else_value: Box::new(else_value),
        },
        source,
    }
}

/// Lowers an array literal.
fn array(node: &SyntaxNode, source: SourceInfo, context: &mut Context<'_>) -> IrExpression {
    let values = node
        .children()
        .filter(|child| child.kind() == SyntaxKind::Expr)
        .map(|child| lower(&child, context))
        .collect::<Vec<_>>();
    let element_type = values
        .first()
        .map_or(IrType::Unknown, |value| value.value_type.clone());
    IrExpression {
        id: expression_id(&source),
        value_type: IrType::Array(Box::new(element_type)),
        kind: IrExpressionKind::Array(values),
        source,
    }
}

/// Emits a lowering invariant failure and returns a typed unknown constant.
fn invalid(
    node: &SyntaxNode,
    context: &mut Context<'_>,
    message: impl Into<String>,
) -> IrExpression {
    let source = source(node, context);
    context.errors.push(LowerError::new(
        message,
        source.span.expect("lowered source retains its span"),
    ));
    IrExpression {
        id: expression_id(&source),
        value_type: IrType::Unknown,
        kind: IrExpressionKind::Constant(IrValue::Null),
        source,
    }
}

/// Creates source metadata for an expression node.
fn source(node: &SyntaxNode, context: &Context<'_>) -> SourceInfo {
    SourceInfo::new(
        Span::new(context.file, node.text_range()),
        context.component,
        context.site,
    )
}

/// Derives a package-local expression identity from its owner and exact syntax range.
pub(crate) fn expression_id(source: &SourceInfo) -> ExpressionId {
    let span = source.span.expect("lowered source retains its span");
    let namespace = source
        .site
        .map_or_else(|| u64::from(span.file.raw()), crate::SiteId::raw);
    let start = u64::from(u32::from(span.range.start()));
    let end = u64::from(u32::from(span.range.end()));
    ExpressionId::from_raw(crate::id::derive(
        namespace,
        "expression",
        (start << 32) | end,
    ))
}

/// Parses a numeric literal and retains its unit as the IR type.
fn parse_number(text: &str) -> Option<(IrType, IrValue)> {
    let (value_type, suffix) = if text.ends_with("px") {
        (IrType::Length, "px")
    } else if text.ends_with('%') {
        (IrType::Percentage, "%")
    } else if text.ends_with("ms") {
        (IrType::Duration, "ms")
    } else if text.ends_with("deg") {
        (IrType::Angle, "deg")
    } else if text.ends_with("rad") {
        (IrType::Angle, "rad")
    } else if text.ends_with('s') {
        (IrType::Duration, "s")
    } else if text.contains(['.', 'e', 'E']) {
        (IrType::Float, "")
    } else {
        return text
            .replace('_', "")
            .parse::<i64>()
            .ok()
            .map(|value| (IrType::Int, IrValue::Int(value)));
    };
    text.strip_suffix(suffix)?
        .replace('_', "")
        .parse::<f64>()
        .ok()
        .map(|value| {
            let value = if suffix == "s" { value * 1000.0 } else { value };
            (value_type, IrValue::Float(value))
        })
}

/// Parses CSS-style RGB/RGBA hexadecimal digits into RGBA8.
fn parse_color(text: &str) -> Option<u32> {
    let expanded = match text.len() {
        3 => format!(
            "{}{}{}{}{}{}ff",
            &text[0..1],
            &text[0..1],
            &text[1..2],
            &text[1..2],
            &text[2..3],
            &text[2..3]
        ),
        4 => format!(
            "{}{}{}{}{}{}{}{}",
            &text[0..1],
            &text[0..1],
            &text[1..2],
            &text[1..2],
            &text[2..3],
            &text[2..3],
            &text[3..4],
            &text[3..4]
        ),
        6 => format!("{text}ff"),
        8 => text.into(),
        _ => return None,
    };
    u32::from_str_radix(&expanded, 16).ok()
}

/// Iterates direct non-trivia tokens.
fn tokens(
    node: &SyntaxNode,
) -> impl DoubleEndedIterator<Item = argui_dsl_syntax::SyntaxToken> + '_ {
    node.children_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| !token.kind().is_trivia())
        .collect::<Vec<_>>()
        .into_iter()
}

/// Returns direct child expressions.
fn expression_children(node: &SyntaxNode) -> impl Iterator<Item = SyntaxNode> + '_ {
    node.children().filter(|child| is_expression(child.kind()))
}

/// Returns the first direct child expression.
fn expression_child(node: &SyntaxNode) -> Option<SyntaxNode> {
    expression_children(node).next()
}

/// Returns whether a syntax kind carries expression semantics.
fn is_expression(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Expr
            | SyntaxKind::LiteralExpr
            | SyntaxKind::PathExpr
            | SyntaxKind::MemberExpr
            | SyntaxKind::CallExpr
            | SyntaxKind::UnaryExpr
            | SyntaxKind::BinaryExpr
            | SyntaxKind::ConditionalExpr
            | SyntaxKind::ArrayExpr
    )
}

/// Returns the first direct identifier.
fn direct_name(node: &SyntaxNode) -> Option<String> {
    tokens(node)
        .find(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text().to_string())
}

/// Returns a direct expression operator token.
fn direct_operator(node: &SyntaxNode) -> Option<SyntaxKind> {
    tokens(node).map(|token| token.kind()).find(|kind| {
        matches!(
            kind,
            SyntaxKind::Bang
                | SyntaxKind::Plus
                | SyntaxKind::Minus
                | SyntaxKind::Star
                | SyntaxKind::Slash
                | SyntaxKind::Percent
                | SyntaxKind::EqEq
                | SyntaxKind::BangEq
                | SyntaxKind::Lt
                | SyntaxKind::LtEq
                | SyntaxKind::Gt
                | SyntaxKind::GtEq
                | SyntaxKind::AndAnd
                | SyntaxKind::OrOr
        )
    })
}

/// Unquotes a static string token.
fn unquote(text: &str) -> String {
    text.strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .unwrap_or(text)
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}
