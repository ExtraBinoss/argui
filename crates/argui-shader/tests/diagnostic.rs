use argui_shader::{ShaderDiagnostic, ShaderError, ShaderSourcePosition};

#[test]
fn diagnostics_render_file_line_column_and_message() {
    let diagnostic = ShaderDiagnostic {
        position: ShaderSourcePosition {
            source_name: "fx.wgsl".into(),
            line: 7,
            column: 12,
        },
        message: "invalid expression".into(),
    };
    assert_eq!(diagnostic.to_string(), "fx.wgsl:7:12: invalid expression");
    assert_eq!(
        ShaderError::Diagnostic(diagnostic).to_string(),
        "fx.wgsl:7:12: invalid expression"
    );
}
