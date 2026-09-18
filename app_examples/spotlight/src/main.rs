/// Launches the native Spotlight example and propagates startup failures.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    argui_example_spotlight::launch()
}
