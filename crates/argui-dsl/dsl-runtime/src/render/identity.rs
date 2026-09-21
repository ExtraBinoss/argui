//! Stable identities shared by retained elements, children, and SVG animations.

use argui_dsl_ir::SiteId;

use crate::{DslValue, InstanceId, RuntimeError};

/// Derives a deterministic child instance from owner, site, and repeater key.
pub(super) fn child_instance_id(
    owner: InstanceId,
    site: SiteId,
    key: Option<&DslValue>,
) -> InstanceId {
    let mut value = owner.raw() ^ site.raw().rotate_left(17);
    if let Some(key) = key {
        value ^= key_hash(key).rotate_left(31);
    }
    InstanceId::from_raw(value.max(1))
}

/// Builds engine retained identity with the supported stable key classes.
///
/// # Errors
///
/// Returns a type mismatch if the repeater key is neither string nor integer.
pub(super) fn retained(
    owner: InstanceId,
    site: SiteId,
    key: Option<&DslValue>,
) -> Result<argui_ui::RetainedIdentity, RuntimeError> {
    let identity = argui_ui::RetainedIdentity::new(owner.raw(), site.raw());
    match key {
        None => Ok(identity),
        Some(DslValue::Int(value)) => Ok(identity.with_signed_key(*value)),
        Some(DslValue::String(value)) => Ok(identity.with_name_key(value.clone())),
        Some(value) => Err(RuntimeError::TypeMismatch {
            expected: "string or int repeater key".into(),
            actual: value.type_name().into(),
        }),
    }
}

/// Hashes supported repeater keys without source/runtime name lookup.
fn key_hash(value: &DslValue) -> u64 {
    match value {
        DslValue::Int(value) => *value as u64,
        DslValue::String(value) => value.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        }),
        _ => 0,
    }
}
