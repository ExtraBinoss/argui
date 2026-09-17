/// Launches the native GPU Canvas Lab and propagates startup failures.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    argui_example_gpu_canvas::launch()
}
