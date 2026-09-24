use fluent_bundle::{FluentBundle, FluentError, FluentResource};
use std::collections::BTreeMap;
use unic_langid::LanguageIdentifier;

/// One locale's immutable collection of Fluent resources.
pub struct Catalog {
    pub(crate) locale: LanguageIdentifier,
    pub(crate) bundle: FluentBundle<FluentResource>,
}

impl Catalog {
    /// Parses a flat JSON object whose string values are Fluent message patterns.
    ///
    /// * `locale` — language identifier associated with the JSON catalog.
    /// * `source` — JSON object mapping Fluent message IDs to pattern strings.
    ///
    /// # Errors
    /// Returns an error for invalid JSON, unsupported message IDs, or invalid
    /// Fluent patterns. Message IDs must start with an ASCII letter and contain
    /// only ASCII letters, digits, underscores, or hyphens.
    pub fn from_json(locale: LanguageIdentifier, source: &str) -> Result<Self, JsonCatalogError> {
        let messages: BTreeMap<String, String> = serde_json::from_str(source)
            .map_err(|error| JsonCatalogError::InvalidJson(error.to_string()))?;
        let mut fluent = String::new();
        for (id, pattern) in messages {
            let mut characters = id.chars();
            if !characters
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic())
                || !characters.all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
                })
            {
                return Err(JsonCatalogError::InvalidId(id));
            }
            let pattern = pattern.replace("\r\n", "\n").replace('\r', "\n");
            let pattern = if pattern.is_empty() {
                "{ \"\" }"
            } else {
                &pattern
            };
            fluent.push_str(&id);
            fluent.push_str(" = ");
            fluent.push_str(&pattern.replace('\n', "\n    "));
            fluent.push('\n');
        }
        Self::parse(locale, fluent).map_err(JsonCatalogError::Fluent)
    }

    /// Parses one Fluent resource for `locale`.
    ///
    /// # Arguments
    /// * `locale` — language identifier associated with the resource.
    /// * `source` — Fluent source text.
    ///
    /// # Errors
    /// Returns an error if the source has invalid Fluent syntax or duplicate entries.
    pub fn parse(
        locale: LanguageIdentifier,
        source: impl Into<String>,
    ) -> Result<Self, CatalogError> {
        Self::from_resources(locale, [source.into()])
    }

    /// Parses and combines several Fluent resources for one locale.
    ///
    /// Message and term identifiers must be unique across the resources.
    ///
    /// # Arguments
    /// * `locale` — language identifier shared by all resources.
    /// * `sources` — Fluent source texts to parse and combine.
    ///
    /// # Errors
    /// Returns an error if a resource has invalid syntax or an identifier duplicates one already
    /// present in the combined catalog.
    pub fn from_resources<S, I>(
        locale: LanguageIdentifier,
        sources: I,
    ) -> Result<Self, CatalogError>
    where
        S: Into<String>,
        I: IntoIterator<Item = S>,
    {
        let mut bundle = FluentBundle::new(vec![locale.clone()]);
        for source in sources {
            let resource = FluentResource::try_new(source.into()).map_err(|(_, problems)| {
                CatalogError::new(
                    locale.clone(),
                    problems.into_iter().map(FluentError::from).collect(),
                )
            })?;
            bundle
                .add_resource(resource)
                .map_err(|problems| CatalogError::new(locale.clone(), problems))?;
        }
        Ok(Self { locale, bundle })
    }

    /// Locale represented by this catalog.
    #[must_use]
    /// Returns the language identifier represented by this catalog.
    pub const fn locale(&self) -> &LanguageIdentifier {
        &self.locale
    }
}

/// Invalid JSON, unsupported message ID, or Fluent syntax in a JSON catalog.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum JsonCatalogError {
    #[error("invalid JSON catalog: {0}")]
    InvalidJson(String),
    #[error("invalid Fluent message ID `{0}` in JSON catalog")]
    InvalidId(String),
    #[error(transparent)]
    Fluent(#[from] CatalogError),
}

impl std::fmt::Debug for Catalog {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Catalog")
            .field("locale", &self.locale)
            .finish_non_exhaustive()
    }
}

/// Fluent syntax or duplicate-entry errors in a catalog.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("invalid Fluent catalog for {locale}")]
pub struct CatalogError {
    locale: LanguageIdentifier,
    problems: Vec<FluentError>,
}

impl CatalogError {
    fn new(locale: LanguageIdentifier, problems: Vec<FluentError>) -> Self {
        Self { locale, problems }
    }

    #[must_use]
    /// Returns the locale associated with the catalog error.
    pub const fn locale(&self) -> &LanguageIdentifier {
        &self.locale
    }

    #[must_use]
    /// Returns the Fluent syntax or duplicate-entry problems collected for this error.
    pub fn problems(&self) -> &[FluentError] {
        &self.problems
    }
}
