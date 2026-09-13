use fluent_bundle::{FluentBundle, FluentError, FluentResource};
use unic_langid::LanguageIdentifier;

/// One locale's immutable collection of Fluent resources.
pub struct Catalog {
    pub(crate) locale: LanguageIdentifier,
    pub(crate) bundle: FluentBundle<FluentResource>,
}

impl Catalog {
    /// Parses one Fluent resource for `locale`.
    pub fn parse(
        locale: LanguageIdentifier,
        source: impl Into<String>,
    ) -> Result<Self, CatalogError> {
        Self::from_resources(locale, [source.into()])
    }

    /// Parses and combines several Fluent resources for one locale.
    ///
    /// Message and term identifiers must be unique across the resources.
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
    pub const fn locale(&self) -> &LanguageIdentifier {
        &self.locale
    }
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
    pub const fn locale(&self) -> &LanguageIdentifier {
        &self.locale
    }

    #[must_use]
    pub fn problems(&self) -> &[FluentError] {
        &self.problems
    }
}
