/// Generates the gallery UI for AOT builds and skips it for the live-only client.
fn main() {
    println!("cargo:rerun-if-env-changed=ARGUI_DEV_CLIENT");
    println!("cargo:rustc-check-cfg=cfg(argui_dev_client)");
    argui_dsl_build::compile("ui/main.argui").expect("DSL widget gallery must compile");
}
