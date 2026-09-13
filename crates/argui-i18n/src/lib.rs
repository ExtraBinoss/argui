//! Fluent localization with negotiated locale fallback for Argui applications.
//!
//! `argui-i18n` has no renderer, runtime, or widget dependency. Keep a [`Localizer`]
//! in application state, select the user's preferred locales, and pass its output
//! to ordinary Argui elements and widget labels.
//!
//! ```
//! use argui_i18n::{Catalog, FluentArgs, Localizer, langid};
//!
//! let english = Catalog::parse(
//!     langid!("en-US"),
//!     "greeting = Hello, { $name }!\nitems = { $count ->\n    [one] One item\n   *[other] { $count } items\n}",
//! )?;
//! let french = Catalog::parse(
//!     langid!("fr"),
//!     "greeting = Bonjour, { $name } !\nitems = { $count ->\n    [one] Un élément\n   *[other] { $count } éléments\n}",
//! )?;
//! let mut localizer = Localizer::new(langid!("en-US"), [english, french])?;
//! localizer.select([langid!("fr-CA")]);
//!
//! let mut args = FluentArgs::new();
//! args.set("name", "Ada");
//! assert_eq!(
//!     localizer.format("greeting", &args)?,
//!     "Bonjour, \u{2068}Ada\u{2069} !",
//! );
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod catalog;
mod localizer;

pub use catalog::{Catalog, CatalogError};
pub use fluent_bundle::{FluentArgs, FluentError, FluentValue};
pub use localizer::{Localizer, LocalizerError, TranslationError};
pub use unic_langid::{CharacterDirection, LanguageIdentifier, langid};
