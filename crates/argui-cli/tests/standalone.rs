#![cfg(not(target_arch = "wasm32"))]

use argui_cli::run_in;
use serde_json::Value;
use std::{fs, path::Path, process::Command};

#[path = "standalone/build.rs"]
mod build;
#[path = "standalone/init.rs"]
mod init;
#[path = "standalone/menu.rs"]
mod menu;

/// Runs a CLI command against one isolated directory.
fn run(path: &Path, args: &[&str]) -> Result<(), String> {
    let mut arguments = args
        .iter()
        .map(|argument| (*argument).to_owned())
        .collect::<Vec<_>>();
    if args.first() == Some(&"init") && args.contains(&"--yes") {
        arguments.push("--no-install".into());
    }
    run_in(path, &arguments)
}

/// Parses the generated app manifest after a successful init.
fn manifest(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path.join("argui.json")).unwrap()).unwrap()
}

#[test]
/// Rust and TSX apps are generated directly with pinned, simple manifests.
fn init_generates_three_standalone_frameworks() {
    let root = tempfile::tempdir().unwrap();
    for framework in ["rust", "solid", "react"] {
        let app = root.path().join(framework);
        run(
            root.path(),
            &["init", framework, "--dir", framework, "--yes"],
        )
        .unwrap();
        let state = manifest(&app);
        assert_eq!(state["projectVersion"], 2);
        assert_eq!(state["framework"], framework);
        assert_eq!(state["arguiVersion"], env!("CARGO_PKG_VERSION"));
        assert_eq!(state["sdkSha256"].as_str().unwrap().len(), 64);
        assert_eq!(state["distribution"], "embedded-development");
        assert!(!app.join("sdk").exists());
        assert!(!app.join("hosts").exists());
        if framework == "rust" {
            assert_eq!(state["targets"], serde_json::json!(["native"]));
            let cargo = fs::read_to_string(app.join("Cargo.toml")).unwrap();
            assert!(cargo.contains("argui-runtime"));
            assert!(!cargo.contains("argui-platform"));
            assert!(!cargo.contains("workspace:"));
            assert!(app.join("src/main.rs").is_file());
        } else {
            assert_eq!(state["targets"], serde_json::json!(["native", "web"]));
            assert!(app.join("src/main.tsx").is_file());
            assert!(app.join("src/web.ts").is_file());
            let package = fs::read_to_string(app.join("package.json")).unwrap();
            assert!(!package.contains("workspace:"));
            assert!(!package.contains("/home/"));
            let package: Value = serde_json::from_str(&package).unwrap();
            assert_eq!(package["devDependencies"]["oxfmt"], "0.70.0");
            let formatting = fs::read_to_string(app.join(".oxfmtrc.json")).unwrap();
            let formatting: Value = serde_json::from_str(&formatting).unwrap();
            assert_eq!(formatting["singleQuote"], true);
            assert_eq!(formatting["ignorePatterns"][0], "**/*.generated.ts");
        }
    }
}

#[test]
/// Init validates every option and collision before touching existing files.
fn init_rejects_unsupported_choices_and_preserves_existing_files() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("demo");
    fs::create_dir(&app).unwrap();
    fs::write(app.join("src"), "private data").unwrap();
    let result = run(root.path(), &["init", "solid", "--dir", "demo", "--yes"]);
    assert!(
        result
            .unwrap_err()
            .contains("generated file already exists")
    );
    assert_eq!(fs::read_to_string(app.join("src")).unwrap(), "private data");
    assert!(!app.join("argui.json").exists());
    for (arguments, expected) in [
        (vec!["--targets", "native,mobile"], "iOS app shell"),
        (vec!["--targets", "native,native"], "each target once"),
        (vec!["--feature", "media"], "not wired"),
        (
            vec!["--feature", "automation", "--targets", "web"],
            "native target",
        ),
    ] {
        let mut command = vec!["init", "solid", "--dir", "invalid"];
        command.extend(arguments);
        command.push("--yes");
        assert!(run(root.path(), &command).unwrap_err().contains(expected));
        assert!(!root.path().join("invalid").exists());
    }
}

#[test]
/// A selected feature affects metadata and invalid target choices fail early.
fn features_and_build_selection_are_consistent() {
    let root = tempfile::tempdir().unwrap();
    run(
        root.path(),
        &[
            "init",
            "react",
            "--dir",
            "app",
            "--targets",
            "native",
            "--feature",
            "automation",
            "--feature",
            "tasks",
            "--yes",
        ],
    )
    .unwrap();
    let app = root.path().join("app");
    let state = manifest(&app);
    assert_eq!(
        state["features"],
        serde_json::json!(["automation", "tasks"])
    );
    assert_eq!(state["targets"], serde_json::json!(["native"]));
    assert!(!app.join("src/web.ts").exists());
    assert!(
        run(&app, &["build", "--target", "web"])
            .unwrap_err()
            .contains("not selected")
    );
}

#[test]
/// Unpublished development sources are never installed or presented as a tagged release.
fn add_is_atomic_while_sources_are_unpublished() {
    let root = tempfile::tempdir().unwrap();
    run(root.path(), &["init", "solid", "--dir", "app", "--yes"]).unwrap();
    let app = root.path().join("app");
    let original = fs::read(app.join("argui.json")).unwrap();
    let error = run(&app, &["add", "button", "input", "dialog", "--solid"]).unwrap_err();
    assert!(error.contains("no matching published widget source"));
    assert_eq!(fs::read(app.join("argui.json")).unwrap(), original);
    assert!(!app.join("ui").exists());
    assert!(
        run(&app, &["add", "button", "--solid", "--react", "select"])
            .unwrap_err()
            .contains("between framework selectors")
    );
}

#[test]
/// The production catalog exposes every v2 widget for each adapter.
fn list_finds_every_component_pair_outside_checkout() {
    let root = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_argui"))
        .args(["list", "components", "--json"])
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let catalog: Value = serde_json::from_slice(&output.stdout).unwrap();
    let rows = catalog.as_array().unwrap();
    assert_eq!(rows.len(), 12);
    assert_eq!(
        rows.iter()
            .filter(|row| row["framework"] == "solid")
            .count(),
        6
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["framework"] == "react")
            .count(),
        6
    );
    assert!(
        rows.iter()
            .all(|row| row["source"] == "unpublished development snapshot")
    );
    assert!(
        rows.iter()
            .all(|row| row["version"] == env!("CARGO_PKG_VERSION"))
    );
}

#[test]
/// Init fills an existing empty directory and rejects symlink traversal.
fn init_accepts_existing_directory_and_rejects_symlinks() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("existing");
    fs::create_dir(&app).unwrap();
    fs::write(app.join("notes.txt"), "keep me").unwrap();
    run(
        root.path(),
        &["init", "solid", "--dir", "existing", "--yes"],
    )
    .unwrap();
    assert_eq!(
        fs::read_to_string(app.join("notes.txt")).unwrap(),
        "keep me"
    );
    assert!(app.join("argui.json").is_file());

    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let redirected = root.path().join("redirected");
        symlink(&app, &redirected).unwrap();
        assert!(
            run(
                root.path(),
                &["init", "rust", "--dir", "redirected", "--yes"]
            )
            .unwrap_err()
            .contains("is a symlink")
        );
        let nested = root.path().join("nested");
        fs::create_dir(&nested).unwrap();
        symlink(&app, nested.join("src")).unwrap();
        assert!(
            run(root.path(), &["init", "rust", "--dir", "nested", "--yes"])
                .unwrap_err()
                .contains("symlink conflict")
        );
    }
    assert!(
        run(
            root.path(),
            &["init", "rust", "--dir", "missing/child", "--yes"]
        )
        .unwrap_err()
        .contains("parent directory does not exist")
    );
}

#[test]
/// Init rejects inconsistent unattended target and feature arguments.
fn init_validates_unattended_arguments() {
    let root = tempfile::tempdir().unwrap();
    for (options, message) in [
        (vec!["--name", "Invalid"], "project name"),
        (vec!["--targets", ""], "--targets accepts"),
        (vec!["--targets", "tablet"], "--targets accepts"),
        (
            vec!["--targets", "web"],
            "Rust scaffold currently supports native",
        ),
        (
            vec!["--feature", "tasks", "--feature", "tasks"],
            "only once",
        ),
        (vec!["--feature", "automation"], "Solid or React"),
    ] {
        let mut args = vec!["init", "rust", "--dir", "demo"];
        args.extend(options);
        args.push("--yes");
        let error = run(root.path(), &args).unwrap_err();
        assert!(error.contains(message), "{args:?}: {error}");
        assert!(!root.path().join("demo").exists());
    }
    assert!(
        run(root.path(), &["init", "rust", "--dir"])
            .unwrap_err()
            .contains("needs a value")
    );
}

#[test]
/// Unattended defaults select Solid and both supported targets.
fn init_yes_without_framework_uses_solid_defaults() {
    let root = tempfile::tempdir().unwrap();
    run(root.path(), &["init", "--dir", "demo", "--yes"]).unwrap();
    let state = manifest(&root.path().join("demo"));
    assert_eq!(state["framework"], "solid");
    assert_eq!(state["targets"], serde_json::json!(["native", "web"]));
}

#[test]
/// Build option parsing rejects incomplete selectors and unsupported flags before compilation.
fn standalone_commands_reject_incomplete_options() {
    let root = tempfile::tempdir().unwrap();
    run(root.path(), &["init", "rust", "--dir", "app", "--yes"]).unwrap();
    let app = root.path().join("app");
    for (args, expected) in [
        (vec!["check", "--target"], "needs native or web"),
        (vec!["build", "--target"], "needs native or web"),
        (vec!["build", "--target", "web"], "not selected"),
        (vec!["build", "--json"], "unknown build option"),
        (vec!["run", "--unknown"], "unknown run option"),
    ] {
        let error = run(&app, &args).unwrap_err();
        assert!(error.contains(expected), "{args:?}: {error}");
    }
}

#[test]
/// Automation requires a native TSX target even when a test file exists.
fn automation_rejects_rust_and_browser_only_apps() {
    let root = tempfile::tempdir().unwrap();
    for (framework, targets) in [("rust", "native"), ("solid", "web")] {
        let app = root.path().join(framework);
        run(
            root.path(),
            &[
                "init",
                framework,
                "--dir",
                framework,
                "--targets",
                targets,
                "--yes",
            ],
        )
        .unwrap();
        fs::write(app.join("src/example.test.ts"), "export default {}\n").unwrap();
        let error = run(&app, &["test", "src/example.test.ts", "--out", "results"]).unwrap_err();
        assert!(error.contains("native Solid or React target"), "{error}");
    }
}

#[test]
/// Commands reject each inconsistent standalone manifest before launching build tools.
fn malformed_standalone_manifests_fail_before_build() {
    let root = tempfile::tempdir().unwrap();
    run(root.path(), &["init", "solid", "--dir", "app", "--yes"]).unwrap();
    let app = root.path().join("app");
    let original = manifest(&app);
    type ManifestCase = (Box<dyn Fn(&mut Value)>, &'static str);
    let cases: [ManifestCase; 9] = [
        (
            Box::new(|value| {
                value.as_object_mut().unwrap().remove("name");
            }),
            "missing field",
        ),
        (
            Box::new(|value| value["name"] = "Invalid".into()),
            "invalid application manifest",
        ),
        (
            Box::new(|value| value["framework"] = "other".into()),
            "unknown framework",
        ),
        (
            Box::new(|value| value["targets"] = serde_json::json!([])),
            "invalid target set",
        ),
        (
            Box::new(|value| value["targets"] = serde_json::json!(["native", "native"])),
            "invalid target set",
        ),
        (
            Box::new(|value| value["targets"] = serde_json::json!(["mobile"])),
            "invalid target set",
        ),
        (
            Box::new(|value| {
                value["framework"] = "rust".into();
                value["targets"] = serde_json::json!(["web"]);
            }),
            "Rust scaffold currently supports native",
        ),
        (
            Box::new(|value| value["arguiVersion"] = "0.0.0".into()),
            "exact SDK snapshot",
        ),
        (
            Box::new(|value| value["sdkSha256"] = "0000".into()),
            "exact SDK snapshot",
        ),
    ];
    for (alter, expected) in cases {
        let mut changed = original.clone();
        alter(&mut changed);
        fs::write(
            app.join("argui.json"),
            serde_json::to_vec(&changed).unwrap(),
        )
        .unwrap();
        let error = run(&app, &["check"]).unwrap_err();
        assert!(error.contains(expected), "{changed}: {error}");
    }
}
