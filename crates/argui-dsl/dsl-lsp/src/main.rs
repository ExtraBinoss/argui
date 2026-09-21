/// Runs the Argui language server over standard input and output.
fn main() {
    if let Err(error) = argui_dsl_lsp::run_stdio() {
        eprintln!("argui-lsp: {error}");
        std::process::exit(1);
    }
}
