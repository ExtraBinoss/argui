#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output},
};

/// Creates one executable fake tool in an isolated PATH.
fn script(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// Creates a standalone app and fake build tools with observable output.
fn fixture(root: &Path, framework: &str, targets: &str, features: &[&str]) {
    fs::create_dir_all(root.join("bin")).unwrap();
    let mut init = Command::new(env!("CARGO_BIN_EXE_argui"));
    init.args([
        "init",
        framework,
        "--dir",
        "app",
        "--name",
        "demo",
        "--targets",
        targets,
        "--yes",
        "--no-install",
    ])
    .current_dir(root);
    for feature in features {
        init.args(["--feature", feature]);
    }
    let output = init.output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let app = root.join("app");
    fs::create_dir_all(app.join("node_modules/typescript/bin")).unwrap();
    fs::create_dir_all(app.join("node_modules/vite/bin")).unwrap();
    fs::create_dir_all(app.join("node_modules/oxfmt/bin")).unwrap();
    fs::write(app.join("node_modules/oxfmt/bin/oxfmt"), "fake formatter").unwrap();
    script(
        &root.join("bin/bun"),
        r##"#!/bin/sh
printf 'bun %s\n' "$*" >> "$ARGUI_FAKE_LOG"
case "$1" in
  --version) echo 1.4.0 ;;
  *tsc) if [ "${ARGUI_FAKE_TSC_FAIL:-}" = 1 ]; then exit 2; fi; echo types-ok ;;
  *oxfmt)
    case "$*" in *'src vite.config.ts'*) ;; *) exit 4 ;; esac
    case "$*" in
      *--check*) if grep -q 'const  x' src/format-case.tsx; then exit 2; fi ;;
      *) printf 'const x = 1;\n' > src/format-case.tsx ;;
    esac ;;
  *vite.js)
    if [ "$2" = build ]; then
      case "$*" in
        *native*) if [ "${ARGUI_FAKE_NO_BUNDLE:-}" != 1 ]; then mkdir -p dist/native; echo 'export const app=1' > dist/native/app.mjs; fi ;;
        *web*) mkdir -p dist/web; echo '<html></html>' > dist/web/index.html ;;
      esac
    fi
    if [ "${ARGUI_FAKE_VITE_FAIL:-}" = 1 ]; then exit 2; fi
    ;;
  scripts/generate-assets.mjs)
    if [ -n "${ARGUI_REAL_BUN:-}" ]; then "$ARGUI_REAL_BUN" "$ARGUI_REAL_GENERATOR" "$2"; else
      echo '{"version":1,"totalBytes":0,"assets":[]}' > assets.generated.json
      cp assets.generated.json assets.dev.generated.json
      echo 'export const mediaAssets = {}' > assets.generated.ts
    fi ;;
  *.mjs) mkdir -p "$4"; echo 'export default {}' > "$4/test.mjs" ;;
  *) exit 3 ;;
esac
"##,
    );
    script(
        &root.join("bin/cargo"),
        r##"#!/bin/sh
printf 'cargo %s\n' "$*" >> "$ARGUI_FAKE_LOG"
if [ "$1" = check ]; then
  if [ "${ARGUI_FAKE_CARGO_FAIL:-}" = 1 ]; then exit 2; fi
  echo rust-ok
  exit 0
fi
if [ "$1" != build ]; then exit 3; fi
target=''; profile=debug
while [ "$#" -gt 0 ]; do
  case "$1" in
    --target-dir) shift; target="$1" ;;
    --release) profile=release ;;
  esac
  shift
done
mkdir -p "$target/$profile"
if [ "${ARGUI_FAKE_NO_BINARY:-}" = 1 ]; then exit 0; fi
case "$(cat argui.json)" in
  *'"framework": "rust"'*) binary=demo ;;
  *) binary=argui-app-native ;;
esac
printf '#!/bin/sh\nif [ -n "${ARGUI_AUTOMATION_GATE:-}" ]; then dd bs=1 count=1 of=/dev/null 2>/dev/null; sleep 0.35; fi\nif [ -n "${ARGUI_AUTOMATION_OUT:-}" ]; then mkdir -p "$ARGUI_AUTOMATION_OUT"; echo '\''{"ok":true,"steps":[]}'\'' > "$ARGUI_AUTOMATION_OUT/report.json"; fi\nexit 0\n' > "$target/$profile/$binary"
chmod +x "$target/$profile/$binary"
if [ "${ARGUI_FAKE_CARGO_FAIL:-}" = 1 ]; then exit 2; fi
"##,
    );
    script(
        &root.join("bin/wasm-pack"),
        r##"#!/bin/sh
printf 'wasm-pack %s\n' "$*" >> "$ARGUI_FAKE_LOG"
if [ "$1" = --version ]; then echo wasm-pack; exit 0; fi
if [ "${ARGUI_FAKE_WASM_FAIL:-}" = 1 ]; then exit 2; fi
if [ "${ARGUI_FAKE_NO_PKG:-}" = 1 ]; then exit 0; fi
while [ "$#" -gt 0 ]; do
  if [ "$1" = --out-dir ]; then shift; output="$1"; fi
  shift
done
mkdir -p "$output"
echo wasm > "$output/argui_app_web_bg.wasm"
echo 'export class ArguiWebHost {}' > "$output/argui_app_web.js"
"##,
    );
    script(
        &root.join("bin/rustup"),
        r##"#!/bin/sh
if [ "${ARGUI_FAKE_NO_WASM:-}" != 1 ]; then echo wasm32-unknown-unknown; fi
"##,
    );
}

/// Executes the installed CLI against one fixture and captures diagnostics.
fn cli(root: &Path, args: &[&str], flags: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_argui"));
    command
        .args(args)
        .current_dir(root.join("app"))
        .env(
            "PATH",
            format!(
                "{}:{}",
                root.join("bin").display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .env("ARGUI_FAKE_LOG", root.join("commands.log"))
        .env("ARGUI_CACHE_DIR", root.join("cache"))
        .env("CARGO_TARGET_DIR", root.join("target"));
    for (key, value) in flags {
        command.env(key, value);
    }
    command.output().unwrap()
}

/// Reports an unsuccessful command with its stderr in test output.
fn succeeds(output: Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
/// A generated app formats TSX with its local Oxfmt and checks without writing.
fn format_uses_generated_app_dependency_and_supports_check() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "react", "native", &[]);
    let app = root.join("app");
    let source = app.join("src/format-case.tsx");
    fs::write(&source, "const  x=1\n").unwrap();
    let failed = cli(root, &["format", "--check"], &[]);
    assert!(!failed.status.success());
    assert_eq!(fs::read_to_string(&source).unwrap(), "const  x=1\n");
    succeeds(cli(root, &["format", "."], &[]));
    assert_eq!(fs::read_to_string(&source).unwrap(), "const x = 1;\n");
    succeeds(cli(root, &["format", "--check"], &[]));
    let log = fs::read_to_string(root.join("commands.log")).unwrap();
    assert!(log.contains("--config .oxfmtrc.json --check src vite.config.ts"));
    assert!(log.contains("--config .oxfmtrc.json src vite.config.ts"));
    fs::remove_file(app.join("node_modules/oxfmt/bin/oxfmt")).unwrap();
    let missing = cli(root, &["format"], &[]);
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("bun install"));
}

#[test]
/// Solid's one source tree builds native and Web, runs both, and packages release output.
fn solid_builds_both_targets_and_packages_native() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "solid", "native,web", &["tasks"]);
    succeeds(cli(root, &["check", "--json"], &[]));
    succeeds(cli(root, &["build", "release"], &[]));
    succeeds(cli(root, &["run", "dev", "--target", "native"], &[]));
    succeeds(cli(root, &["dev", "--target", "web"], &[]));
    let app = root.join("app");
    assert!(app.join("dist/desktop/run.sh").is_file());
    assert!(app.join("dist/desktop/app.mjs").is_file());
    assert!(app.join("dist/desktop/assets").is_dir());
    assert!(app.join("dist/web/index.html").is_file());
    assert!(
        app.join("node_modules/@argui/web-host/argui_app_web_bg.wasm")
            .is_file()
    );
    let log = fs::read_to_string(root.join("commands.log")).unwrap();
    assert!(log.contains("--features tasks"));
    assert!(!log.contains("--features automation"));
    assert!(log.contains("wasm-pack build"));
    assert!(log.contains("bun "));
}

#[test]
/// A freshly initialized app packages its own referenced SVG and omits an unused peer.
fn initialized_app_packages_only_its_referenced_svg() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "solid", "native,web", &[]);
    let app = root.join("app");
    fs::create_dir(app.join("assets/mine")).unwrap();
    fs::write(
        app.join("assets/mine/star.svg"),
        "<svg><path d=\"M0 0\"/></svg>",
    )
    .unwrap();
    fs::write(
        app.join("assets/mine/unused.svg"),
        "<svg><path d=\"M1 1\"/></svg>",
    )
    .unwrap();
    fs::write(
        app.join("assets.config.json"),
        r#"{"packs":{"mine":"mine"},"files":{},"entries":["src/main.tsx"]}"#,
    )
    .unwrap();
    let source = fs::read_to_string(app.join("src/main.tsx")).unwrap();
    fs::write(
        app.join("src/main.tsx"),
        format!(
            "import {{ mediaAssets }} from '../assets.generated'\nconst ownIcon = mediaAssets['mine/star.svg']\n{source}"
        ),
    )
    .unwrap();
    let bun = std::env::split_paths(&std::env::var_os("PATH").unwrap())
        .map(|path| path.join("bun"))
        .find(|path| path.is_file())
        .expect("Bun is required for generated app tests");
    succeeds(cli(
        root,
        &["build", "release"],
        &[
            ("ARGUI_REAL_BUN", bun.to_str().unwrap()),
            (
                "ARGUI_REAL_GENERATOR",
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../scripts/generate-app-assets.mjs"
                ),
            ),
        ],
    ));
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(app.join("dist/desktop/assets.generated.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["assets"].as_array().unwrap().len(), 1);
    assert_eq!(manifest["assets"][0]["key"], "mine/star.svg");
    assert!(app.join("dist/desktop/assets/mine/star.svg").is_file());
    assert!(!app.join("dist/desktop/assets/mine/unused.svg").exists());
    assert!(app.join("dist/web/assets/mine/star.svg").is_file());
    assert!(!app.join("dist/web/assets/mine/unused.svg").exists());
    assert!(
        fs::read_to_string(app.join("dist/desktop/run.sh"))
            .unwrap()
            .contains("ARGUI_APP_ASSETS")
    );
}

#[test]
/// React's automation switch builds the instrumented host and runs test and screenshot commands.
fn react_test_and_screenshot_use_automation_host() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "react", "native", &["automation"]);
    let app = root.join("app");
    fs::write(app.join("src/click.test.ts"), "export default {}\n").unwrap();
    succeeds(cli(
        root,
        &["test", "src/click.test.ts", "--out", "results"],
        &[],
    ));
    succeeds(cli(root, &["screenshot", "--out", "capture.png"], &[]));
    assert!(app.join("results/report.json").is_file());
    assert!(app.join("report.json").is_file());
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(app.join("results/report.json")).unwrap()).unwrap();
    assert!(report["processMetrics"]["hostPid"].as_u64().unwrap() > 0);
    assert!(
        !report["processMetrics"]["samples"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let log = fs::read_to_string(root.join("commands.log")).unwrap();
    assert!(log.contains("--features automation"));
    assert!(log.contains(".argui-build-test.mjs"));
    assert!(!app.join("node_modules/.argui-build-test.mjs").exists());
}

#[test]
/// Missing dependencies and compiler failures name the failing standalone stage.
fn check_and_build_report_tool_failures() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "solid", "native,web", &[]);
    let app = root.join("app");
    fs::remove_dir_all(app.join("node_modules/vite")).unwrap();
    let missing = cli(root, &["check"], &[]);
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("bun install"));
    fs::create_dir_all(app.join("node_modules/vite/bin")).unwrap();
    let no_wasm = cli(root, &["check"], &[("ARGUI_FAKE_NO_WASM", "1")]);
    assert!(!no_wasm.status.success());
    assert!(String::from_utf8_lossy(&no_wasm.stderr).contains("rustup target add"));
    let wasm_fails = cli(root, &["check"], &[("ARGUI_FAKE_WASM_FAIL", "1")]);
    assert!(!wasm_fails.status.success());
    assert!(String::from_utf8_lossy(&wasm_fails.stderr).contains("wasm-pack build"));
    let types_fail = cli(root, &["check", "--json"], &[("ARGUI_FAKE_TSC_FAIL", "1")]);
    assert!(!types_fail.status.success());
    assert!(String::from_utf8_lossy(&types_fail.stdout).contains("\"ok\":false"));
    let vite_fails = cli(
        root,
        &["build", "--target", "native"],
        &[("ARGUI_FAKE_VITE_FAIL", "1")],
    );
    assert!(!vite_fails.status.success());
    assert!(String::from_utf8_lossy(&vite_fails.stderr).contains("vite native build"));
    let cargo_fails = cli(
        root,
        &["build", "--target", "native"],
        &[("ARGUI_FAKE_CARGO_FAIL", "1")],
    );
    assert!(!cargo_fails.status.success());
    assert!(String::from_utf8_lossy(&cargo_fails.stderr).contains("cargo native build"));
}

#[test]
/// Pure Rust checks and packages through its only direct Argui dependency.
fn rust_check_and_release_package_work_outside_checkout() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "rust", "native", &["tasks"]);
    succeeds(cli(root, &["check", "--json"], &[]));
    succeeds(cli(root, &["build", "release"], &[]));
    assert!(root.join("app/dist/desktop/demo").is_file());
    assert!(!root.join("app/dist/desktop/app.mjs").exists());
    succeeds(cli(root, &["run", "release"], &[]));
}

#[test]
/// Build and run validate output availability and report the exact failed stage.
fn missing_outputs_and_automation_selection_are_reported() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "react", "native,web", &[]);
    let no_automation = cli(root, &["screenshot", "--out", "screen.png"], &[]);
    assert!(!no_automation.status.success());
    assert!(String::from_utf8_lossy(&no_automation.stderr).contains("--feature automation"));
    let no_native_binary = cli(
        root,
        &["run", "dev", "--target", "native"],
        &[("ARGUI_FAKE_NO_BINARY", "1")],
    );
    assert!(!no_native_binary.status.success());
    assert!(
        String::from_utf8_lossy(&no_native_binary.stderr).contains("native executable missing")
    );
    let web_vite = cli(
        root,
        &["build", "--target", "web"],
        &[("ARGUI_FAKE_VITE_FAIL", "1")],
    );
    assert!(!web_vite.status.success());
    assert!(String::from_utf8_lossy(&web_vite.stderr).contains("vite web build"));
    let app = root.join("app");
    fs::remove_file(app.join("node_modules/@argui/host")).unwrap();
    fs::write(app.join("node_modules/@argui/host"), "conflict").unwrap();
    let conflict = cli(root, &["check"], &[]);
    assert!(!conflict.status.success());
    assert!(
        String::from_utf8_lossy(&conflict.stderr).contains("conflicts with the CLI SDK snapshot")
    );
}

#[test]
/// Rust compiler failures and a browser-only app retain their own target paths.
fn rust_failures_and_web_only_app_are_handled() {
    let rust = tempfile::tempdir().unwrap();
    fixture(rust.path(), "rust", "native", &[]);
    let check = cli(rust.path(), &["check"], &[("ARGUI_FAKE_CARGO_FAIL", "1")]);
    assert!(!check.status.success());
    assert!(String::from_utf8_lossy(&check.stderr).contains("Rust check failed"));
    let web = tempfile::tempdir().unwrap();
    fixture(web.path(), "solid", "web", &[]);
    succeeds(cli(web.path(), &["check"], &[]));
    succeeds(cli(web.path(), &["build", "--target", "web"], &[]));
    succeeds(cli(web.path(), &["run", "release"], &[]));
    assert!(!web.path().join("app/dist/native").exists());
}

#[test]
/// Desktop packaging reports missing build products and unwritable launchers.
fn native_release_reports_packaging_faults() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "solid", "native", &[]);
    let missing_binary = cli(
        root,
        &["build", "release"],
        &[("ARGUI_FAKE_NO_BINARY", "1")],
    );
    assert!(!missing_binary.status.success());
    assert!(String::from_utf8_lossy(&missing_binary.stderr).contains("argui-app-native"));
    fs::remove_file(root.join("app/dist/native/app.mjs")).unwrap();
    let missing_bundle = cli(
        root,
        &["build", "release"],
        &[("ARGUI_FAKE_NO_BUNDLE", "1")],
    );
    assert!(!missing_bundle.status.success());
    assert!(String::from_utf8_lossy(&missing_bundle.stderr).contains("app.mjs"));
    let launcher = root.join("app/dist/desktop/run.sh");
    fs::create_dir(&launcher).unwrap();
    let unwritable_launcher = cli(root, &["build", "release"], &[]);
    assert!(!unwritable_launcher.status.success());
    assert!(String::from_utf8_lossy(&unwritable_launcher.stderr).contains("run.sh"));
}

#[test]
/// Web packaging surfaces missing artifacts and filesystem conflicts.
fn web_host_reports_packaging_faults() {
    for (fault, expected) in [
        ("no-pkg", "hosts/web/pkg"),
        ("package-file", "web-host"),
        ("copy-directory", "argui_app_web_bg.wasm"),
        ("manifest-directory", "package.json"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fixture(root, "solid", "web", &[]);
        let app = root.join("app");
        let package = app.join("node_modules/@argui/web-host");
        let flags = match fault {
            "no-pkg" => vec![("ARGUI_FAKE_NO_PKG", "1")],
            "package-file" => {
                fs::create_dir_all(package.parent().unwrap()).unwrap();
                fs::write(&package, "conflict").unwrap();
                vec![]
            }
            "copy-directory" => {
                fs::create_dir_all(package.join("argui_app_web_bg.wasm")).unwrap();
                vec![]
            }
            _ => {
                fs::create_dir_all(package.join("package.json")).unwrap();
                vec![]
            }
        };
        let result = cli(root, &["check"], &flags);
        assert!(!result.status.success(), "{fault}");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(expected),
            "{fault}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
/// Missing native compiler and Web target manager receive prerequisite diagnostics.
fn absent_build_tools_are_reported() {
    let native = tempfile::tempdir().unwrap();
    fixture(native.path(), "rust", "native", &[]);
    fs::remove_file(native.path().join("bin/cargo")).unwrap();
    let path = format!("{}:/bin:/usr/bin", native.path().join("bin").display());
    let result = cli(native.path(), &["check"], &[("PATH", &path)]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("cargo check"));

    let web = tempfile::tempdir().unwrap();
    fixture(web.path(), "solid", "web", &[]);
    fs::remove_file(web.path().join("bin/rustup")).unwrap();
    let path = format!("{}:/bin:/usr/bin", web.path().join("bin").display());
    let result = cli(web.path(), &["check"], &[("PATH", &path)]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("rustup target list"));
}

#[test]
/// SDK link conflicts and output directory conflicts fail with their exact paths.
fn sdk_links_and_desktop_directory_report_conflicts() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "solid", "native", &[]);
    let scope = root.join("app/node_modules/@argui");
    fs::write(&scope, "occupied").unwrap();
    let error = cli(root, &["check"], &[]);
    assert!(!error.status.success());
    assert!(String::from_utf8_lossy(&error.stderr).contains("@argui"));
    fs::remove_file(&scope).unwrap();
    fs::create_dir(&scope).unwrap();
    std::os::unix::fs::symlink("absent", scope.join("host")).unwrap();
    let error = cli(root, &["check"], &[]);
    assert!(!error.status.success());
    assert!(String::from_utf8_lossy(&error.stderr).contains("@argui/host"));
    fs::remove_file(scope.join("host")).unwrap();
    fs::create_dir_all(root.join("app/dist")).unwrap();
    fs::write(root.join("app/dist/desktop"), "occupied").unwrap();
    let error = cli(root, &["build", "release"], &[]);
    assert!(!error.status.success());
    assert!(
        String::from_utf8_lossy(&error.stderr).contains("Not a directory")
            || String::from_utf8_lossy(&error.stderr).contains("File exists")
    );
}

#[test]
/// A missing Cargo executable fails the build launch before any package is emitted.
fn missing_cargo_build_tool_is_reported() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "rust", "native", &[]);
    fs::remove_file(root.join("bin/cargo")).unwrap();
    let path = root.join("bin").to_string_lossy().to_string();
    let result = cli(root, &["build"], &[("PATH", &path)]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("cargo native build"));
}

#[test]
/// Asset generation identifies Bun disappearing after prerequisite detection.
fn bun_disappearing_during_check_is_reported() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "solid", "native", &[]);
    script(
        &root.join("bin/bun"),
        "#!/bin/sh\nif [ \"$1\" = --version ]; then /bin/rm -- \"$0\"; echo 1.4.0; fi\n",
    );
    let path = format!("{}:/bin:/usr/bin", root.join("bin").display());
    let result = cli(root, &["check"], &[("PATH", &path)]);
    assert!(!result.status.success());
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("app asset generation"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
/// Builds use the app target directory when no Cargo target root was requested.
fn native_build_defaults_to_app_target_directory() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root, "rust", "native", &[]);
    let output = Command::new(env!("CARGO_BIN_EXE_argui"))
        .arg("build")
        .current_dir(root.join("app"))
        .env(
            "PATH",
            format!(
                "{}:{}",
                root.join("bin").display(),
                std::env::var("PATH").unwrap()
            ),
        )
        .env("ARGUI_FAKE_LOG", root.join("commands.log"))
        .env("ARGUI_CACHE_DIR", root.join("cache"))
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(root.join("app/target/native/debug/demo").is_file());
}
