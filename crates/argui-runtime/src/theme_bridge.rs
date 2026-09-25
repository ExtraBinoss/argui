//! JSON boundary for application-owned theme sessions in native and Web hosts.

use std::{collections::HashMap, sync::Arc};

use argui_core::{Color, ColorScheme};
use argui_paint::Fill;
use argui_theme::{
    ThemeChange, ThemeImpact, ThemeMode, ThemeRuntime, ThemeSchema, ThemeSelection,
    ThemeTokenDefinition, ThemeValue, ThemeValueType,
};
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value, json};

/// Theme sessions owned by one application host. No global registration is used.
pub struct ThemeBridge {
    next_id: u64,
    sessions: HashMap<u64, ThemeSession>,
    system_scheme: ColorScheme,
}

impl Default for ThemeBridge {
    /// Creates an empty host registry with the light scheme until platform detection.
    fn default() -> Self {
        Self {
            next_id: 0,
            sessions: HashMap::new(),
            system_scheme: ColorScheme::Light,
        }
    }
}

struct ThemeSession {
    runtime: ThemeRuntime,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireDefinition {
    tokens: Map<String, Value>,
    #[serde(default)]
    variants: Map<String, Value>,
    initial_variant: Option<String>,
    system_variants: Option<WireSystemVariants>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireSystemVariants {
    light: String,
    dark: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WirePatch {
    #[serde(default, deserialize_with = "present_variant")]
    variant: Option<Option<String>>,
    overrides: Option<Map<String, Value>>,
    remove_overrides: Option<Vec<String>>,
    system_scheme: Option<String>,
}

/// Distinguishes an explicit null variant from an omitted patch field.
///
/// `deserializer` reads the present JSON field and returns its optional name;
/// the outer `Some` records field presence, including an explicit null.
fn present_variant<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(Some)
}

impl ThemeBridge {
    /// Creates a typed theme from `definition_json` and returns `{id,snapshot}` JSON.
    ///
    /// # Errors
    /// Returns an error for malformed JSON, unsupported token types, invalid values,
    /// or an unknown initial/system variant.
    pub fn create(&mut self, definition_json: &str) -> Result<String, String> {
        let definition: WireDefinition =
            serde_json::from_str(definition_json).map_err(|error| error.to_string())?;
        if let Some(system) = &definition.system_variants {
            for name in [&system.light, &system.dark] {
                if !definition.variants.contains_key(name) {
                    return Err(format!("unknown system theme variant '{name}'"));
                }
            }
        }
        let mut tokens = Vec::with_capacity(definition.tokens.len());
        for (key, token) in definition.tokens {
            let token = token
                .as_object()
                .ok_or_else(|| format!("theme token '{key}' must be an object"))?;
            if let Some(field) = token
                .keys()
                .find(|field| !matches!(field.as_str(), "type" | "default" | "impact"))
            {
                return Err(format!("unknown field '{field}' in theme token '{key}'"));
            }
            let kind = token
                .get("type")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("theme token '{key}' needs a type"))?;
            let kind = parse_type(kind)?;
            let default = token
                .get("default")
                .ok_or_else(|| format!("theme token '{key}' needs a default"))?;
            let value = parse_value(kind, default)?;
            let mut entry = ThemeTokenDefinition::new(key, value);
            if let Some(impact) = token.get("impact") {
                entry = entry.impact(parse_impact(impact.as_str().ok_or("invalid impact")?)?);
            }
            tokens.push(entry);
        }
        let schema = Arc::new(ThemeSchema::new(tokens).map_err(|error| error.to_string())?);
        let runtime = ThemeRuntime::new(Arc::clone(&schema));
        for (name, values) in definition.variants {
            let values = values
                .as_object()
                .ok_or_else(|| format!("theme variant '{name}' must be an object"))?;
            let values = parse_token_values(&schema, values)?;
            runtime
                .define_variant(name, values)
                .map_err(|error| error.to_string())?;
        }
        let variants = definition.system_variants.unwrap_or(WireSystemVariants {
            light: "light".into(),
            dark: "dark".into(),
        });
        runtime.set_system_variants(variants.light.clone(), variants.dark.clone());
        runtime.set_system_scheme(self.system_scheme);
        let selected = definition.initial_variant.or_else(|| Some("system".into()));
        if let Some(name) = selected.as_deref() {
            if name == "system" {
                runtime.set_mode(ThemeMode::System);
            } else {
                runtime
                    .activate(name.to_owned())
                    .map_err(|error| error.to_string())?;
            }
        }
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or("theme session IDs exhausted")?;
        let id = self.next_id;
        self.sessions.insert(id, ThemeSession { runtime });
        let snapshot = self.snapshot_value(id, None)?;
        serde_json::to_string(&json!({ "id": id, "snapshot": snapshot }))
            .map_err(|error| error.to_string())
    }

    /// Applies `patch_json` to session `id` in one theme revision and returns its snapshot JSON.
    ///
    /// # Errors
    /// Returns an error for an unknown session, malformed patch, unknown token or
    /// variant, or a value that does not match its declared token type.
    pub fn update(&mut self, id: u64, patch_json: &str) -> Result<String, String> {
        let patch: WirePatch =
            serde_json::from_str(patch_json).map_err(|error| error.to_string())?;
        let session = self
            .sessions
            .get_mut(&id)
            .ok_or_else(|| format!("unknown theme session {id}"))?;
        let schema = session.runtime.schema();
        let overrides = patch
            .overrides
            .as_ref()
            .map(|values| parse_token_values(schema, values))
            .transpose()?;
        let removals = patch
            .remove_overrides
            .unwrap_or_default()
            .into_iter()
            .map(|key| {
                schema
                    .token(&key)
                    .ok_or_else(|| format!("unknown theme token '{key}'"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let system_scheme = patch
            .system_scheme
            .as_deref()
            .map(parse_scheme)
            .transpose()?;
        let variant = patch.variant;
        let change = session
            .runtime
            .update(|draft| {
                if let Some(scheme) = system_scheme {
                    draft.set_system_scheme(scheme);
                }
                if let Some(selection) = &variant {
                    match selection.as_deref() {
                        None => draft.clear_variant(),
                        Some("system") => draft.set_mode(ThemeMode::System),
                        Some(name) => draft.activate(name.to_owned())?,
                    }
                }
                if let Some(values) = overrides {
                    for (token, value) in values {
                        draft.set_override(token, value)?;
                    }
                }
                for token in removals {
                    draft.remove_override(token);
                }
                Ok(())
            })
            .map_err(|error| error.to_string())?;
        let snapshot = self.snapshot_value(id, Some(&change))?;
        serde_json::to_string(&snapshot).map_err(|error| error.to_string())
    }

    /// Returns a coherent snapshot for session `id`, including optional `change` metadata.
    ///
    /// # Errors
    /// Returns an error if `id` is unknown or a token cannot be represented on the wire.
    pub fn snapshot(&self, id: u64, change: Option<&ThemeChange>) -> Result<String, String> {
        serde_json::to_string(&self.snapshot_value(id, change)?).map_err(|error| error.to_string())
    }

    /// Returns a clone of the runtime for session `id`, if it exists.
    /// Mutations through this clone are visible to the bridge session.
    #[must_use]
    pub fn runtime(&self, id: u64) -> Option<ThemeRuntime> {
        self.sessions
            .get(&id)
            .map(|session| session.runtime.clone())
    }

    /// Subscribes to committed revisions for session `id`.
    /// The receiver yields changes after registration; hosts can call [`Self::snapshot`]
    /// with a received change to deliver a coherent JSON revision.
    ///
    /// # Errors
    /// Returns an error if `id` does not name a live theme session.
    pub fn subscribe(&self, id: u64) -> Result<std::sync::mpsc::Receiver<ThemeChange>, String> {
        self.sessions
            .get(&id)
            .map(|session| session.runtime.subscribe())
            .ok_or_else(|| format!("unknown theme session {id}"))
    }

    /// Applies the platform `scheme` to every live session and returns changed snapshots.
    /// Each theme commits one coherent revision; sessions whose scheme was already
    /// current are omitted from the returned `(id, snapshot_json)` list.
    ///
    /// # Errors
    /// Returns an error if a changed snapshot cannot be encoded on the wire.
    pub fn set_system_scheme(&mut self, scheme: ColorScheme) -> Result<Vec<(u64, String)>, String> {
        self.system_scheme = scheme;
        let mut updates = Vec::new();
        for (&id, session) in &self.sessions {
            let previous = session.runtime.revision();
            let change = session.runtime.set_system_scheme(scheme);
            if change.revision() != previous {
                updates.push((id, self.snapshot(id, Some(&change))?));
            }
        }
        updates.sort_unstable_by_key(|(id, _)| *id);
        Ok(updates)
    }

    /// Removes session `id` and its bridge-owned theme runtime.
    /// Returns whether the session existed.
    pub fn dispose(&mut self, id: u64) -> bool {
        self.sessions.remove(&id).is_some()
    }

    /// Converts session `id` and optional `change` into one JSON value.
    /// Returns an error if the session or a token value cannot be represented.
    fn snapshot_value(&self, id: u64, change: Option<&ThemeChange>) -> Result<Value, String> {
        let session = self
            .sessions
            .get(&id)
            .ok_or_else(|| format!("unknown theme session {id}"))?;
        let snapshot = session.runtime.snapshot();
        let mut values = Map::new();
        let mut revisions = Map::new();
        for (index, definition) in session.runtime.schema().definitions().iter().enumerate() {
            let token = session
                .runtime
                .schema()
                .token(definition.key())
                .ok_or_else(|| format!("missing token at index {index}"))?;
            let value = snapshot
                .value(token)
                .ok_or_else(|| format!("missing value for '{}'", definition.key()))?;
            values.insert(definition.key().into(), value_to_json(value)?);
            revisions.insert(
                definition.key().into(),
                json!(snapshot.token_revision(token).unwrap_or(0)),
            );
        }
        let selected = match snapshot.selection() {
            ThemeSelection::Defaults => None,
            ThemeSelection::System => Some("system"),
            ThemeSelection::Variant(name) => Some(name.as_str()),
        };
        let resolved = snapshot.resolved_variant();
        let change = change.map(|change| {
            let tokens = change
                .tokens()
                .iter()
                .filter_map(|id| session.runtime.schema().definition(*id))
                .map(|definition| definition.key())
                .collect::<Vec<_>>();
            json!({ "tokens": tokens, "impact": change.impact().map(impact_name) })
        });
        let mut result = json!({
            "revision": snapshot.revision(),
            "variant": selected,
            "resolvedVariant": resolved,
            "systemScheme": scheme_name(snapshot.system_scheme()),
            "values": values,
            "tokenRevisions": revisions,
        });
        if let Some(change) = change {
            result["change"] = change;
        }
        Ok(result)
    }
}

/// Resolves named `values` against `schema` into validated token IDs and typed values.
/// Returns an error for unknown names or incompatible wire values.
fn parse_token_values(
    schema: &ThemeSchema,
    values: &Map<String, Value>,
) -> Result<Vec<(argui_theme::ThemeTokenId, ThemeValue)>, String> {
    values
        .iter()
        .map(|(key, value)| {
            let id = schema
                .token(key)
                .ok_or_else(|| format!("unknown theme token '{key}'"))?;
            let kind = schema
                .definition(id)
                .expect("schema owns token ID")
                .value_type();
            Ok((id, parse_value(kind, value)?))
        })
        .collect()
}

/// Parses a supported scalar wire type from `name`, or returns an error.
fn parse_type(name: &str) -> Result<ThemeValueType, String> {
    match name {
        "Color" => Ok(ThemeValueType::Color),
        "Brush" => Ok(ThemeValueType::Brush),
        "Float" => Ok(ThemeValueType::Float),
        "Int" => Ok(ThemeValueType::Int),
        "Bool" => Ok(ThemeValueType::Bool),
        "Length" => Ok(ThemeValueType::Length),
        "Percentage" => Ok(ThemeValueType::Percentage),
        "Duration" => Ok(ThemeValueType::Duration),
        "Angle" => Ok(ThemeValueType::Angle),
        "FontFamily" => Ok(ThemeValueType::FontFamily),
        "FontWeight" => Ok(ThemeValueType::FontWeight),
        "FontSize" => Ok(ThemeValueType::FontSize),
        "LineHeight" => Ok(ThemeValueType::LineHeight),
        _ => Err(format!("unsupported theme token type '{name}'")),
    }
}

/// Decodes `value` as the typed `kind`, or returns an error for invalid input.
fn parse_value(kind: ThemeValueType, value: &Value) -> Result<ThemeValue, String> {
    let string = || value.as_str().ok_or("expected a string".to_owned());
    let number = || {
        value
            .as_f64()
            .filter(|value| {
                value.is_finite() && *value >= f64::from(f32::MIN) && *value <= f64::from(f32::MAX)
            })
            .map(|value| value as f32)
            .ok_or("expected a finite number".to_owned())
    };
    match kind {
        ThemeValueType::Color => Color::from_literal(string()?)
            .map(ThemeValue::Color)
            .map_err(|error| error.to_string()),
        ThemeValueType::Brush => Color::from_literal(string()?)
            .map(|color| ThemeValue::Brush(Fill::Solid(color)))
            .map_err(|error| error.to_string()),
        ThemeValueType::Float => Ok(ThemeValue::Float(number()?)),
        ThemeValueType::Int => value
            .as_i64()
            .map(ThemeValue::Int)
            .ok_or("expected an integer".into()),
        ThemeValueType::Bool => value
            .as_bool()
            .map(ThemeValue::Bool)
            .ok_or("expected a boolean".into()),
        ThemeValueType::Length => Ok(ThemeValue::Length(number()?)),
        ThemeValueType::Percentage => Ok(ThemeValue::Percentage(number()?)),
        ThemeValueType::Duration => Ok(ThemeValue::DurationMillis(number()?)),
        ThemeValueType::Angle => Ok(ThemeValue::AngleRadians(number()?)),
        ThemeValueType::FontFamily => Ok(ThemeValue::FontFamily(string()?.into())),
        ThemeValueType::FontWeight => value
            .as_u64()
            .and_then(|value| u16::try_from(value).ok())
            .map(ThemeValue::FontWeight)
            .ok_or("expected a font weight integer".into()),
        ThemeValueType::FontSize => Ok(ThemeValue::FontSize(number()?)),
        ThemeValueType::LineHeight => Ok(ThemeValue::LineHeight(number()?)),
        _ => Err(format!("unsupported wire value type {kind:?}")),
    }
}

/// Encodes a typed scalar `value`, or returns an error for unsupported shapes.
fn value_to_json(value: &ThemeValue) -> Result<Value, String> {
    match value {
        ThemeValue::Color(color) | ThemeValue::Brush(Fill::Solid(color)) => {
            let mut hex = color.to_hex_rgba();
            if hex.ends_with("ff") || hex.ends_with("FF") {
                hex.truncate(7);
            }
            Ok(json!(hex.to_ascii_lowercase()))
        }
        ThemeValue::Float(value)
        | ThemeValue::Length(value)
        | ThemeValue::Percentage(value)
        | ThemeValue::DurationMillis(value)
        | ThemeValue::AngleRadians(value)
        | ThemeValue::FontSize(value)
        | ThemeValue::LineHeight(value) => Ok(json!(value)),
        ThemeValue::Int(value) => Ok(json!(value)),
        ThemeValue::Bool(value) => Ok(json!(value)),
        ThemeValue::FontFamily(value) => Ok(json!(value)),
        ThemeValue::FontWeight(value) => Ok(json!(value)),
        _ => Err("theme value is not representable in the scalar wire format".into()),
    }
}

/// Parses invalidation impact `name`, or returns an error for an unknown class.
fn parse_impact(name: &str) -> Result<ThemeImpact, String> {
    match name {
        "Semantics" => Ok(ThemeImpact::Semantics),
        "Composite" => Ok(ThemeImpact::Composite),
        "Paint" => Ok(ThemeImpact::Paint),
        "Scroll" => Ok(ThemeImpact::Scroll),
        "Layout" => Ok(ThemeImpact::Layout),
        _ => Err(format!("unknown theme impact '{name}'")),
    }
}

/// Returns the wire name for `impact`.
fn impact_name(impact: ThemeImpact) -> &'static str {
    match impact {
        ThemeImpact::Semantics => "Semantics",
        ThemeImpact::Composite => "Composite",
        ThemeImpact::Paint => "Paint",
        ThemeImpact::Scroll => "Scroll",
        ThemeImpact::Layout => "Layout",
    }
}

/// Parses a system scheme `name`, or returns an error for an unknown scheme.
fn parse_scheme(name: &str) -> Result<ColorScheme, String> {
    match name {
        "light" => Ok(ColorScheme::Light),
        "dark" => Ok(ColorScheme::Dark),
        _ => Err(format!("unknown system color scheme '{name}'")),
    }
}

/// Returns the wire name for `scheme`.
fn scheme_name(scheme: ColorScheme) -> &'static str {
    match scheme {
        ColorScheme::Light => "light",
        ColorScheme::Dark => "dark",
    }
}
