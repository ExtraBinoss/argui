//! Project asset validation and source-location regressions.

use argui_dsl_compiler::{Compiler, CompilerError, SourceModule};

const SVG: &[u8] = br#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"><path d="M1 1h14v14H1z"/></svg>"#;

#[test]
fn project_svg_is_validated_and_embedded_for_aot_native_svg() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Svg } from "@argui/native"
export component Main {
    Svg { source: asset("media/icon.svg") width: 16px height: 16px }
}"#,
        )],
        "ui/main.argui",
        |path| {
            assert_eq!(path, "ui/media/icon.svg");
            Ok(SVG.to_vec())
        },
    )
    .unwrap();
    assert_eq!(compiled.dependencies, ["ui/media/icon.svg"]);
    assert!(compiled.rust.contains("asset_handle("));
    assert!(compiled.rust.contains("pub fn vector_assets()"));
    assert!(compiled.rust.contains("include_bytes!"));
}

#[test]
fn malformed_svg_is_rejected_before_aot_emission() {
    let result = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Svg } from "@argui/native"
export component Main { Svg { source: asset("bad.svg") } }"#,
        )],
        "ui/main.argui",
        |_| Ok(b"<svg".to_vec()),
    );
    assert!(matches!(result, Err(CompilerError::Asset { .. })));
}

#[test]
fn malformed_raster_image_is_rejected_before_aot_emission() {
    let result = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Image } from "@argui/native"
export component Main { Image { source: asset("bad.png") } }"#,
        )],
        "ui/main.argui",
        |_| Ok(b"not-a-png".to_vec()),
    );
    assert!(matches!(result, Err(CompilerError::Asset { .. })));
}

#[test]
fn missing_media_asset_error_retains_its_dsl_expression_span() {
    let source = r#"import { Image } from "@argui/native"
export component Main { Image { source: asset("missing.png") } }"#;
    let result = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("file not found".into()),
    );
    let Err(CompilerError::Asset {
        path,
        source_span: Some(span),
        ..
    }) = result
    else {
        panic!("missing image should retain its DSL source location");
    };
    assert_eq!(path.to_string_lossy(), "ui/missing.png");
    assert_eq!(
        &source[u32::from(span.range.start()) as usize..u32::from(span.range.end()) as usize],
        "asset(\"missing.png\")"
    );
}

#[test]
fn svg_source_cannot_be_used_as_raster_image() {
    let result = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Image } from "@argui/native"
export component Main { Image { source: asset("icon.svg") } }"#,
        )],
        "ui/main.argui",
        |_| Ok(SVG.to_vec()),
    );
    let Err(CompilerError::Asset { message, .. }) = result else {
        panic!("expected media kind rejection");
    };
    assert!(message.contains("expected Image media"));
}

#[test]
fn virtual_tabler_svg_uses_embedded_catalogue_bytes() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { Svg } from "@argui/native"
export component Main { Svg { source: asset("@argui/icons/Loader2.svg") } }"#,
        )],
        "ui/main.argui",
        |_| Err("filesystem loader must not receive virtual icons".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("pub static ASSET_"));
    assert!(
        !compiled
            .rust
            .contains("include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/@argui/")
    );
}

#[test]
fn missing_asset_in_theme_token_retains_token_expression_span() {
    let source = r#"import { Svg } from "@argui/native"
export theme Icons { --logo: asset = asset("theme-logo.svg") }
export component Main { Svg { source: var(--logo) } }"#;
    let error = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("theme asset missing".into()),
    )
    .err()
    .expect("reachable theme asset must load");
    let CompilerError::Asset {
        source_span: Some(span),
        message,
        ..
    } = error
    else {
        panic!("expected source-located asset error");
    };
    assert_eq!(message, "theme asset missing");
    assert_eq!(
        &source[u32::from(span.range.start()) as usize..u32::from(span.range.end()) as usize],
        "asset(\"theme-logo.svg\")"
    );
}

#[test]
fn missing_asset_in_theme_mode_retains_override_expression_span() {
    let source = r#"import { Svg } from "@argui/native"
export theme Icons {
    --logo: asset = asset("light-logo.svg")
    dark { --logo: asset("dark-logo.svg") }
}
export component Main { Svg { source: var(--logo) } }"#;
    let error = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |path| {
            if path == "ui/light-logo.svg" {
                Ok(SVG.to_vec())
            } else {
                Err("dark asset missing".into())
            }
        },
    )
    .err()
    .expect("reachable dark theme asset must load");
    let CompilerError::Asset {
        source_span: Some(span),
        message,
        ..
    } = error
    else {
        panic!("expected source-located asset error");
    };
    assert_eq!(message, "dark asset missing");
    assert_eq!(
        &source[u32::from(span.range.start()) as usize..u32::from(span.range.end()) as usize],
        "asset(\"dark-logo.svg\")"
    );
}

#[test]
fn missing_asset_in_conditional_expression_retains_selected_operand_span() {
    let source = r#"import { Svg } from "@argui/native"
export component Main {
    private property dark: bool = false
    private property logo: asset = dark ? asset("dark.svg") : asset("light.svg")
    Svg { source: logo }
}"#;
    let error = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |path| {
            if path == "ui/dark.svg" {
                Ok(SVG.to_vec())
            } else {
                Err("light asset missing".into())
            }
        },
    )
    .err()
    .expect("reachable conditional asset must load");
    let CompilerError::Asset {
        source_span: Some(span),
        message,
        ..
    } = error
    else {
        panic!("expected source-located asset error");
    };
    assert_eq!(message, "light asset missing");
    assert_eq!(
        &source[u32::from(span.range.start()) as usize..u32::from(span.range.end()) as usize],
        "asset(\"light.svg\")"
    );
}

#[test]
fn missing_asset_in_component_state_retains_assignment_span() {
    let source = r#"import { Svg } from "@argui/native"
export component Main {
    private property dark: bool = false
    private property logo: asset = asset("light.svg")
    states { dark_mode when dark { logo: asset("dark.svg") } }
    Svg { source: logo }
}"#;
    let error = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |path| {
            if path == "ui/light.svg" {
                Ok(SVG.to_vec())
            } else {
                Err("dark state asset missing".into())
            }
        },
    )
    .err()
    .expect("reachable state asset must load");
    let CompilerError::Asset {
        source_span: Some(span),
        message,
        ..
    } = error
    else {
        panic!("expected source-located asset error");
    };
    assert_eq!(message, "dark state asset missing");
    assert_eq!(
        &source[u32::from(span.range.start()) as usize..u32::from(span.range.end()) as usize],
        "asset(\"dark.svg\")"
    );
}

#[test]
fn missing_asset_in_exported_style_retains_binding_span() {
    let source = r#"import { Svg } from "@argui/native"
export style Icon for Svg { source: asset("style.svg") }
export component Main {}"#;
    let error = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("style asset missing".into()),
    )
    .err()
    .expect("reachable style asset must load");
    let CompilerError::Asset {
        source_span: Some(span),
        message,
        ..
    } = error
    else {
        panic!("expected source-located asset error");
    };
    assert_eq!(message, "style asset missing");
    assert_eq!(
        &source[u32::from(span.range.start()) as usize..u32::from(span.range.end()) as usize],
        "asset(\"style.svg\")"
    );
}

#[test]
fn missing_asset_in_exported_style_state_retains_binding_span() {
    let source = r#"import { Svg } from "@argui/native"
export style Icon for Svg { hovered { source: asset("hover.svg") } }
export component Main {}"#;
    let error = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("style-state asset missing".into()),
    )
    .err()
    .expect("reachable style-state asset must load");
    let CompilerError::Asset {
        source_span: Some(span),
        message,
        ..
    } = error
    else {
        panic!("expected source-located asset error");
    };
    assert_eq!(message, "style-state asset missing");
    assert_eq!(
        &source[u32::from(span.range.start()) as usize..u32::from(span.range.end()) as usize],
        "asset(\"hover.svg\")"
    );
}

#[test]
fn missing_asset_in_array_default_retains_element_span() {
    let source = r#"export component Main {
    private property icons: array<asset> = [asset("first.svg"), asset("second.svg")]
}"#;
    let error = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |path| {
            if path == "ui/first.svg" {
                Ok(SVG.to_vec())
            } else {
                Err("second array asset missing".into())
            }
        },
    )
    .err()
    .expect("reachable array asset must load");
    let CompilerError::Asset {
        source_span: Some(span),
        message,
        ..
    } = error
    else {
        panic!("expected source-located asset error");
    };
    assert_eq!(message, "second array asset missing");
    assert_eq!(
        &source[u32::from(span.range.start()) as usize..u32::from(span.range.end()) as usize],
        "asset(\"second.svg\")"
    );
}

#[test]
fn missing_asset_in_event_assignment_retains_handler_expression_span() {
    let source = r#"import { TouchArea, Svg } from "@argui/native"
export component Main {
    private property logo: asset
    TouchArea { on click { logo = asset("clicked.svg") } }
    Svg { source: logo }
}"#;
    let error = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("clicked asset missing".into()),
    )
    .err()
    .expect("reachable handler asset must load");
    let CompilerError::Asset {
        source_span: Some(span),
        message,
        ..
    } = error
    else {
        panic!("expected source-located asset error");
    };
    assert_eq!(message, "clicked asset missing");
    assert_eq!(
        &source[u32::from(span.range.start()) as usize..u32::from(span.range.end()) as usize],
        "asset(\"clicked.svg\")"
    );
}
