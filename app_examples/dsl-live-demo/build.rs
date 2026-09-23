/// Generates the AOT version of the live demo for normal and release builds.
fn main() {
    argui_dsl_build::compile("ui/main.argui").expect("DSL live demo must compile");
    let manifest = std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    argui_dsl_build::compile_to(
        &manifest,
        std::path::Path::new("tests/fixtures/language.argui"),
        &output.join("conformance.rs"),
    )
    .expect("language conformance fixtures must compile to Rust");
}
