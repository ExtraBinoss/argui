macro_rules! numeric_id {
    ($name:ident, $raw:ty, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub struct $name($raw);

        impl $name {
            /// Creates an identifier from its stable wire representation.
            ///
            /// * `raw` — schema-defined numeric value.
            #[must_use]
            pub const fn from_raw(raw: $raw) -> Self {
                Self(raw)
            }

            /// Returns the stable numeric representation.
            #[must_use]
            pub const fn raw(self) -> $raw {
                self.0
            }
        }
    };
}

numeric_id!(
    NativeTypeId,
    u32,
    "Stable identifier for a native primitive or behavior."
);
numeric_id!(
    PropertyId,
    u16,
    "Stable property identifier within a native schema."
);
numeric_id!(
    EventId,
    u16,
    "Stable event identifier within a native schema."
);
numeric_id!(
    SlotId,
    u16,
    "Stable child-slot identifier within a native schema."
);
numeric_id!(
    VariantId,
    u16,
    "Stable variant identifier within a native schema."
);
numeric_id!(
    StylePartId,
    u16,
    "Stable style-part identifier within a native schema."
);
