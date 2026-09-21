use std::{collections::HashMap, fmt, sync::Arc};

use argui_core::Name;

use crate::{ThemeValue, ThemeValueType};

/// Dense token identity resolved once by the compiler or schema loader.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ThemeTokenId(u32);

impl ThemeTokenId {
    pub(crate) fn from_index(index: usize) -> Self {
        Self(index as u32)
    }

    /// Returns the dense zero-based token index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// Smallest engine update phase required by a changed token.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ThemeImpact {
    Semantics,
    Composite,
    Paint,
    Scroll,
    Layout,
}

#[derive(Clone, Debug)]
enum ThemeDefault {
    Value(ThemeValue),
    Reference(Name),
}

/// One typed token declaration before dense IDs and references are resolved.
#[derive(Clone, Debug)]
pub struct ThemeTokenDefinition {
    key: Name,
    value_type: ThemeValueType,
    default: ThemeDefault,
    impact: ThemeImpact,
}

impl ThemeTokenDefinition {
    /// Creates a token whose type is inferred from its concrete default value.
    ///
    /// * `key` — unique token name.
    /// * `default` — fallback value used without a variant or runtime override.
    #[must_use]
    pub fn new(key: impl Into<Name>, default: ThemeValue) -> Self {
        Self {
            key: key.into(),
            value_type: default.value_type(),
            default: ThemeDefault::Value(default),
            impact: ThemeImpact::Paint,
        }
    }

    /// Creates a derived token that resolves another token of the same type.
    ///
    /// * `key` — unique derived token name.
    /// * `value_type` — required type of the referenced token.
    /// * `source` — token name resolved by the schema.
    #[must_use]
    pub fn derived(
        key: impl Into<Name>,
        value_type: ThemeValueType,
        source: impl Into<Name>,
    ) -> Self {
        Self {
            key: key.into(),
            value_type,
            default: ThemeDefault::Reference(source.into()),
            impact: ThemeImpact::Paint,
        }
    }

    /// Sets the smallest update phase required by consumers of this token.
    ///
    /// * `impact` — invalidation class assigned to token changes.
    #[must_use]
    pub const fn impact(mut self, impact: ThemeImpact) -> Self {
        self.impact = impact;
        self
    }

    /// Returns the token name.
    #[must_use]
    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    /// Returns the declared token type.
    #[must_use]
    pub const fn value_type(&self) -> ThemeValueType {
        self.value_type
    }

    /// Returns the declared invalidation class.
    #[must_use]
    pub const fn impact_class(&self) -> ThemeImpact {
        self.impact
    }
}

/// Error returned while defining or mutating a typed theme.
#[derive(Clone, Debug, PartialEq)]
pub enum ThemeError {
    DuplicateToken(String),
    UnknownToken(String),
    UnknownVariant(String),
    InvalidValue(String),
    TypeMismatch {
        token: String,
        expected: ThemeValueType,
        actual: ThemeValueType,
    },
    ReferenceCycle(Vec<String>),
}

impl fmt::Display for ThemeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateToken(token) => write!(formatter, "duplicate theme token '{token}'"),
            Self::UnknownToken(token) => write!(formatter, "unknown theme token '{token}'"),
            Self::UnknownVariant(variant) => write!(formatter, "unknown theme variant '{variant}'"),
            Self::InvalidValue(token) => {
                write!(formatter, "invalid value for theme token '{token}'")
            }
            Self::TypeMismatch {
                token,
                expected,
                actual,
            } => write!(
                formatter,
                "theme token '{token}' expects {expected:?}, received {actual:?}"
            ),
            Self::ReferenceCycle(path) => {
                write!(formatter, "theme token cycle: {}", path.join(" -> "))
            }
        }
    }
}

impl std::error::Error for ThemeError {}

/// Validated immutable token schema with resolved defaults and references.
#[derive(Clone, Debug)]
pub struct ThemeSchema {
    definitions: Arc<[ThemeTokenDefinition]>,
    ids: HashMap<Name, ThemeTokenId>,
    defaults: Arc<[ThemeValue]>,
    references: Arc<[Option<ThemeTokenId>]>,
}

impl ThemeSchema {
    /// Validates definitions, resolves references, and assigns dense IDs.
    ///
    /// # Errors
    ///
    /// Returns an error for duplicate/unknown tokens, invalid defaults, type
    /// mismatches, or a derived-token cycle.
    ///
    /// * `definitions` — token declarations in deterministic schema order.
    pub fn new(
        definitions: impl IntoIterator<Item = ThemeTokenDefinition>,
    ) -> Result<Self, ThemeError> {
        let definitions = definitions.into_iter().collect::<Vec<_>>();
        let mut ids = HashMap::with_capacity(definitions.len());
        for (index, definition) in definitions.iter().enumerate() {
            let id = ThemeTokenId(index as u32);
            if ids.insert(definition.key.clone(), id).is_some() {
                return Err(ThemeError::DuplicateToken(definition.key().to_owned()));
            }
            if let ThemeDefault::Value(value) = &definition.default {
                validate_value(definition, value)?;
            }
        }
        let mut references = vec![None; definitions.len()];
        for (index, definition) in definitions.iter().enumerate() {
            if let ThemeDefault::Reference(source) = &definition.default {
                let source_id = ids
                    .get(source)
                    .copied()
                    .ok_or_else(|| ThemeError::UnknownToken(source.as_str().to_owned()))?;
                let source_definition = &definitions[source_id.index()];
                if source_definition.value_type != definition.value_type {
                    return Err(ThemeError::TypeMismatch {
                        token: definition.key().to_owned(),
                        expected: definition.value_type,
                        actual: source_definition.value_type,
                    });
                }
                references[index] = Some(source_id);
            }
        }
        let defaults = resolve_defaults(&definitions, &references)?;
        Ok(Self {
            definitions: definitions.into(),
            ids,
            defaults: defaults.into(),
            references: references.into(),
        })
    }

    /// Returns the dense ID assigned to `key`.
    #[must_use]
    pub fn token(&self, key: &str) -> Option<ThemeTokenId> {
        self.ids.get(key).copied()
    }

    /// Returns the definition assigned to `id`.
    #[must_use]
    pub fn definition(&self, id: ThemeTokenId) -> Option<&ThemeTokenDefinition> {
        self.definitions.get(id.index())
    }

    /// Returns the resolved default value assigned to `id`.
    #[must_use]
    pub fn default_value(&self, id: ThemeTokenId) -> Option<&ThemeValue> {
        self.defaults.get(id.index())
    }

    /// Returns token definitions in deterministic ID order.
    #[must_use]
    pub fn definitions(&self) -> &[ThemeTokenDefinition] {
        &self.definitions
    }

    /// Returns the number of tokens in this schema.
    #[must_use]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Returns whether this schema contains no tokens.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    pub(crate) fn reference(&self, id: ThemeTokenId) -> Option<ThemeTokenId> {
        self.references.get(id.index()).copied().flatten()
    }

    pub(crate) fn validate(&self, id: ThemeTokenId, value: &ThemeValue) -> Result<(), ThemeError> {
        let definition = self
            .definition(id)
            .ok_or_else(|| ThemeError::UnknownToken(format!("#{}", id.index())))?;
        validate_value(definition, value)
    }
}

fn validate_value(definition: &ThemeTokenDefinition, value: &ThemeValue) -> Result<(), ThemeError> {
    if value.value_type() != definition.value_type {
        return Err(ThemeError::TypeMismatch {
            token: definition.key().to_owned(),
            expected: definition.value_type,
            actual: value.value_type(),
        });
    }
    if !value.is_valid() {
        return Err(ThemeError::InvalidValue(definition.key().to_owned()));
    }
    Ok(())
}

fn resolve_defaults(
    definitions: &[ThemeTokenDefinition],
    references: &[Option<ThemeTokenId>],
) -> Result<Vec<ThemeValue>, ThemeError> {
    fn resolve(
        index: usize,
        definitions: &[ThemeTokenDefinition],
        references: &[Option<ThemeTokenId>],
        states: &mut [u8],
        values: &mut [Option<ThemeValue>],
        stack: &mut Vec<usize>,
    ) -> Result<ThemeValue, ThemeError> {
        if let Some(value) = &values[index] {
            return Ok(value.clone());
        }
        if states[index] == 1 {
            let start = stack
                .iter()
                .position(|candidate| *candidate == index)
                .unwrap_or(0);
            let path = stack[start..]
                .iter()
                .copied()
                .chain(std::iter::once(index))
                .map(|candidate| definitions[candidate].key().to_owned())
                .collect();
            return Err(ThemeError::ReferenceCycle(path));
        }
        states[index] = 1;
        stack.push(index);
        let value = match (&definitions[index].default, references[index]) {
            (ThemeDefault::Value(value), _) => value.clone(),
            (ThemeDefault::Reference(_), Some(source)) => resolve(
                source.index(),
                definitions,
                references,
                states,
                values,
                stack,
            )?,
            (ThemeDefault::Reference(source), None) => {
                return Err(ThemeError::UnknownToken(source.as_str().to_owned()));
            }
        };
        stack.pop();
        states[index] = 2;
        values[index] = Some(value.clone());
        Ok(value)
    }

    let mut states = vec![0; definitions.len()];
    let mut values = vec![None; definitions.len()];
    for index in 0..definitions.len() {
        resolve(
            index,
            definitions,
            references,
            &mut states,
            &mut values,
            &mut Vec::new(),
        )?;
    }
    Ok(values.into_iter().flatten().collect())
}
