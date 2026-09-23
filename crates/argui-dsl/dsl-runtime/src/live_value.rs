//! Typed host values used by generated live component facades.

use argui_reactive::Model;

use crate::{DslValue, RuntimeError};

/// Converts Rust API values at the generated live boundary.
///
/// The generated public API mentions this trait only in its implementation;
/// callers work with the same Rust types as the compiled AOT component.
pub trait LiveValue: Sized {
    /// Converts an owned Rust value into its checked live representation.
    fn into_dsl(self) -> DslValue;

    /// Decodes a checked live value as this Rust type.
    ///
    /// `value` is the stored or callback value. Returns a type mismatch for an
    /// incompatible runtime value rather than leaking type-erased data to callers.
    ///
    /// # Errors
    ///
    /// Returns [`RuntimeError::TypeMismatch`] when conversion is impossible.
    fn from_dsl(value: DslValue) -> Result<Self, RuntimeError>;
}

/// Creates one boundary type mismatch using a stable public type label.
///
/// `expected` names the Rust-side type and `actual` is the live value.
/// Returns a structured runtime error.
fn mismatch(expected: &str, actual: &DslValue) -> RuntimeError {
    RuntimeError::TypeMismatch {
        expected: expected.into(),
        actual: actual.type_name().into(),
    }
}

impl LiveValue for () {
    fn into_dsl(self) -> DslValue {
        DslValue::Null
    }

    fn from_dsl(value: DslValue) -> Result<Self, RuntimeError> {
        match value {
            DslValue::Null => Ok(()),
            other => Err(mismatch("void", &other)),
        }
    }
}

impl LiveValue for bool {
    fn into_dsl(self) -> DslValue {
        DslValue::Bool(self)
    }

    fn from_dsl(value: DslValue) -> Result<Self, RuntimeError> {
        match value {
            DslValue::Bool(value) => Ok(value),
            other => Err(mismatch("bool", &other)),
        }
    }
}

impl LiveValue for i64 {
    fn into_dsl(self) -> DslValue {
        DslValue::Int(self)
    }

    fn from_dsl(value: DslValue) -> Result<Self, RuntimeError> {
        match value {
            DslValue::Int(value) => Ok(value),
            other => Err(mismatch("int", &other)),
        }
    }
}

impl LiveValue for f32 {
    fn into_dsl(self) -> DslValue {
        DslValue::Float(f64::from(self))
    }

    fn from_dsl(value: DslValue) -> Result<Self, RuntimeError> {
        match value {
            DslValue::Float(value) => Ok(value as f32),
            DslValue::Int(value) => Ok(value as f32),
            other => Err(mismatch("float", &other)),
        }
    }
}

impl LiveValue for String {
    fn into_dsl(self) -> DslValue {
        DslValue::String(self)
    }

    fn from_dsl(value: DslValue) -> Result<Self, RuntimeError> {
        match value {
            DslValue::String(value) => Ok(value),
            other => Err(mismatch("string", &other)),
        }
    }
}

impl LiveValue for argui_core::Color {
    fn into_dsl(self) -> DslValue {
        DslValue::Color(self)
    }

    fn from_dsl(value: DslValue) -> Result<Self, RuntimeError> {
        match value {
            DslValue::Color(value) => Ok(value),
            other => Err(mismatch("color", &other)),
        }
    }
}

impl LiveValue for argui_paint::Fill {
    fn into_dsl(self) -> DslValue {
        DslValue::Brush(self)
    }

    fn from_dsl(value: DslValue) -> Result<Self, RuntimeError> {
        match value {
            DslValue::Brush(value) => Ok(value),
            other => Err(mismatch("brush", &other)),
        }
    }
}

macro_rules! live_variant {
    ($type:ty, $variant:ident, $label:literal) => {
        impl LiveValue for $type {
            fn into_dsl(self) -> DslValue {
                DslValue::$variant(self)
            }

            fn from_dsl(value: DslValue) -> Result<Self, RuntimeError> {
                match value {
                    DslValue::$variant(value) => Ok(value),
                    other => Err(mismatch($label, &other)),
                }
            }
        }
    };
}

live_variant!(argui_ui::Dimension, Dimension, "dimension");
live_variant!(argui_core::Insets, Insets, "insets");
live_variant!(argui_paint::CornerRadii, Radii, "radii");
live_variant!(argui_paint::Border, Border, "border");
live_variant!(argui_paint::Shadow, Shadow, "shadow");
live_variant!(argui_core::Transform2D, Transform, "transform");
live_variant!(argui_assets::AssetHandle, AssetHandle, "asset");

impl<T: LiveValue> LiveValue for Option<T> {
    fn into_dsl(self) -> DslValue {
        self.map_or(DslValue::Null, LiveValue::into_dsl)
    }

    fn from_dsl(value: DslValue) -> Result<Self, RuntimeError> {
        match value {
            DslValue::Null => Ok(None),
            value => T::from_dsl(value).map(Some),
        }
    }
}

impl<T: LiveValue> LiveValue for Vec<T> {
    fn into_dsl(self) -> DslValue {
        DslValue::Array(self.into_iter().map(LiveValue::into_dsl).collect())
    }

    fn from_dsl(value: DslValue) -> Result<Self, RuntimeError> {
        match value {
            DslValue::Array(values) => values.into_iter().map(T::from_dsl).collect(),
            other => Err(mismatch("array", &other)),
        }
    }
}

impl<T: LiveValue + Clone> LiveValue for Model<T> {
    fn into_dsl(self) -> DslValue {
        DslValue::Array(self.into_iter().map(LiveValue::into_dsl).collect())
    }

    fn from_dsl(value: DslValue) -> Result<Self, RuntimeError> {
        Vec::<T>::from_dsl(value).map(Self::new)
    }
}
