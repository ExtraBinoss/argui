/// Generates the gallery's release UI from the shared DSL source tree.
fn main() {
    argui_dsl_build::compile("ui/main.argui").expect("DSL widget gallery must compile");
}
