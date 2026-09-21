macro_rules! stable_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name(u64);

        impl $name {
            /// Creates an ID from its stable wire representation.
            #[must_use]
            pub const fn from_raw(raw: u64) -> Self {
                Self(raw)
            }

            /// Returns the stable wire representation.
            #[must_use]
            pub const fn raw(self) -> u64 {
                self.0
            }
        }
    };
}

stable_id!(ModuleId, "Stable canonical module identity.");
stable_id!(ComponentId, "Stable component definition identity.");
stable_id!(PropertyId, "Stable component property identity.");
stable_id!(CallbackId, "Stable component callback identity.");
stable_id!(SlotId, "Stable component slot identity.");
stable_id!(FieldId, "Stable user-struct field identity.");
stable_id!(VariantId, "Stable user-enum variant identity.");
stable_id!(ThemeId, "Stable theme definition identity.");
stable_id!(TokenId, "Stable theme-token identity.");
stable_id!(ThemeModeId, "Stable named theme-mode identity.");

impl ThemeModeId {
    /// Derives the shared identity of a mode name across all theme declarations.
    ///
    /// * `name` — the mode spelling used in a theme or by `set_theme_mode()`.
    ///
    /// Returns the stable ID used for matching all overrides of this name.
    #[must_use]
    pub fn named(name: &str) -> Self {
        Self(hash_text(name))
    }
}
stable_id!(StyleId, "Stable named-style identity.");
stable_id!(StyleStateId, "Stable named style-state identity.");
stable_id!(EffectId, "Stable DSL effect definition identity.");
stable_id!(AssetId, "Stable imported asset identity.");
stable_id!(SiteId, "Stable source-site identity within a component.");
stable_id!(
    ExpressionId,
    "Explicit typed-expression identity within an IR package."
);
stable_id!(LocalId, "Stable repeater/event local identity.");
stable_id!(AnimationId, "Stable component animation identity.");

/// Fully resolved property destination for native and DSL components.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PropertyTargetId {
    Native(argui_schema::PropertyId),
    Component(PropertyId),
}

/// Fully resolved event destination for native and DSL components.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EventTargetId {
    Native(argui_schema::EventId),
    Component(CallbackId),
}

pub(crate) fn derive(namespace: u64, discriminator: &str, ordinal: u64) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in namespace
        .to_le_bytes()
        .into_iter()
        .chain(discriminator.bytes())
        .chain(ordinal.to_le_bytes())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

pub(crate) fn hash_text(text: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}
