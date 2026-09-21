/// Generates the AOT version of the live demo for normal and release builds.
fn main() {
    argui_dsl_build::compile("ui/main.argui").expect("DSL live demo must compile");
}
