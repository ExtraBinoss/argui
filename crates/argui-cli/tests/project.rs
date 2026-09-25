#![cfg(not(target_arch = "wasm32"))]

use argui_cli::{Framework, Project, Target, run_in};
use std::{fs, path::Path, process::Command};

/// Creates the marker files needed to recognize a temporary Argui checkout.
fn fake_root(root: &Path) {
    fs::create_dir_all(root.join("packages/host")).unwrap();
    fs::create_dir_all(root.join("apps/gallery/quickjs-host")).unwrap();
    fs::write(root.join("packages/host/package.json"), "{}").unwrap();
    fs::write(root.join("apps/gallery/quickjs-host/Cargo.toml"), "").unwrap();
}

#[test]
/// Covers both generated adapters and input name validation.
fn init_writes_both_adapter_templates_and_rejects_unsafe_names() {
    let temp = tempfile::tempdir().unwrap();
    fake_root(temp.path());
    for (framework, name) in [("solid", "solid-demo"), ("react", "react-demo")] {
        run_in(temp.path(), &["init".into(), framework.into(), name.into()]).unwrap();
        let app = temp.path().join("apps").join(name);
        let manifest: Project =
            serde_json::from_slice(&fs::read(app.join("argui.json")).unwrap()).unwrap();
        assert_eq!(manifest.framework, Framework::parse(framework).unwrap());
        assert!(
            fs::read_to_string(app.join("src/main.tsx"))
                .unwrap()
                .contains("mountGallery")
        );
        assert!(
            fs::read_to_string(app.join("src/main.tsx"))
                .unwrap()
                .contains("backdrop_filter")
        );
        assert!(run_in(temp.path(), &["init".into(), framework.into(), name.into()]).is_err());
    }
    assert!(
        run_in(
            temp.path(),
            &["init".into(), "solid".into(), "../bad".into()]
        )
        .is_err()
    );
    run_in(temp.path(), &["init".into(), "solid".into()]).unwrap();
    assert!(
        temp.path()
            .join("apps/argui-solid-app/argui.json")
            .is_file()
    );
}

#[test]
/// Initialization reports a dangling app path before writing files.
#[cfg(unix)]
fn init_reports_dangling_app_path() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    fake_root(temp.path());
    symlink(temp.path().join("missing"), temp.path().join("apps/demo")).unwrap();
    assert!(run_in(temp.path(), &["init".into(), "solid".into(), "demo".into()]).is_err());
}

#[test]
/// Reports invalid commands without starting external tools.
fn invalid_commands_report_errors() {
    let temp = tempfile::tempdir().unwrap();
    assert!(run_in(temp.path(), &["build".into(), "other".into()]).is_err());
    assert!(run_in(temp.path(), &["check".into()]).is_err());
    for args in [
        vec!["build"],
        vec!["build", "apps/missing"],
        vec!["run", "dev", "apps/missing"],
        vec!["dev"],
        vec!["check", "apps/missing", "--json"],
        vec!["add", "solid"],
        vec!["dev", "apps/missing", "extra"],
        vec!["run"],
        vec!["unknown"],
    ] {
        assert!(
            run_in(
                temp.path(),
                &args.into_iter().map(str::to_owned).collect::<Vec<_>>()
            )
            .is_err()
        );
    }
}

#[test]
/// Web presets and explicit React selection produce an embeddable canvas app.
fn web_scaffold_has_target_and_embed_entry() {
    let temp = tempfile::tempdir().unwrap();
    fake_root(temp.path());
    for (args, name, framework) in [
        (vec!["init", "counter-web"], "counter-web", Framework::Solid),
        (
            vec!["init", "react", "react-web"],
            "react-web",
            Framework::React,
        ),
    ] {
        run_in(
            temp.path(),
            &args.into_iter().map(str::to_owned).collect::<Vec<_>>(),
        )
        .unwrap();
        let app = temp.path().join("apps").join(name);
        let project: Project =
            serde_json::from_slice(&fs::read(app.join("argui.json")).unwrap()).unwrap();
        assert_eq!(project.target, Target::Web);
        assert_eq!(project.framework, framework);
        assert!(
            fs::read_to_string(app.join("index.html"))
                .unwrap()
                .contains("argui-root")
        );
        assert!(
            fs::read_to_string(app.join("src/mount.ts"))
                .unwrap()
                .contains("ArguiWebHost")
        );
        assert!(
            fs::read_to_string(app.join("README.md"))
                .unwrap()
                .contains("mountArgui")
        );
    }
    run_in(temp.path(), &["init".into(), "counter-native".into()]).unwrap();
    let project: Project = serde_json::from_slice(
        &fs::read(temp.path().join("apps/counter-native/argui.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(project.target, Target::Native);
    let counter = |path: &str| {
        let source = fs::read_to_string(temp.path().join(path)).unwrap();
        source
            .split("function App() {")
            .nth(1)
            .unwrap()
            .split("/** Mounts")
            .next()
            .unwrap()
            .to_owned()
    };
    assert_eq!(
        counter("apps/counter-web/src/main.tsx").trim(),
        counter("apps/counter-native/src/main.tsx").trim()
    );
}

#[test]
/// The entry screen identifies the platform and gives quiet project guidance.
fn overview_and_help_are_usable_without_a_project() {
    let temp = tempfile::tempdir().unwrap();
    run_in(temp.path(), &[]).unwrap();
    run_in(temp.path(), &["--help".into()]).unwrap();
    let doctor = run_in(temp.path(), &["doctor".into()]);
    if let Err(message) = doctor {
        assert!(message.contains("missing prerequisites"));
    }
    let overview = Command::new(env!("CARGO_BIN_EXE_argui"))
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(overview.status.success());
    let output = String::from_utf8(overview.stdout).unwrap();
    assert!(output.contains(std::env::consts::OS));
    assert!(output.contains("argui doctor"));
    assert!(output.contains("https://discord.gg/66rjffMmD"));

    let help = Command::new(env!("CARGO_BIN_EXE_argui"))
        .arg("--help")
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(help.status.success());
    assert!(
        String::from_utf8(help.stdout)
            .unwrap()
            .contains("argui add")
    );
    fake_root(temp.path());
    run_in(temp.path(), &["init".into(), "solid".into(), "demo".into()]).unwrap();
    run_in(&temp.path().join("apps/demo"), &[]).unwrap();
}
