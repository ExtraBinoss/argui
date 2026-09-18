use argui_platform::WindowConfig;

/// Runs the model-free launch and close integration scenario.
pub fn run() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if std::env::var_os("ARGUI_MODEL_FREE_TEST_CHILD").is_none() {
        let success = root
            .join("target/native-launch")
            .join(format!("success-{}", std::process::id()));
        std::fs::create_dir_all(success.parent().unwrap()).unwrap();
        let _ = std::fs::remove_file(&success);
        let status = std::process::Command::new(root.join("scripts/linux-hidden-display.sh"))
            .env("ARGUI_TEST_BACKEND", "x11")
            .env("ARGUI_MODEL_FREE_TEST_CHILD", "1")
            .env("ARGUI_MODEL_FREE_TEST_SUCCESS", &success)
            .current_dir(&root)
            .arg("timeout")
            .arg("20s")
            .arg(std::env::current_exe().unwrap())
            .status()
            .unwrap();
        assert!(
            status.success() || success.exists(),
            "hidden model-free launch test failed"
        );
        let _ = std::fs::remove_file(success);
        return;
    }
    let driver = std::thread::spawn(move || {
        std::process::Command::new("python3")
            .arg(root.join("crates/argui-runtime/tests/launch/close.py"))
            .current_dir(root)
            .status()
            .unwrap()
    });
    argui_runtime::run(
        WindowConfig {
            title: "Argui model-free launch test".into(),
            ..Default::default()
        },
        Default::default(),
        |_| {},
    )
    .unwrap();
    assert!(driver.join().unwrap().success());
    std::fs::write(
        std::env::var_os("ARGUI_MODEL_FREE_TEST_SUCCESS").unwrap(),
        "passed",
    )
    .unwrap();
}
