#![cfg(test)]

use crate::source_cache;

#[test]
/// URL inputs reject traversal, malformed versions, and invalid checksums before network access.
fn release_source_inputs_are_validated_before_fetch() {
    let valid_hash = "a".repeat(64);
    let long_version = "1".repeat(41);
    for version in ["", "0/3", "A.1", long_version.as_str()] {
        assert!(source_cache::registry(version).is_err());
    }
    for path in [
        "",
        "../escape.tsx",
        "solid/BAD.tsx",
        "solid/a.js",
        "/x.tsx",
        "solid/a?.tsx",
    ] {
        assert!(source_cache::file("0.3.3", path, &valid_hash).is_err());
    }
    let uppercase_hash = "A".repeat(64);
    for hash in ["abc", uppercase_hash.as_str()] {
        assert!(source_cache::file("0.3.3", "solid/button.tsx", hash).is_err());
    }
}
