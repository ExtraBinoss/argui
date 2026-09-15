use fluent_bundle::{FluentArgs, FluentError};
use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use unic_langid::{CharacterDirection, LanguageIdentifier};

use crate::Catalog;

/// Fluent catalogs plus the application's currently negotiated locale chain.
#[derive(Debug)]
pub struct Localizer {
    fallback: LanguageIdentifier,
    available: Vec<LanguageIdentifier>,
    catalogs: Vec<Catalog>,
    active: Vec<usize>,
}

impl Localizer {
    /// Creates a localizer whose fallback catalog is always searched last.
    ///
    /// # Arguments
    /// * `fallback` — locale required as the final lookup fallback.
    /// * `catalogs` — catalogs available for negotiation and lookup.
    ///
    /// # Errors
    /// Returns an error if a locale occurs more than once or no catalog matches `fallback`.
    pub fn new(
        fallback: LanguageIdentifier,
        catalogs: impl IntoIterator<Item = Catalog>,
    ) -> Result<Self, LocalizerError> {
        let catalogs: Vec<_> = catalogs.into_iter().collect();
        for (index, catalog) in catalogs.iter().enumerate() {
            if catalogs[..index]
                .iter()
                .any(|existing| existing.locale == catalog.locale)
            {
                return Err(LocalizerError::DuplicateLocale(catalog.locale.clone()));
            }
        }
        let Some(fallback_index) = catalogs
            .iter()
            .position(|catalog| catalog.locale == fallback)
        else {
            return Err(LocalizerError::MissingFallback(fallback));
        };
        let available = catalogs
            .iter()
            .map(|catalog| catalog.locale.clone())
            .collect();
        Ok(Self {
            fallback,
            available,
            catalogs,
            active: vec![fallback_index],
        })
    }

    /// Negotiates preferred locales in order and returns whether visible output may change.
    ///
    /// # Arguments
    /// * `requested` — preferred locales, ordered by preference.
    pub fn select(&mut self, requested: impl IntoIterator<Item = LanguageIdentifier>) -> bool {
        let requested: Vec<_> = requested.into_iter().collect();
        let negotiated = negotiate_languages(
            &requested,
            &self.available,
            Some(&self.fallback),
            NegotiationStrategy::Filtering,
        );
        let active: Vec<_> = negotiated
            .into_iter()
            .filter_map(|locale| {
                self.available
                    .iter()
                    .position(|available| available == locale)
            })
            .collect();
        if self.active == active {
            return false;
        }
        self.active = active;
        true
    }

    /// Primary negotiated locale. This is the fallback before the first selection.
    #[must_use]
    pub fn locale(&self) -> &LanguageIdentifier {
        &self.available[self.active[0]]
    }

    /// Negotiated locales in lookup order, including the fallback.
    pub fn locales(&self) -> impl ExactSizeIterator<Item = &LanguageIdentifier> {
        self.active.iter().map(|index| &self.available[*index])
    }

    #[must_use]
    /// Returns the locale used as the final lookup fallback.
    pub const fn fallback_locale(&self) -> &LanguageIdentifier {
        &self.fallback
    }

    /// Unicode character direction inferred from the primary locale and explicit script.
    #[must_use]
    pub fn direction(&self) -> CharacterDirection {
        self.locale().character_direction()
    }

    #[must_use]
    /// Returns whether the primary negotiated locale uses right-to-left character direction.
    pub fn is_rtl(&self) -> bool {
        self.direction() == CharacterDirection::RTL
    }

    /// Formats a message without variables.
    ///
    /// # Arguments
    /// * `id` — Fluent message identifier.
    ///
    /// # Errors
    /// Returns an error if no matching message/value exists or Fluent formatting reports problems.
    pub fn text(&self, id: &str) -> Result<String, TranslationError> {
        self.format_optional(id, None)
    }

    /// Formats a message with Fluent variables, selectors, terms, and built-ins.
    ///
    /// # Arguments
    /// * `id` — Fluent message identifier.
    /// * `args` — variables referenced while formatting the message.
    ///
    /// # Errors
    /// Returns an error if no matching message/value exists or Fluent formatting reports problems.
    pub fn format(&self, id: &str, args: &FluentArgs<'_>) -> Result<String, TranslationError> {
        self.format_optional(id, Some(args))
    }

    fn format_optional(
        &self,
        id: &str,
        args: Option<&FluentArgs<'_>>,
    ) -> Result<String, TranslationError> {
        let mut found = false;
        for index in &self.active {
            let catalog = &self.catalogs[*index];
            let Some(message) = catalog.bundle.get_message(id) else {
                continue;
            };
            found = true;
            let Some(pattern) = message.value() else {
                continue;
            };
            let mut problems = Vec::new();
            let text = catalog
                .bundle
                .format_pattern(pattern, args, &mut problems)
                .into_owned();
            if problems.is_empty() {
                return Ok(text);
            }
            return Err(TranslationError::Formatting {
                locale: catalog.locale.clone(),
                id: id.into(),
                problems,
            });
        }
        if found {
            Err(TranslationError::MissingValue(id.into()))
        } else {
            Err(TranslationError::MissingMessage(id.into()))
        }
    }

    /// Formats a message attribute without variables.
    ///
    /// # Arguments
    /// * `id` — Fluent message identifier.
    /// * `attribute` — attribute name on that message.
    ///
    /// # Errors
    /// Returns an error if the message or attribute is absent, or formatting reports problems.
    pub fn attribute(&self, id: &str, attribute: &str) -> Result<String, TranslationError> {
        self.format_attribute_optional(id, attribute, None)
    }

    /// Formats a message attribute with Fluent variables.
    ///
    /// # Arguments
    /// * `id` — Fluent message identifier.
    /// * `attribute` — attribute name on that message.
    /// * `args` — variables referenced while formatting the attribute.
    ///
    /// # Errors
    /// Returns an error if the message or attribute is absent, or formatting reports problems.
    pub fn format_attribute(
        &self,
        id: &str,
        attribute: &str,
        args: &FluentArgs<'_>,
    ) -> Result<String, TranslationError> {
        self.format_attribute_optional(id, attribute, Some(args))
    }

    fn format_attribute_optional(
        &self,
        id: &str,
        attribute: &str,
        args: Option<&FluentArgs<'_>>,
    ) -> Result<String, TranslationError> {
        let mut found = false;
        for index in &self.active {
            let catalog = &self.catalogs[*index];
            let Some(message) = catalog.bundle.get_message(id) else {
                continue;
            };
            found = true;
            let Some(value) = message.get_attribute(attribute) else {
                continue;
            };
            let mut problems = Vec::new();
            let text = catalog
                .bundle
                .format_pattern(value.value(), args, &mut problems)
                .into_owned();
            if problems.is_empty() {
                return Ok(text);
            }
            return Err(TranslationError::Formatting {
                locale: catalog.locale.clone(),
                id: id.into(),
                problems,
            });
        }
        if found {
            Err(TranslationError::MissingAttribute {
                id: id.into(),
                attribute: attribute.into(),
            })
        } else {
            Err(TranslationError::MissingMessage(id.into()))
        }
    }
}

/// Invalid catalog configuration.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum LocalizerError {
    #[error("no Fluent catalog exists for fallback locale {0}")]
    MissingFallback(LanguageIdentifier),
    #[error("more than one Fluent catalog exists for locale {0}")]
    DuplicateLocale(LanguageIdentifier),
}

/// A requested message could not produce safe localized text.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum TranslationError {
    #[error("Fluent message `{0}` was not found")]
    MissingMessage(String),
    #[error("Fluent message `{0}` has no value")]
    MissingValue(String),
    #[error("Fluent message `{id}` has no `{attribute}` attribute")]
    MissingAttribute { id: String, attribute: String },
    #[error("Fluent message `{id}` could not be formatted for {locale}")]
    Formatting {
        locale: LanguageIdentifier,
        id: String,
        problems: Vec<FluentError>,
    },
}
