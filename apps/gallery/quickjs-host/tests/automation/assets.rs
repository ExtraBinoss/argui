use super::*;
use serde_json::json;
use std::path::PathBuf;

struct Fixture(PathBuf);

impl Fixture {
    /// Creates an isolated application asset root for one manifest test.
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("argui-app-assets-{}-{nonce}", std::process::id()));
        fs::create_dir_all(root.join("assets")).unwrap();
        Self(root)
    }

    /// Writes a generated manifest with the supplied sources.
    fn manifest(&self, sources: Value) -> PathBuf {
        let path = self.0.join("assets.generated.json");
        fs::write(
            &path,
            serde_json::to_vec(&json!({"assets": sources})).unwrap(),
        )
        .unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn application_manifest_decodes_vectors_and_accepts_an_empty_catalog() {
    let fixture = Fixture::new();
    let empty = load(&fixture.manifest(json!([]))).unwrap();
    assert!(empty.vectors.is_empty() && empty.images.is_empty());
    fs::write(fixture.0.join("assets/icon.svg"), r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><rect width="16" height="16" fill="red"/></svg>"#).unwrap();
    let manifest =
        fixture.manifest(json!([{"key":"icon", "path":"icon.svg", "id":42, "kind":"svg"}]));
    let assets = load(&manifest).unwrap();
    assert_eq!(assets.vectors.len(), 1);
    assert_eq!(assets.vectors[0].id.0, 42);
}

#[test]
fn application_manifest_rejects_invalid_ids_kinds_and_escaping_paths() {
    let fixture = Fixture::new();
    for (id, kind, path, reason) in [
        (0_u64, "svg", "icon.svg", "invalid asset ID"),
        (1_u64 << 53, "svg", "icon.svg", "invalid asset ID"),
        (1, "script", "icon.svg", "unsupported asset kind"),
        (1, "svg", "../icon.svg", "unsafe asset path"),
        (1, "svg", "/icon.svg", "unsafe asset path"),
    ] {
        let manifest = fixture.manifest(json!([{"key":"icon", "path":path, "id":id, "kind":kind}]));
        assert!(load(&manifest).unwrap_err().contains(reason));
    }
}

#[cfg(unix)]
#[test]
fn application_manifest_rejects_symlinks_leaving_its_asset_root() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("outside.svg"), b"invalid svg").unwrap();
    std::os::unix::fs::symlink(
        fixture.0.join("outside.svg"),
        fixture.0.join("assets/link.svg"),
    )
    .unwrap();
    let manifest =
        fixture.manifest(json!([{"key":"icon", "path":"link.svg", "id":1, "kind":"svg"}]));
    assert!(
        load(&manifest)
            .unwrap_err()
            .contains("asset escapes app root")
    );
}
