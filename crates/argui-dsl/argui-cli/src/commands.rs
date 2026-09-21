use std::{fs, path::Path};

use argui_dsl_semantic::{CompilerDatabase, DefinitionKind, SymbolKind};
use serde_json::{Value, json};

use crate::{ProjectFiles, canonical_relative};

/// Formats selected DSL files or every module below the project root.
///
/// * `root` — project root used to resolve relative paths.
/// * `paths` — files or directories to format; an empty slice selects the project.
/// * `check` — reports drift without writing when true.
///
/// # Errors
///
/// Returns filesystem, parse, or formatting-drift errors.
pub fn format(
    root: &Path,
    paths: &[String],
    check: bool,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    if paths.is_empty() {
        files.extend(
            ProjectFiles::read(root)?
                .modules
                .into_iter()
                .map(|module| root.join(module.path)),
        );
    } else {
        for selected in paths {
            collect_selected(root, &resolve(root, selected), &mut files)?;
        }
    }
    files.sort();
    files.dedup();
    let mut changed = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path)?;
        let parsed = argui_dsl_parser::parse(&source);
        if let Some(diagnostic) = parsed.diagnostics().first() {
            return Err(format!(
                "{}: cannot format invalid source: {diagnostic:?}",
                path.display()
            )
            .into());
        }
        let formatted = argui_dsl_parser::format_source(&source);
        if formatted != source {
            let relative = canonical_relative(root, &path)?;
            changed.push(relative);
            if !check {
                fs::write(path, formatted)?;
            }
        }
    }
    if check && !changed.is_empty() {
        return Err(format!(
            "{} DSL file(s) need formatting: {}",
            changed.len(),
            changed.join(", ")
        )
        .into());
    }
    Ok(json!({"ok": true, "changed": changed, "check": check}))
}

/// Returns context-sensitive completion data as stable JSON.
///
/// * `root` — project root containing all loaded `.argui` files.
/// * `path` — project-relative or absolute source path.
/// * `position` — one-based `line:column` in Unicode scalar values.
///
/// # Errors
///
/// Returns project loading, position, or semantic query errors.
pub fn complete(
    root: &Path,
    path: &str,
    position: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let project = ProjectFiles::read(root)?;
    let module_path = canonical_relative(root, &resolve(root, path))?;
    let source = project
        .modules
        .iter()
        .find(|module| module.path == module_path)
        .ok_or_else(|| format!("module `{module_path}` is not part of the project"))?;
    let offset = line_column_offset(&source.source, position)?;
    let mut database = project.semantic_database()?;
    let items = database
        .completions(&module_path, offset)?
        .into_iter()
        .map(|item| {
            json!({
                "label": item.label,
                "kind": symbol_kind(item.kind),
                "detail": item.detail,
                "documentation": item.documentation,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({"path": module_path, "offset": offset, "items": items}))
}

/// Returns document or workspace symbols as stable JSON.
///
/// * `root` — project root containing DSL sources.
/// * `path` — optional source path for document-only symbols.
///
/// # Errors
///
/// Returns project loading or semantic query errors.
pub fn symbols(root: &Path, path: Option<&str>) -> Result<Value, Box<dyn std::error::Error>> {
    let project = ProjectFiles::read(root)?;
    let selected = path
        .map(|path| canonical_relative(root, &resolve(root, path)))
        .transpose()?;
    let mut database = project.semantic_database()?;
    let symbols = database
        .symbols(selected.as_deref())?
        .into_iter()
        .map(|symbol| {
            json!({
                "name": symbol.name,
                "kind": symbol_kind(symbol.kind),
                "path": symbol.location.path,
                "start": symbol.location.start,
                "end": symbol.location.end,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({"symbols": symbols}))
}

/// Returns canonical native and DSL standard-library component metadata.
///
/// * `selected` — optional exact component/primitive name.
///
/// # Errors
///
/// Returns schema construction errors or an unknown-component error.
pub fn schema(selected: Option<&str>) -> Result<Value, Box<dyn std::error::Error>> {
    let mut database = CompilerDatabase::with_builtins()?;
    let mut entries = database
        .schema()
        .schemas()
        .filter(|schema| selected.is_none_or(|name| schema.name.as_str() == name))
        .map(native_schema)
        .collect::<Vec<_>>();
    let project = database.check();
    entries.extend(
        project
            .modules
            .iter()
            .filter(|module| module.path.starts_with("@argui/ui/"))
            .flat_map(|module| &module.definitions)
            .filter(|definition| selected.is_none_or(|name| definition.name == name))
            .filter_map(dsl_schema),
    );
    entries.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    if selected.is_some() && entries.is_empty() {
        return Err(format!("unknown component `{}`", selected.unwrap_or_default()).into());
    }
    Ok(json!({"components": entries}))
}

/// Encodes one native schema as JSON without exposing Rust serialization layout.
fn native_schema(schema: &argui_schema::NativeSchema) -> Value {
    json!({
        "kind": "native",
        "id": schema.id.raw(),
        "name": schema.name.as_str(),
        "documentation": schema.documentation,
        "properties": schema.properties.iter().map(|property| json!({
            "id": property.id.raw(),
            "name": property.name.as_str(),
            "type": format!("{:?}", property.value_type),
            "required": property.required,
            "documentation": property.documentation,
        })).collect::<Vec<_>>(),
        "events": schema.events.iter().map(|event| json!({
            "id": event.id.raw(),
            "name": event.name.as_str(),
            "payload": event.payload.map(|payload| format!("{payload:?}")),
            "documentation": event.documentation,
        })).collect::<Vec<_>>(),
        "slots": schema.slots.iter().map(|slot| json!({
            "id": slot.id.raw(),
            "name": slot.name.as_str(),
            "arity": format!("{:?}", slot.arity),
            "documentation": slot.documentation,
        })).collect::<Vec<_>>(),
    })
}

/// Encodes one standard-library component definition as JSON.
fn dsl_schema(definition: &argui_dsl_semantic::Definition) -> Option<Value> {
    let DefinitionKind::Component(component) = &definition.kind else {
        return None;
    };
    Some(json!({
        "kind": "component",
        "id": definition.id.raw(),
        "name": definition.name,
        "documentation": format!("Official @argui/ui component `{}`.", definition.name),
        "properties": component.properties.iter().map(|property| json!({
            "name": property.name,
            "type": property.value_type.to_string(),
            "direction": format!("{:?}", property.direction),
            "required": property.required,
        })).collect::<Vec<_>>(),
        "events": component.callbacks.iter().map(|callback| json!({
            "name": callback.name,
            "parameters": callback.parameters.iter().map(|parameter| json!({
                "name": parameter.name,
                "type": parameter.value_type.to_string(),
            })).collect::<Vec<_>>(),
            "result": callback.result.to_string(),
        })).collect::<Vec<_>>(),
        "slots": component.slots.iter().map(|slot| json!({"name": slot.name})).collect::<Vec<_>>(),
    }))
}

/// Maps semantic symbol kinds to stable lowercase protocol labels.
fn symbol_kind(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Component => "component",
        SymbolKind::Property => "property",
        SymbolKind::Callback => "callback",
        SymbolKind::Slot => "slot",
        SymbolKind::Struct => "struct",
        SymbolKind::Enum => "enum",
        SymbolKind::Theme => "theme",
        SymbolKind::Token => "token",
        SymbolKind::Style => "style",
        SymbolKind::Effect => "effect",
        SymbolKind::Native => "native",
        SymbolKind::Keyword => "keyword",
    }
}

/// Resolves one user path without requiring it to exist yet.
fn resolve(root: &Path, path: &str) -> std::path::PathBuf {
    let path = Path::new(path);
    if path.is_absolute() {
        path.into()
    } else {
        root.join(path)
    }
}

/// Recursively selects formatter inputs while preserving project boundaries.
fn collect_selected(
    root: &Path,
    path: &Path,
    output: &mut Vec<std::path::PathBuf>,
) -> Result<(), std::io::Error> {
    let _ = path.strip_prefix(root).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "format path is outside project root",
        )
    })?;
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let child = entry.path();
            if child.is_dir() {
                let name = child
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default();
                if !matches!(name, ".git" | ".codex" | "target" | "node_modules") {
                    collect_selected(root, &child, output)?;
                }
            } else if child
                .extension()
                .is_some_and(|extension| extension == "argui")
            {
                output.push(child);
            }
        }
    } else if path
        .extension()
        .is_some_and(|extension| extension == "argui")
    {
        output.push(path.into());
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "format inputs must be .argui files or directories",
        ));
    }
    Ok(())
}

/// Converts `position` in one-based `line:column` form within `source` to a UTF-8 byte offset.
///
/// # Errors
///
/// Returns an error for malformed, zero, or out-of-range coordinates.
fn line_column_offset(source: &str, position: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let (line, column) = position
        .split_once(':')
        .ok_or("position must be LINE:COLUMN")?;
    let line = line.parse::<usize>()?;
    let column = column.parse::<usize>()?;
    if line == 0 || column == 0 {
        return Err("line and column are one-based".into());
    }
    let mut lines = source.split_inclusive('\n');
    let mut line_start = 0;
    for _ in 1..line {
        line_start += lines.next().ok_or("line is outside the source")?.len();
    }
    if line > 1 && line_start == source.len() && !source.ends_with('\n') {
        return Err("line is outside the source".into());
    }
    let text = source[line_start..]
        .split_once('\n')
        .map_or(&source[line_start..], |(line, _)| line);
    let in_line = text.char_indices().nth(column - 1).map_or_else(
        || {
            if column == text.chars().count() + 1 {
                Ok(text.len())
            } else {
                Err("column is outside the line")
            }
        },
        |(offset, _)| Ok(offset),
    )?;
    Ok(u32::try_from(line_start + in_line)?)
}
