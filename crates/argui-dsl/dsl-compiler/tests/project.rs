use argui_dsl_compiler::{Compiler, CompilerError, CompilerSession, SourceModule};
use argui_dsl_semantic::DiagnosticCode;

#[path = "project/asset_source.rs"]
mod asset_source;

const SOURCE: &str = r#"import { VirtualWindow, Text } from "@argui/native"
export component Main { private property items: array<string> = ["one"] VirtualWindow {
    row_height: 32.0
    animate row_height { duration: 100ms }
    for item in items key item { Text { content: item } }
} }"#;

#[test]
fn missing_entry_is_distinct_from_invalid_source() {
    let error = Compiler::compile(
        [SourceModule::new(
            "ui/other.argui",
            "export component Other {}",
        )],
        "ui/main.argui",
        |_| Err("unexpected asset".into()),
    )
    .err()
    .expect("missing entry must fail");
    assert!(matches!(error, CompilerError::MissingEntry(path) if path == "ui/main.argui"));
}

#[test]
fn removing_a_module_invalidates_incremental_compilation() {
    let mut session = CompilerSession::new("ui/main.argui").unwrap();
    session.update_module(SourceModule::new(
        "ui/main.argui",
        "export component Main {}",
    ));
    session.compile(|_| Err("unexpected asset".into())).unwrap();
    assert!(!session.remove_module("ui/absent.argui"));
    assert!(session.remove_module("ui/main.argui"));
    let error = session
        .compile(|_| Err("unexpected asset".into()))
        .err()
        .expect("removed entry must fail");
    assert!(matches!(error, CompilerError::MissingEntry(path) if path == "ui/main.argui"));
}

#[test]
fn raster_source_cannot_be_used_as_vector_media() {
    let error = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            "import { Svg } from \"@argui/native\" export component Main { Svg { source: asset(\"icon.png\") } }",
        )],
        "ui/main.argui",
        |_| Err("loader should not run before media kind validation".into()),
    )
    .err()
    .expect("raster/vector mismatch must fail");
    let CompilerError::Asset {
        path,
        message,
        source_span,
        ..
    } = error
    else {
        panic!("expected a media error");
    };
    assert_eq!(path.to_string_lossy(), "ui/icon.png");
    assert!(message.contains("expected Vector media"));
    assert!(source_span.is_some());
}

#[test]
fn missing_shader_reports_effect_declaration_span() {
    let source =
        "export effect Missing { shader: \"effects/missing.wgsl\" } export component Main {}";
    let error = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |path| {
            assert_eq!(path, "ui/effects/missing.wgsl");
            Err("missing shader".into())
        },
    )
    .err()
    .expect("reachable missing shader must fail");
    let CompilerError::Asset {
        source_span: Some(span),
        message,
        ..
    } = error
    else {
        panic!("expected an asset error with source span");
    };
    assert_eq!(message, "missing shader");
    assert!(
        source[u32::from(span.range.start()) as usize..u32::from(span.range.end()) as usize]
            .contains("effect Missing")
    );
}

#[test]
fn non_utf8_shader_is_rejected_before_validation() {
    let error = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            "export effect Broken { shader: \"effects/broken.wgsl\" } export component Main {}",
        )],
        "ui/main.argui",
        |_| Ok(vec![0xff]),
    )
    .err()
    .expect("non-UTF8 shader must fail");
    assert!(matches!(
        error,
        CompilerError::Asset {
            source_span: Some(_),
            ..
        }
    ));
}

#[test]
fn missing_default_asset_reports_property_expression_span() {
    let source = "export component Main { private property icon: asset = asset(\"missing.png\") }";
    let error = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("missing default asset".into()),
    )
    .err()
    .expect("reachable property asset must be loaded");
    let CompilerError::Asset {
        source_span: Some(span),
        message,
        ..
    } = error
    else {
        panic!("expected an asset error with source span");
    };
    assert_eq!(message, "missing default asset");
    assert_eq!(
        &source[u32::from(span.range.start()) as usize..u32::from(span.range.end()) as usize],
        "asset(\"missing.png\")"
    );
}

#[test]
fn missing_asset_in_conditional_visual_child_retains_inner_span() {
    let source = r#"import { Container, Image } from "@argui/native"
export component Main {
    private property show: bool = true
    Container {
        if show { Image { source: asset("conditional.png") } }
        else { Image { source: asset("other.png") } }
    }
}"#;
    let error = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("missing nested asset".into()),
    )
    .err()
    .expect("missing nested asset must fail");
    let CompilerError::Asset {
        source_span: Some(span),
        ..
    } = error
    else {
        panic!("expected source-located asset error");
    };
    let expression =
        &source[u32::from(span.range.start()) as usize..u32::from(span.range.end()) as usize];
    assert!(expression.starts_with("asset(\""), "{expression}");
}

#[test]
fn missing_asset_in_repeated_visual_child_retains_inner_span() {
    let source = r#"import { Container, Image } from "@argui/native"
export component Main {
    private property items: array<string> = ["one"]
    Container {
        for item in items key item {
            Image { source: asset("row.png") }
        }
    }
}"#;
    let error = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("missing row asset".into()),
    )
    .err()
    .expect("missing row asset must fail");
    let CompilerError::Asset {
        source_span: Some(span),
        ..
    } = error
    else {
        panic!("expected source-located asset error");
    };
    assert_eq!(
        &source[u32::from(span.range.start()) as usize..u32::from(span.range.end()) as usize],
        "asset(\"row.png\")"
    );
}

/// Both one-shot AOT and incremental live compilation reject the same
/// structural target at its authored DSL span.
#[test]
fn structural_animation_has_matching_aot_and_live_source_diagnostics() {
    let aot = Compiler::compile(
        [SourceModule::new("ui/main.argui", SOURCE)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .err()
    .expect("AOT compilation should reject structural animation");
    let mut live = CompilerSession::new("ui/main.argui").unwrap();
    live.update_module(SourceModule::new("ui/main.argui", SOURCE));
    let live = live
        .compile(|_| Err("no assets".into()))
        .err()
        .expect("incremental compilation should reject structural animation");
    for error in [aot, live] {
        let CompilerError::Semantic(diagnostics) = error else {
            panic!("expected semantic diagnostic");
        };
        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == DiagnosticCode::InvalidAnimation)
            .expect("structural animation diagnostic");
        assert!(diagnostic.message.contains("`row_height` is structural"));
        let range = diagnostic.primary.range;
        assert_eq!(
            &SOURCE[u32::from(range.start()) as usize..u32::from(range.end()) as usize],
            "animate row_height { duration: 100ms }"
        );
    }
}
