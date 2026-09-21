use argui_dsl_compiler::{Compiler, SourceModule};

#[test]
fn aot_virtual_list_borrows_direct_models_and_emits_measured_scroll_viewport() {
    let source = r#"import { VList, Text } from "@argui/native"
export component Main {
    private property items: array<string> = ["one", "two"]
    private property offset: float = 0.0
    VList #results {
        width: 100%
        height: 100%
        row_height: 32.0
        offset <=> offset
        for item in items key item { Text { content: item } }
    }
}"#;
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no external assets".into()),
    )
    .unwrap();
    assert!(
        compiled.rust.contains(".with(|children_"),
        "{}",
        compiled.rust
    );
    assert!(compiled.rust.contains("virtual_viewports.get(&identity_"));
    assert!(
        compiled
            .rust
            .contains("HashMap<::argui::ui::RetainedIdentity, f32>")
    );
    assert!(compiled.rust.contains("fn layout_changed(&mut self"));
    assert!(
        compiled
            .rust
            .contains("UiEventKind::Scrolled { offset, .. }")
    );
    assert!(compiled.rust.contains("SchemaValue::Int(children_"));
    assert!(compiled.rust.contains("usize::try_from(3).unwrap_or(0)"));
    assert!(!compiled.rust.contains("as usize).window"));
}

#[test]
fn aot_template_virtual_list_keeps_caller_model_lazy() {
    let source = r#"import { VirtualList } from "@argui/ui"
import { Text } from "@argui/native"
export component Main {
    private property items: array<string> = ["one", "two"]
    private property offset: float = 0.0
    VirtualList {
        row_height: 32.0
        offset <=> offset
        for item in items key item { Text { content: item } }
    }
}"#;
    let compiled = Compiler::compile(
        [SourceModule::new("ui/main.argui", source)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("template_roots_"));
    assert!(compiled.rust.contains(".with(|children_"));
    assert!(
        compiled
            .rust
            .contains("UiEventKind::Scrolled { offset, .. }")
    );
    assert!(!compiled.rust.contains("template slot is outside"));
}
