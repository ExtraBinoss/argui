/// Generates the standalone virtual-list UI from its DSL source.
fn main() {
    argui_dsl_build::compile("ui/main.argui").expect("virtual-list DSL example must compile");
}
