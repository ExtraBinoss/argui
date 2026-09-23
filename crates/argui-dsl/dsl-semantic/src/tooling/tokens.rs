use argui_dsl_syntax::{SyntaxKind, SyntaxNode, TextSize};

use crate::{CompilerDatabase, DefinitionKind, DiagnosticCode};

use super::{
    CodeAction, ColorPresentation, Location, RenameEdit, SemanticClass, SemanticHighlight,
    ToolingError, direct_name,
};

/// Classifies every relevant token without reimplementing parsing in the LSP.
pub(super) fn semantic_highlights(
    database: &mut CompilerDatabase,
    path: &str,
) -> Result<Vec<SemanticHighlight>, ToolingError> {
    let context = database.source_context(path, 0)?;
    let mut output = Vec::new();
    for token in context
        .root
        .descendants_with_tokens()
        .filter_map(|item| item.into_token())
    {
        let class = match token.kind() {
            SyntaxKind::LineComment | SyntaxKind::BlockComment => Some(SemanticClass::Comment),
            SyntaxKind::String => Some(SemanticClass::String),
            SyntaxKind::Number => Some(SemanticClass::Number),
            SyntaxKind::ThemeName => Some(SemanticClass::ThemeToken),
            kind if is_keyword(kind) => Some(SemanticClass::Keyword),
            SyntaxKind::Ident => Some(classify_identifier(&context, &token)),
            _ => None,
        };
        if let Some(class) = class {
            output.push(SemanticHighlight {
                start: token.text_range().start().into(),
                end: token.text_range().end().into(),
                class,
            });
        }
    }
    Ok(output)
}

/// Builds typo corrections for unknown component and property diagnostics.
pub(super) fn code_actions(
    database: &mut CompilerDatabase,
    path: &str,
) -> Result<Vec<CodeAction>, ToolingError> {
    let context = database.source_context(path, 0)?;
    let mut output = Vec::new();
    for diagnostic in context
        .project
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.primary.file == context.file)
    {
        let fragment = usize::from(diagnostic.code == DiagnosticCode::UnknownProperty);
        let Some(misspelled) = quoted(&diagnostic.message, fragment) else {
            continue;
        };
        let candidates = match diagnostic.code {
            DiagnosticCode::UnknownComponent => component_candidates(database, &context),
            DiagnosticCode::UnknownProperty => {
                property_candidates(database, &context, diagnostic.primary.range.start().into())
            }
            _ => Vec::new(),
        };
        let Some(replacement) = nearest(&misspelled, candidates) else {
            continue;
        };
        let range = identifier_range(&context.root, diagnostic.primary.range.start().into())
            .unwrap_or(diagnostic.primary.range);
        output.push(CodeAction {
            title: format!("Change `{misspelled}` to `{replacement}`"),
            edit: RenameEdit {
                location: Location {
                    path: path.into(),
                    start: range.start().into(),
                    end: range.end().into(),
                },
                replacement,
            },
        });
    }
    Ok(output)
}

/// Parses and normalizes the hexadecimal color literal surrounding `offset`.
pub(super) fn color_presentation(
    database: &mut CompilerDatabase,
    path: &str,
    offset: u32,
) -> Result<Option<ColorPresentation>, ToolingError> {
    let context = database.source_context(path, offset)?;
    let source = database.source(context.file).unwrap_or_default();
    let offset = usize::try_from(offset)
        .unwrap_or(source.len())
        .min(source.len());
    let start = if source.as_bytes().get(offset) == Some(&b'#') {
        offset
    } else {
        source[..offset].rfind('#').unwrap_or(offset)
    };
    if start == offset && source.as_bytes().get(start) != Some(&b'#') {
        return Ok(None);
    }
    let end = source[start + 1..]
        .char_indices()
        .take_while(|(_, character)| character.is_ascii_hexdigit())
        .last()
        .map_or(start + 1, |(index, character)| {
            start + 1 + index + character.len_utf8()
        });
    if offset > end {
        return Ok(None);
    }
    let digits = &source[start + 1..end];
    let Some([red, green, blue, alpha]) = color_channels(digits) else {
        return Ok(None);
    };
    Ok(Some(ColorPresentation {
        label: format!("#{red:02x}{green:02x}{blue:02x}{alpha:02x}"),
        red: f32::from(red) / 255.0,
        green: f32::from(green) / 255.0,
        blue: f32::from(blue) / 255.0,
        alpha: f32::from(alpha) / 255.0,
        location: Location {
            path: path.into(),
            start: u32::try_from(start).unwrap_or(u32::MAX),
            end: u32::try_from(end).unwrap_or(u32::MAX),
        },
    }))
}

/// Determines one identifier's semantic highlighting class.
fn classify_identifier(
    context: &super::SourceContext,
    token: &argui_dsl_syntax::SyntaxToken,
) -> SemanticClass {
    if let Some(id) = context.module.scope.get(token.text())
        && let Some(definition) = context.project.definition(*id)
    {
        return match definition.kind {
            DefinitionKind::Component(_) => SemanticClass::Component,
            DefinitionKind::Struct(_) | DefinitionKind::Enum(_) => SemanticClass::Type,
            DefinitionKind::Theme(_) | DefinitionKind::Style(_) | DefinitionKind::Effect(_) => {
                SemanticClass::Type
            }
            DefinitionKind::Function(_) => SemanticClass::Variable,
        };
    }
    if context.module.native_scope.contains_key(token.text()) {
        return SemanticClass::Component;
    }
    if token.parent_ancestors().any(|node| {
        matches!(
            node.kind(),
            SyntaxKind::PropertyDecl | SyntaxKind::PropertyAssignment
        )
    }) {
        return SemanticClass::Property;
    }
    if token.parent_ancestors().any(|node| {
        matches!(
            node.kind(),
            SyntaxKind::CallbackDecl | SyntaxKind::EventBlock
        )
    }) {
        return SemanticClass::Callback;
    }
    SemanticClass::Variable
}

/// Returns public component and native names visible to an import fix.
fn component_candidates(
    database: &CompilerDatabase,
    context: &super::SourceContext,
) -> Vec<String> {
    let mut values = context
        .module
        .scope
        .keys()
        .chain(context.module.native_scope.keys())
        .cloned()
        .collect::<Vec<_>>();
    values.extend(
        database
            .schema()
            .schemas()
            .map(|schema| schema.name.to_string()),
    );
    values.sort();
    values.dedup();
    values
}

/// Returns properties for the element enclosing one diagnostic offset.
fn property_candidates(
    database: &CompilerDatabase,
    context: &super::SourceContext,
    offset: u32,
) -> Vec<String> {
    let Some(token) = context
        .root
        .token_at_offset(TextSize::from(offset))
        .right_biased()
    else {
        return Vec::new();
    };
    let Some(element) = token
        .parent_ancestors()
        .find(|node| node.kind() == SyntaxKind::Element)
    else {
        return Vec::new();
    };
    let Some(target) = direct_name(&element) else {
        return Vec::new();
    };
    if let Some(schema) = context
        .module
        .native_scope
        .get(&target)
        .and_then(|id| database.schema().schema(*id))
    {
        return schema
            .properties
            .iter()
            .map(|property| property.name.to_string())
            .collect();
    }
    context
        .module
        .scope
        .get(&target)
        .and_then(|id| context.project.definition(*id))
        .and_then(|definition| match &definition.kind {
            DefinitionKind::Component(component) => Some(
                component
                    .properties
                    .iter()
                    .map(|property| property.name.clone())
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

/// Extracts the indexed backtick-delimited fragment from a diagnostic message.
fn quoted(message: &str, index: usize) -> Option<String> {
    message
        .split('`')
        .nth(index.saturating_mul(2).saturating_add(1))
        .map(str::to_owned)
}

/// Selects a close spelling with a conservative edit-distance threshold.
fn nearest(value: &str, candidates: Vec<String>) -> Option<String> {
    candidates
        .into_iter()
        .map(|candidate| (distance(value, &candidate), candidate))
        .filter(|(distance, candidate)| *distance <= 3 || *distance * 2 <= candidate.len())
        .min_by(|left, right| left.cmp(right))
        .map(|(_, candidate)| candidate)
}

/// Computes Unicode-scalar Levenshtein distance with bounded row storage.
fn distance(left: &str, right: &str) -> usize {
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    for (row, left) in left.chars().enumerate() {
        let mut current = vec![row + 1];
        for (column, right) in right.iter().enumerate() {
            current.push(
                (previous[column + 1] + 1)
                    .min(current[column] + 1)
                    .min(previous[column] + usize::from(left != *right)),
            );
        }
        previous = current;
    }
    previous[right.len()]
}

/// Finds the identifier token which intersects an error start offset.
fn identifier_range(root: &SyntaxNode, offset: u32) -> Option<argui_dsl_syntax::TextRange> {
    let offset = TextSize::from(offset);
    root.token_at_offset(offset)
        .right_biased()
        .or_else(|| root.token_at_offset(offset).left_biased())
        .filter(|token| token.kind() == SyntaxKind::Ident)
        .map(|token| token.text_range())
}

/// Converts supported hexadecimal CSS-like lengths to RGBA bytes.
fn color_channels(digits: &str) -> Option<[u8; 4]> {
    let expand = |digit: u8| (digit << 4) | digit;
    let nibble = |byte: u8| match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    };
    let bytes = digits.as_bytes();
    Some(match bytes.len() {
        3 => [
            expand(nibble(bytes[0])?),
            expand(nibble(bytes[1])?),
            expand(nibble(bytes[2])?),
            255,
        ],
        4 => [
            expand(nibble(bytes[0])?),
            expand(nibble(bytes[1])?),
            expand(nibble(bytes[2])?),
            expand(nibble(bytes[3])?),
        ],
        6 | 8 => {
            let pair = |index| u8::from_str_radix(&digits[index..index + 2], 16).ok();
            [
                pair(0)?,
                pair(2)?,
                pair(4)?,
                if bytes.len() == 8 { pair(6)? } else { 255 },
            ]
        }
        _ => return None,
    })
}

/// Returns whether a syntax token is a reserved language keyword.
fn is_keyword(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ImportKw
            | SyntaxKind::FromKw
            | SyntaxKind::AsKw
            | SyntaxKind::ExportKw
            | SyntaxKind::StructKw
            | SyntaxKind::EnumKw
            | SyntaxKind::ComponentKw
            | SyntaxKind::ThemeKw
            | SyntaxKind::StyleKw
            | SyntaxKind::EffectKw
            | SyntaxKind::FnKw
            | SyntaxKind::ForKw
            | SyntaxKind::InKw
            | SyntaxKind::KeyKw
            | SyntaxKind::IfKw
            | SyntaxKind::ElseKw
            | SyntaxKind::PropertyKw
            | SyntaxKind::OutKw
            | SyntaxKind::InOutKw
            | SyntaxKind::PrivateKw
            | SyntaxKind::CallbackKw
            | SyntaxKind::SlotKw
            | SyntaxKind::OnKw
            | SyntaxKind::StatesKw
            | SyntaxKind::WhenKw
            | SyntaxKind::AnimateKw
            | SyntaxKind::ParameterKw
            | SyntaxKind::ShaderKw
            | SyntaxKind::TrueKw
            | SyntaxKind::FalseKw
            | SyntaxKind::NullKw
            | SyntaxKind::ForTargetKw
            | SyntaxKind::LetKw
            | SyntaxKind::ReturnKw
    )
}
