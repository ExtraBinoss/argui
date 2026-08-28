fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let host_is_windows = std::env::var("HOST").is_ok_and(|host| host.contains("windows"));
    let is_release = std::env::var("PROFILE").as_deref() == Ok("release");
    if !host_is_windows && !is_release {
        println!(
            "cargo:warning=skipping the Windows icon during a cross-target debug check; \
             release packaging still requires windres"
        );
        return;
    }

    winresource::WindowsResource::new()
        .set_icon("assets/icon.ico")
        .compile()
        .expect(
            "failed to embed the Windows application icon; install windres when cross-compiling",
        );
}
