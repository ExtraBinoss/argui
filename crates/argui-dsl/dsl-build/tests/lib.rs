use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use argui_dsl_compiler::CompilerError;

#[test]
fn explicit_build_output_is_atomic_and_contains_no_interpreter_calls() {
    let temporary = tempfile::tempdir().unwrap();
    let manifest = temporary.path();
    std::fs::create_dir_all(manifest.join("ui")).unwrap();
    std::fs::write(
        manifest.join("ui/main.argui"),
        "import { Text } from \"@argui/ui\" export component Main { Text { content: \"ok\" } }",
    )
    .unwrap();
    let output = manifest.join("generated/argui_ui.rs");
    argui_dsl_build::compile_to(manifest, Path::new("ui/main.argui"), &output).unwrap();
    let generated = std::fs::read_to_string(&output).unwrap();
    assert!(generated.contains("construct_native"));
    assert!(!generated.contains("argui_dsl_runtime"));
    assert!(!output.with_extension("rs.tmp").exists());
}

#[test]
fn build_errors_keep_actionable_context_for_every_failure_kind() {
    let io = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "locked");
    let cases = [
        (
            argui_dsl_build::BuildError::MissingEnvironment("OUT_DIR"),
            "build environment lacks `OUT_DIR`",
        ),
        (
            argui_dsl_build::BuildError::Io {
                path: PathBuf::from("ui/main.argui"),
                source: io,
            },
            "ui/main.argui: locked",
        ),
        (
            argui_dsl_build::BuildError::Compiler(CompilerError::MissingEntry(
                "ui/main.argui".into(),
            )),
            "entry module `ui/main.argui` was not found",
        ),
        (
            argui_dsl_build::BuildError::EntryOutsideManifest(PathBuf::from("../main.argui")),
            "entry `../main.argui` must be inside CARGO_MANIFEST_DIR",
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
    let converted = argui_dsl_build::BuildError::from(CompilerError::Codegen("broken".into()));
    assert_eq!(converted.to_string(), "Rust AOT generation: broken");
}

#[test]
fn compile_to_reports_entry_source_and_output_failures() {
    let temporary = tempfile::tempdir().unwrap();
    let manifest = temporary.path();
    std::fs::create_dir_all(manifest.join("ui")).unwrap();

    let missing_entry = argui_dsl_build::compile_to(
        manifest,
        Path::new("ui/missing.argui"),
        &manifest.join("generated.rs"),
    )
    .unwrap_err();
    assert_eq!(
        missing_entry.to_string(),
        "entry module `ui/missing.argui` was not found"
    );

    std::fs::write(
        manifest.join("ui/broken.argui"),
        "export component Broken {",
    )
    .unwrap();
    let invalid = argui_dsl_build::compile_to(
        manifest,
        Path::new("ui/broken.argui"),
        &manifest.join("generated.rs"),
    )
    .unwrap_err();
    assert!(invalid.to_string().contains("DSL diagnostic"));

    let outside = argui_dsl_build::compile_to(
        manifest,
        &manifest.parent().unwrap().join("outside.argui"),
        &manifest.join("generated.rs"),
    )
    .unwrap_err();
    assert!(
        outside
            .to_string()
            .contains("must be inside CARGO_MANIFEST_DIR")
    );

    std::fs::remove_file(manifest.join("ui/broken.argui")).unwrap();
    std::fs::write(manifest.join("ui/valid.argui"), "export component Valid {}").unwrap();
    std::fs::write(manifest.join("blocked"), "not a directory").unwrap();
    let blocked_parent = argui_dsl_build::compile_to(
        manifest,
        Path::new("ui/valid.argui"),
        &manifest.join("blocked/generated.rs"),
    )
    .unwrap_err();
    assert!(blocked_parent.to_string().contains("blocked"));

    let output_directory = manifest.join("output");
    std::fs::create_dir(&output_directory).unwrap();
    let rename_failure =
        argui_dsl_build::compile_to(manifest, Path::new("ui/valid.argui"), &output_directory)
            .unwrap_err();
    assert!(rename_failure.to_string().contains("output"));
    assert!(!output_directory.with_extension("rs.tmp").exists());
}

#[test]
/// Accepts absolute entries and preserves relative `./` module keys.
fn compile_to_accepts_absolute_and_curdir_entries() {
    let temporary = tempfile::tempdir().unwrap();
    let manifest = temporary.path();
    std::fs::create_dir_all(manifest.join("ui")).unwrap();
    std::fs::write(manifest.join("ui/main.argui"), "export component Main {}").unwrap();

    let absolute_output = manifest.join("absolute.rs");
    argui_dsl_build::compile_to(manifest, &manifest.join("ui/main.argui"), &absolute_output)
        .unwrap();
    assert!(absolute_output.is_file());

    let curdir_output = manifest.join("curdir.rs");
    argui_dsl_build::compile_to(manifest, Path::new("./ui/main.argui"), &curdir_output).unwrap();
    assert!(curdir_output.is_file());
}

#[test]
/// Reports invalid source bytes and atomic output failures without a parent path.
fn compile_to_reports_invalid_utf8_sources_and_output_without_parent() {
    let temporary = tempfile::tempdir().unwrap();
    let manifest = temporary.path();
    std::fs::create_dir_all(manifest.join("ui")).unwrap();
    std::fs::write(manifest.join("ui/broken.argui"), [0xff_u8]).unwrap();
    let invalid = argui_dsl_build::compile_to(
        manifest,
        Path::new("ui/broken.argui"),
        &manifest.join("generated.rs"),
    )
    .unwrap_err();
    assert!(invalid.to_string().contains("broken.argui"));
    std::fs::remove_file(manifest.join("ui/broken.argui")).unwrap();

    std::fs::write(manifest.join("ui/main.argui"), "export component Main {}").unwrap();
    let no_parent =
        argui_dsl_build::compile_to(manifest, Path::new("ui/main.argui"), Path::new(""))
            .unwrap_err();
    match no_parent {
        argui_dsl_build::BuildError::Io { path, .. } => {
            assert!(
                path.as_os_str().is_empty(),
                "unexpected temporary path: {path:?}"
            );
        }
        other => panic!("empty output should fail while writing, got {other:?}"),
    }
}

#[test]
/// Embeds reachable binary assets and keeps their Cargo dependency paths current.
fn compile_to_embeds_reachable_assets_and_reports_loader_failures() {
    let temporary = tempfile::tempdir().unwrap();
    let manifest = temporary.path();
    std::fs::create_dir_all(manifest.join("ui")).unwrap();
    let source = r#"import { Text } from "@argui/ui"
export component Main {
    private property icon: asset = asset("icon.bin")
    Text { content: "asset" }
}"#;
    std::fs::write(manifest.join("ui/main.argui"), source).unwrap();
    std::fs::write(manifest.join("ui/icon.bin"), [1_u8, 2, 3]).unwrap();

    let output = manifest.join("generated/argui_ui.rs");
    argui_dsl_build::compile_to(manifest, Path::new("ui/main.argui"), &output).unwrap();
    let generated = std::fs::read_to_string(&output).unwrap();
    assert!(generated.contains("ASSET_"));
    assert!(generated.contains("ui/icon.bin"));

    std::fs::write(
        manifest.join("ui/main.argui"),
        source.replace("icon.bin", "missing.bin"),
    )
    .unwrap();
    argui_dsl_build::compile_to(manifest, Path::new("ui/main.argui"), &output).unwrap();
    let generated = std::fs::read_to_string(&output).unwrap();
    assert!(generated.contains("ui/missing.bin"));
}

#[test]
/// Loads reachable WGSL assets and reports a missing shader through `BuildError`.
fn compile_to_loads_reachable_shaders_and_reports_missing_shader_files() {
    let temporary = tempfile::tempdir().unwrap();
    let manifest = temporary.path();
    std::fs::create_dir_all(manifest.join("ui/shaders")).unwrap();
    std::fs::write(
        manifest.join("ui/main.argui"),
        "export effect Glow { shader: \"shaders/glow.wgsl\" } export component Main {}",
    )
    .unwrap();
    std::fs::write(
        manifest.join("ui/shaders/glow.wgsl"),
        "fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> { return source; }",
    )
    .unwrap();

    let output = manifest.join("generated.rs");
    argui_dsl_build::compile_to(manifest, Path::new("ui/main.argui"), &output).unwrap();
    assert!(
        std::fs::read_to_string(&output)
            .unwrap()
            .contains("EffectDefinition")
    );

    std::fs::remove_file(manifest.join("ui/shaders/glow.wgsl")).unwrap();
    let error =
        argui_dsl_build::compile_to(manifest, Path::new("ui/main.argui"), &output).unwrap_err();
    assert!(error.to_string().contains("asset"));
}

#[test]
/// Reports discovery failures and failed writes to an occupied temporary path.
fn compile_to_reports_discovery_and_temporary_output_failures() {
    let temporary = tempfile::tempdir().unwrap();
    let manifest = temporary.path();
    std::fs::write(manifest.join("not-a-directory"), "blocked").unwrap();
    let discovery = argui_dsl_build::compile_to(
        manifest,
        Path::new("not-a-directory/main.argui"),
        &manifest.join("generated.rs"),
    )
    .unwrap_err();
    assert!(discovery.to_string().contains("not-a-directory"));

    std::fs::create_dir(manifest.join("ui")).unwrap();
    std::fs::write(manifest.join("ui/main.argui"), "export component Main {}").unwrap();
    let output = manifest.join("generated.rs");
    std::fs::create_dir(output.with_extension("rs.tmp")).unwrap();
    let write_failure =
        argui_dsl_build::compile_to(manifest, Path::new("ui/main.argui"), &output).unwrap_err();
    assert!(write_failure.to_string().contains("generated.rs.tmp"));
}

#[test]
fn compile_to_discovers_nested_modules_and_ignores_non_sources() {
    let temporary = tempfile::tempdir().unwrap();
    let manifest = temporary.path();
    std::fs::create_dir_all(manifest.join("ui/nested")).unwrap();
    std::fs::create_dir_all(manifest.join("ui/node_modules/ignored")).unwrap();
    std::fs::write(manifest.join("ui/main.argui"), "export component Main {}").unwrap();
    std::fs::write(
        manifest.join("ui/nested/card.argui"),
        "export component Card {}",
    )
    .unwrap();
    std::fs::write(manifest.join("ui/notes.txt"), "not DSL").unwrap();
    std::fs::write(
        manifest.join("ui/node_modules/ignored/ignored.argui"),
        "export component Ignored {}",
    )
    .unwrap();
    let output = manifest.join("generated/argui_ui.rs");

    argui_dsl_build::compile_to(manifest, Path::new("ui/main.argui"), &output).unwrap();
    let generated = std::fs::read_to_string(output).unwrap();
    assert!(generated.contains("pub struct Main"));
    assert!(!generated.contains("pub struct Card"));
    assert!(!generated.contains("Ignored"));
}

#[test]
/// Resolves imported nested modules using manifest-relative keys from discovery.
fn compile_to_resolves_nested_module_keys() {
    let temporary = tempfile::tempdir().unwrap();
    let manifest = temporary.path();
    std::fs::create_dir_all(manifest.join("ui/nested")).unwrap();
    std::fs::write(
        manifest.join("ui/main.argui"),
        "import { Card } from \"./nested/card.argui\" export component Main { Card {} }",
    )
    .unwrap();
    std::fs::write(
        manifest.join("ui/nested/card.argui"),
        "export component Card {}",
    )
    .unwrap();

    let output = manifest.join("generated.rs");
    argui_dsl_build::compile_to(manifest, Path::new("ui/main.argui"), &output).unwrap();
    let generated = std::fs::read_to_string(output).unwrap();
    assert!(generated.contains("pub struct Main"));
}

#[cfg(unix)]
#[test]
fn compile_to_does_not_follow_symlink_directories() {
    use std::os::unix::fs::symlink;

    let temporary = tempfile::tempdir().unwrap();
    let manifest = temporary.path();
    std::fs::create_dir_all(manifest.join("ui/real")).unwrap();
    std::fs::write(manifest.join("ui/main.argui"), "export component Main {}").unwrap();
    std::fs::write(
        manifest.join("ui/real/child.argui"),
        "export component Child {}",
    )
    .unwrap();
    symlink(manifest.join("ui/real"), manifest.join("ui/link")).unwrap();
    let output = manifest.join("generated.rs");

    argui_dsl_build::compile_to(manifest, Path::new("ui/main.argui"), &output).unwrap();
    let generated = std::fs::read_to_string(output).unwrap();
    assert!(!generated.contains("Child"));
}

#[test]
fn compile_uses_cargo_environment_and_reports_missing_variables() {
    let executable = env::current_exe().unwrap();
    for missing in ["manifest", "out"] {
        let mut command = Command::new(&executable);
        command
            .args([
                "--exact",
                "compile_without_environment_helper",
                "--nocapture",
            ])
            .env("ARGUI_BUILD_CHILD", missing);
        if missing == "manifest" {
            command.env_remove("CARGO_MANIFEST_DIR");
        } else {
            command
                .env("CARGO_MANIFEST_DIR", env!("CARGO_MANIFEST_DIR"))
                .env_remove("OUT_DIR");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "child failed for {missing}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let project = tempfile::tempdir().unwrap();
    let output_directory = tempfile::tempdir().unwrap();
    let output = Command::new(&executable)
        .args([
            "--exact",
            "compile_without_environment_helper",
            "--nocapture",
        ])
        .env("ARGUI_BUILD_CHILD", "success")
        .env("CARGO_MANIFEST_DIR", project.path())
        .env("OUT_DIR", output_directory.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "successful compile child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_directory.path().join("argui_ui.rs").is_file());
}

#[test]
fn compile_without_environment_helper() {
    let Ok(missing) = env::var("ARGUI_BUILD_CHILD") else {
        return;
    };
    if missing == "success" {
        let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
        std::fs::create_dir_all(manifest.join("ui")).unwrap();
        std::fs::write(manifest.join("ui/main.argui"), "export component Main {}").unwrap();
        argui_dsl_build::compile("ui/main.argui").unwrap();
        argui_dsl_build::compile(PathBuf::from("ui/main.argui")).unwrap();
        argui_dsl_build::compile(Path::new("ui/main.argui")).unwrap();
        return;
    }
    let error = argui_dsl_build::compile("ui/main.argui").unwrap_err();
    assert_eq!(
        error.to_string(),
        format!(
            "build environment lacks `{}`",
            if missing == "manifest" {
                "CARGO_MANIFEST_DIR"
            } else {
                "OUT_DIR"
            }
        )
    );
}
