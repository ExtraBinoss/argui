#![cfg(not(target_arch = "wasm32"))]

use argui_cli::{DevCompilerService, ServiceError, is_relevant_event, is_relevant_path};
use argui_dsl_protocol::{
    ENGINE_COMPATIBILITY_VERSION, IR_FORMAT_VERSION, LiveMessage, PROTOCOL_VERSION, read_frame,
    write_frame,
};
use argui_dsl_runtime::{ClientEvent, LiveClient, LivePackage, LiveRuntime};
use argui_testing::TestApp;

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
fn service_turns_package_filesystem_failures_into_rejection_diagnostics() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::create_dir(directory.path().join("ui")).unwrap();
    std::fs::write(
        directory.path().join("ui/main.argui"),
        r#"import { Text } from "@argui/ui"
export component Main {
    private property icon: asset = asset("missing.bin")
    Text { content: "asset" }
}"#,
    )
    .unwrap();
    let mut service = DevCompilerService::open(directory.path(), "ui/main.argui").unwrap();
    let LiveMessage::Diagnostics {
        generation,
        diagnostics,
    } = service.compile().message
    else {
        panic!("missing package asset should reject the generation");
    };
    assert_eq!(generation, 1);
    assert_eq!(diagnostics[0].code, "package");
    assert!(!diagnostics[0].message.is_empty());
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
