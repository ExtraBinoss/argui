#![cfg(not(target_arch = "wasm32"))]

use argui_cli::{DevCompilerService, ServiceError, is_relevant_event, is_relevant_path};
use argui_dsl_compiler::{CompilerSession, SourceModule};
use argui_dsl_protocol::{
    ENGINE_COMPATIBILITY_VERSION, IR_FORMAT_VERSION, LiveMessage, LivePackageEnvelope,
    PROTOCOL_VERSION, PackageHeader, read_frame, write_frame,
};
use argui_dsl_runtime::{ClientEvent, LiveClient, LivePackage, LiveRuntime};
use argui_testing::TestApp;

/// Compiles and prunes one in-memory standard-library generation for a live client.
///
/// * `session` — long-lived compiler session with project and stdlib modules.
/// * `generation` — monotonically increasing version sent to the client.
///
/// Returns the exact package shape used by the development service.
fn compile_stdlib_package(session: &mut CompilerSession, generation: u64) -> LivePackageEnvelope {
    let mut compiled = session
        .compile(|path| Err(format!("unexpected asset `{path}`")))
        .unwrap();
    compiled.reachability.prune(&mut compiled.ir);
    LivePackageEnvelope {
        header: PackageHeader::current(compiled.public_api_hash, generation),
        roots: compiled.roots,
        ir: compiled.ir,
        assets: Vec::new(),
    }
}

#[test]
fn watcher_ignores_compiler_reads_without_losing_real_or_imprecise_edits() {
    use notify::{
        Event, EventKind,
        event::{AccessKind, ModifyKind},
    };

    let source = std::path::PathBuf::from("ui/main.argui");
    let access = Event::new(EventKind::Access(AccessKind::Any)).add_path(source.clone());
    let modify = Event::new(EventKind::Modify(ModifyKind::Any)).add_path(source.clone());
    let imprecise = Event::new(EventKind::Any).add_path(source);
    let unrelated = Event::new(EventKind::Modify(ModifyKind::Any))
        .add_path(std::path::PathBuf::from("src/main.rs"));

    assert!(!is_relevant_event(&access));
    assert!(is_relevant_event(&modify));
    assert!(is_relevant_event(&imprecise));
    assert!(!is_relevant_event(&unrelated));
}

#[test]
fn invalid_edit_keeps_last_valid_generation_available_to_clients() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    let path = directory.path().join("ui/main.argui");
    std::fs::write(
        &path,
        r#"import { Text } from "@argui/ui"
export component Main { Text { content: "valid" } }"#,
    )
    .unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();
    assert!(matches!(service.compile().message, LiveMessage::Package(_)));

    std::fs::write(&path, "export component Main { Missing {").unwrap();
    service.refresh_sources().unwrap();
    assert!(matches!(
        service.compile().message,
        LiveMessage::Diagnostics { .. }
    ));

    std::fs::write(
        &path,
        r#"import { Text } from "@argui/ui"
export component Main { Text { content: "fixed" } }"#,
    )
    .unwrap();
    service.refresh_sources().unwrap();
    let LiveMessage::Package(package) = service.compile().message else {
        panic!("fixed generation should compile");
    };
    assert_eq!(package.header.generation, 3);
}

/// The dev service sends generated path bytes without looking for a disk SVG.
#[test]
fn authored_path_bytes_are_included_in_live_packages() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    std::fs::write(
        directory.path().join("ui/main.argui"),
        r#"import { Path } from "@argui/native"
export component Main {
    Path { source: path(20.0, 20.0, [move_to(0.0, 0.0), line_to(20.0, 20.0)], false, 2.0, false) }
}"#,
    )
    .unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();
    let LiveMessage::Package(package) = service.compile().message else {
        panic!("authored path should compile without a source SVG file");
    };
    let generated = package
        .ir
        .assets
        .iter()
        .find(|asset| asset.inline_bytes.is_some())
        .unwrap();
    let payload = package
        .assets
        .iter()
        .find(|asset| asset.id == generated.id)
        .unwrap();
    assert_eq!(payload.bytes, *generated.inline_bytes.as_ref().unwrap());
    assert!(
        std::str::from_utf8(&payload.bytes)
            .unwrap()
            .contains("stroke-width=\"2\"")
    );
}

#[test]
fn malformed_number_rejection_has_file_span_and_specific_message() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    let source =
        "import { Column } from \"@argui/ui\"\nexport component Main { Column { gap: 100zzz8.0 } }";
    std::fs::write(directory.path().join("ui/main.argui"), source).unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();
    let LiveMessage::Diagnostics { diagnostics, .. } = service.compile().message else {
        panic!("malformed literal should reject the generation");
    };
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "InvalidNumber")
        .unwrap_or_else(|| {
            panic!("a specific numeric diagnostic should be emitted: {diagnostics:?}")
        });
    assert_eq!(diagnostic.path.as_deref(), Some("ui/main.argui"));
    let start = diagnostic.start.unwrap() as usize;
    let end = diagnostic.end.unwrap() as usize;
    assert_eq!(&source[start..end], "100zzz8.0");
    assert!(diagnostic.message.contains("100zzz8.0"));
}

#[test]
fn accepted_ui_edit_commits_without_rebuilding_rust() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    let path = directory.path().join("ui/main.argui");
    let source = |label: &str| {
        format!(
            r#"import {{ Text }} from "@argui/ui"
export component Main {{ Text {{ content: "{label}" }} }}"#
        )
    };
    std::fs::write(&path, source("first")).unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();
    let LiveMessage::Package(first) = service.compile().message else {
        panic!("initial package should compile");
    };
    let component = first.roots[0];
    let mut runtime = LiveRuntime::new(LivePackage::from_envelope(*first).unwrap()).unwrap();
    runtime.mount(component, []).unwrap();
    runtime.render().unwrap();

    std::fs::write(&path, source("second")).unwrap();
    service.refresh_sources().unwrap();
    let LiveMessage::Package(second) = service.compile().message else {
        panic!("edited package should compile");
    };
    let result = LiveClient::apply(&mut runtime, ClientEvent::Package(second)).unwrap();
    assert!(matches!(result, ClientEvent::Committed(_)));
    assert_eq!(runtime.generation(), 2);
    runtime.render().unwrap();
}

#[test]
fn native_live_runtime_connects_mounts_and_renders_the_entry_root() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    std::fs::write(
        directory.path().join("ui/main.argui"),
        r#"import { Text } from "@argui/ui"
export component Main { Text { content: "connected" } }"#,
    )
    .unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();
    let package = service.compile().message;
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        write_frame(
            &mut stream,
            &LiveMessage::Hello {
                protocol_version: PROTOCOL_VERSION,
                ir_format_version: IR_FORMAT_VERSION,
                engine_version: ENGINE_COMPATIBILITY_VERSION.into(),
            },
        )
        .unwrap();
        write_frame(&mut stream, &package).unwrap();
        assert_eq!(
            read_frame::<LiveMessage>(&mut stream).unwrap(),
            LiveMessage::Committed { generation: 1 }
        );
    });

    let runtime = LiveRuntime::connect(address, std::time::Duration::from_secs(2)).unwrap();
    TestApp::new(runtime).assert_text("connected");
    server.join().unwrap();
}

#[test]
fn a_remote_package_rebuilds_the_visible_tree_without_user_input() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    let path = directory.path().join("ui/main.argui");
    let source = |label: &str| {
        format!(
            "import {{ Text }} from \"@argui/ui\" export component Main {{ Text {{ content: \"{label}\" }} }}"
        )
    };
    std::fs::write(&path, source("before")).unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();
    let initial = service.compile().message;
    std::fs::write(&path, source("after")).unwrap();
    service.refresh_sources().unwrap();
    let changed = service.compile().message;
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (send_change, receive_change) = std::sync::mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        write_frame(
            &mut stream,
            &LiveMessage::Hello {
                protocol_version: PROTOCOL_VERSION,
                ir_format_version: IR_FORMAT_VERSION,
                engine_version: ENGINE_COMPATIBILITY_VERSION.into(),
            },
        )
        .unwrap();
        write_frame(&mut stream, &initial).unwrap();
        assert_eq!(
            read_frame::<LiveMessage>(&mut stream).unwrap(),
            LiveMessage::Committed { generation: 1 }
        );
        receive_change.recv().unwrap();
        write_frame(&mut stream, &changed).unwrap();
        assert_eq!(
            read_frame::<LiveMessage>(&mut stream).unwrap(),
            LiveMessage::Committed { generation: 2 }
        );
        std::thread::sleep(std::time::Duration::from_secs(2));
    });

    let runtime = LiveRuntime::connect(address, std::time::Duration::from_secs(2)).unwrap();
    let mut app = TestApp::new(runtime);
    app.assert_text("before");
    send_change.send(()).unwrap();
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        app.settle().unwrap();
        if app.entity().read(LiveRuntime::generation) == 2 {
            break;
        }
    }
    assert_eq!(app.entity().read(LiveRuntime::generation), 2);
    app.assert_text("after");
    server.join().unwrap();
}

/// Replacing a standard-library module in one live compiler session must
/// repaint the mounted UI after the client acknowledges the new generation.
#[test]
fn edited_stdlib_module_rebuilds_visible_ui_without_input_or_restart() {
    let mut session = CompilerSession::new("ui/main.argui").unwrap();
    session.update_module(SourceModule::new(
        "ui/main.argui",
        "import { Probe } from \"@argui/ui\" export component Main { Probe {} }",
    ));
    let module = |label: &str| {
        SourceModule::new(
            "@argui/ui/probe.argui",
            format!(
                "import {{ Text }} from \"@argui/native\" export component Probe {{ Text {{ content: \"{label}\" }} }}"
            ),
        )
    };
    session.update_module(module("before stdlib edit"));
    let first = compile_stdlib_package(&mut session, 1);
    session.update_module(module("after stdlib edit"));
    let second = compile_stdlib_package(&mut session, 2);
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (send_change, receive_change) = std::sync::mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        write_frame(
            &mut stream,
            &LiveMessage::Hello {
                protocol_version: PROTOCOL_VERSION,
                ir_format_version: IR_FORMAT_VERSION,
                engine_version: ENGINE_COMPATIBILITY_VERSION.into(),
            },
        )
        .unwrap();
        write_frame(&mut stream, &LiveMessage::Package(Box::new(first))).unwrap();
        assert_eq!(
            read_frame::<LiveMessage>(&mut stream).unwrap(),
            LiveMessage::Committed { generation: 1 }
        );
        receive_change.recv().unwrap();
        write_frame(&mut stream, &LiveMessage::Package(Box::new(second))).unwrap();
        assert_eq!(
            read_frame::<LiveMessage>(&mut stream).unwrap(),
            LiveMessage::Committed { generation: 2 }
        );
        std::thread::sleep(std::time::Duration::from_millis(300));
    });
    let runtime = LiveRuntime::connect(address, std::time::Duration::from_secs(2)).unwrap();
    let mut app = TestApp::new(runtime);
    app.assert_text("before stdlib edit");
    send_change.send(()).unwrap();
    for _ in 0..40 {
        std::thread::sleep(std::time::Duration::from_millis(25));
        app.settle().unwrap();
        if app.entity().read(LiveRuntime::generation) == 2 {
            break;
        }
    }
    assert_eq!(app.entity().read(LiveRuntime::generation), 2);
    app.assert_text("after stdlib edit");
    server.join().unwrap();
}

/// Import and asset failures report the exact DSL source site whenever known.
#[test]
fn icon_and_asset_diagnostics_include_source_path_and_spans() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    let path = directory.path().join("ui/main.argui");
    let icon_source = "import { NotATablerIcon } from \"@argui/icons\"\nexport component Main {}";
    std::fs::write(&path, icon_source).unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();
    let LiveMessage::Diagnostics { diagnostics, .. } = service.compile().message else {
        panic!("unknown icon should be rejected");
    };
    let icon = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("NotATablerIcon"))
        .unwrap();
    assert_eq!(icon.code, "UnresolvedImport");
    assert_eq!(icon.path.as_deref(), Some("ui/main.argui"));
    assert_eq!(
        &icon_source[icon.start.unwrap() as usize..icon.end.unwrap() as usize],
        "NotATablerIcon"
    );

    let invalid_source = "export component Main { private property icon: asset = asset() }";
    std::fs::write(&path, invalid_source).unwrap();
    service.refresh_sources().unwrap();
    let LiveMessage::Diagnostics { diagnostics, .. } = service.compile().message else {
        panic!("invalid asset expression should be rejected");
    };
    let invalid = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "InvalidAsset")
        .unwrap();
    assert_eq!(invalid.path.as_deref(), Some("ui/main.argui"));
    assert_eq!(
        &invalid_source[invalid.start.unwrap() as usize..invalid.end.unwrap() as usize],
        "asset()"
    );

    let missing_source = "import { Image } from \"@argui/native\"\nexport component Main { Image { source: asset(\"media/missing.png\") } }";
    std::fs::write(&path, missing_source).unwrap();
    service.refresh_sources().unwrap();
    let LiveMessage::Diagnostics { diagnostics, .. } = service.compile().message else {
        panic!("missing image source should be rejected");
    };
    let missing = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "Asset")
        .unwrap();
    assert_eq!(missing.path.as_deref(), Some("ui/main.argui"));
    assert!(missing.message.contains("ui/media/missing.png"));
    assert_eq!(
        &missing_source[missing.start.unwrap() as usize..missing.end.unwrap() as usize],
        "asset(\"media/missing.png\")"
    );
    let unknown_icon_source = "import { Svg } from \"@argui/native\"\nexport component Main { Svg { source: asset(\"@argui/icons/NoSuchIcon.svg\") } }";
    std::fs::write(&path, unknown_icon_source).unwrap();
    service.refresh_sources().unwrap();
    let LiveMessage::Diagnostics { diagnostics, .. } = service.compile().message else {
        panic!("unknown virtual icon should reject the generation");
    };
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "Asset"
            && diagnostic.path.as_deref() == Some("ui/main.argui")
            && diagnostic.message.contains("NoSuchIcon")
    }));
}

#[test]
fn relevant_paths_cover_every_supported_asset_extension_and_metadata_case() {
    for extension in ["argui", "wgsl", "png", "jpg", "jpeg", "webp", "gif", "svg"] {
        assert!(is_relevant_path(std::path::Path::new(&format!(
            "asset.{extension}"
        ))));
    }
    for path in ["asset", "asset.txt", "asset.ARGUI", "directory/"] {
        assert!(!is_relevant_path(std::path::Path::new(path)));
    }

    use notify::{Event, EventKind, event::AccessKind};
    let no_paths = Event::new(EventKind::Any);
    let other = Event::new(EventKind::Other).add_path(std::path::PathBuf::from("main.argui"));
    let access = Event::new(EventKind::Access(AccessKind::Any))
        .add_path(std::path::PathBuf::from("main.argui"));
    assert!(!is_relevant_event(&no_paths));
    assert!(!is_relevant_event(&other));
    assert!(!is_relevant_event(&access));
}

#[test]
fn service_reports_initialization_and_nonsemantic_compile_errors() {
    let missing = match DevCompilerService::open("/tmp/argui-service-no-such-root", "main.argui") {
        Ok(_) => panic!("a missing project root should fail initialization"),
        Err(error) => error,
    };
    assert!(matches!(missing, ServiceError::Io(_)));

    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    std::fs::write(
        directory.path().join("ui/main.argui"),
        "export component Main {}",
    )
    .unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/missing.argui").unwrap();
    let attempt = service.compile();
    let LiveMessage::Diagnostics {
        generation,
        diagnostics,
    } = attempt.message
    else {
        panic!("a missing entry should be reported as diagnostics");
    };
    assert_eq!(generation, 1);
    assert_eq!(diagnostics[0].code, "compile");
    assert!(diagnostics[0].start.is_none());
    assert!(diagnostics[0].end.is_none());
}

#[test]
fn service_tracks_asset_revisions_and_releases_removed_assets() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    let source = directory.path().join("ui/main.argui");
    std::fs::write(
        &source,
        r#"import { Text } from "@argui/ui"
export component Main {
    private property icon: asset = asset("icon.bin")
    Text { content: "asset" }
}"#,
    )
    .unwrap();
    std::fs::write(directory.path().join("ui/icon.bin"), [1_u8, 2, 3]).unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();

    let first = service.compile().message;
    let LiveMessage::Package(first) = first else {
        panic!("asset source should compile");
    };
    assert_eq!(first.assets.len(), 1);
    assert_eq!(first.assets[0].revision, 1);
    assert_eq!(first.assets[0].bytes, [1, 2, 3]);

    let same = service.compile().message;
    let LiveMessage::Package(same) = same else {
        panic!("unchanged asset should compile");
    };
    assert_eq!(same.assets[0].revision, 1);

    std::fs::write(directory.path().join("ui/icon.bin"), [4_u8, 5]).unwrap();
    let changed = service.compile().message;
    let LiveMessage::Package(changed) = changed else {
        panic!("changed asset should compile");
    };
    assert_eq!(changed.assets[0].revision, 2);
    assert_eq!(changed.assets[0].bytes, [4, 5]);

    std::fs::write(
        &source,
        r#"import { Text } from "@argui/ui"
export component Main { Text { content: "asset removed" } }"#,
    )
    .unwrap();
    service.refresh_sources().unwrap();
    let removed = service.compile().message;
    let LiveMessage::Package(removed) = removed else {
        panic!("asset removal should preserve a valid package");
    };
    assert!(removed.assets.is_empty());
}

#[test]
fn service_reports_missing_generic_asset_at_its_dsl_expression() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    let source = r#"import { Text } from "@argui/ui"
export component Main {
    private property icon: asset = asset("missing.bin")
    Text { content: "asset" }
}"#;
    std::fs::write(directory.path().join("ui/main.argui"), source).unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();
    let LiveMessage::Diagnostics {
        generation,
        diagnostics,
    } = service.compile().message
    else {
        panic!("missing generic asset should reject the generation");
    };
    assert_eq!(generation, 1);
    assert_eq!(diagnostics[0].code, "Asset");
    assert_eq!(diagnostics[0].path.as_deref(), Some("ui/main.argui"));
    assert_eq!(
        &source[diagnostics[0].start.unwrap() as usize..diagnostics[0].end.unwrap() as usize],
        "asset(\"missing.bin\")"
    );
    assert!(diagnostics[0].message.contains("ui/missing.bin"));
}

/// Virtual Tabler assets are packaged from the embedded catalog, not project files.
#[test]
fn service_packages_embedded_icon_svg_without_a_filesystem_asset() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    std::fs::write(
        directory.path().join("ui/main.argui"),
        "import { Svg } from \"@argui/native\"\nexport component Main { Svg { source: asset(\"@argui/icons/Loader2.svg\") } }",
    )
    .unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();
    let LiveMessage::Package(first) = service.compile().message else {
        panic!("an embedded icon should compile without a project asset");
    };
    assert_eq!(first.assets.len(), 1);
    assert_eq!(first.assets[0].revision, 1);
    assert!(first.assets[0].bytes.starts_with(b"<svg "));
    let LiveMessage::Package(second) = service.compile().message else {
        panic!("an unchanged embedded icon should remain available");
    };
    assert_eq!(second.assets[0].revision, 1);
    assert_eq!(second.assets[0].bytes, first.assets[0].bytes);
}

#[test]
fn service_preserves_warning_severity_in_semantic_diagnostics() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    std::fs::write(
        directory.path().join("ui/main.argui"),
        r#"import { Column, Text } from "@argui/ui"
export component Main {
    in property items: model<string>
    Column {
        for item in items { Text { content: item } }
        Missing {}
    }
}"#,
    )
    .unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();
    let LiveMessage::Diagnostics { diagnostics, .. } = service.compile().message else {
        panic!("a keyless repeater should produce a semantic warning");
    };
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == argui_dsl_protocol::Severity::Warning
            && diagnostic.code == "MissingRepeaterKey"
    }));
}

#[test]
fn refresh_sources_removes_deleted_modules_without_losing_the_service() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    let main = directory.path().join("ui/main.argui");
    let helper = directory.path().join("ui/helper.argui");
    std::fs::write(&main, "export component Main {}").unwrap();
    std::fs::write(&helper, "export component Helper {}").unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();
    std::fs::remove_file(helper).unwrap();
    service.refresh_sources().unwrap();
    assert!(matches!(service.compile().message, LiveMessage::Package(_)));
}
