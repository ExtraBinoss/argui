use argui_dsl_compiler::{Compiler, SourceModule};

/// The capability metadata sends VirtualWindow through the same lazy AOT path.
#[test]
fn aot_virtual_window_emits_bounded_keyed_rows() {
    let compiled = Compiler::compile(
        [SourceModule::new(
            "ui/main.argui",
            r#"import { VirtualWindow, Text } from "@argui/native"
export component Main {
    private property rows: array<string> = ["a", "b"]
    private property offset: float = 0.0
    VirtualWindow { row_height: 24.0 viewport_height: 48.0 offset <=> offset
        for row in rows key row { Text { content: row } }
    }
}"#,
        )],
        "ui/main.argui",
        |_| Err("no external assets".into()),
    )
    .unwrap();
    assert!(compiled.rust.contains("::argui::ui::VirtualList::fixed("));
    assert!(compiled.rust.contains("SchemaValue::Int(children_"));
}

#[test]
fn aot_virtual_list_borrows_direct_models_and_emits_measured_scroll_viewport() {
    let source = r#"import { VirtualWindow, Text } from "@argui/native"
export component Main {
    private property items: array<string> = ["one", "two"]
    private property offset: float = 0.0
    VirtualWindow #results {
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
    let source = r#"import { ListView } from "@argui/ui"
import { Text } from "@argui/native"
export component Main {
    private property items: array<string> = ["one", "two"]
    private property offset: float = 0.0
    ListView {
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

/// A custom template list samples its own stateful property before rendering rows.
#[test]
fn aot_template_virtual_list_samples_component_state() {
    let source = r#"import { VirtualWindow, Text } from "@argui/native"
component Rows {
    in property tone: float = 1.0
    in property active: bool = false
    in-out property offset: float = 0.0
    states { dim when active { tone: 0.5 } }
    animate tone { transition: in-out duration: 100ms }
    slot rows: template
    VirtualWindow { row_height: 24.0 offset <=> offset rows }
}
export component Main {
    private property items: array<string> = ["one"]
    private property offset: float = 0.0
    Rows {
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
    assert!(compiled.rust.contains("template_state_"));
    assert!(compiled.rust.contains("StateTransitionPolicy::InOut"));
    assert!(compiled.rust.contains("template_roots_"));

    let without_animation = source.replace(
        "    animate tone { transition: in-out duration: 100ms }\n",
        "",
    );
    let state_only = Compiler::compile(
        [SourceModule::new("ui/main.argui", without_animation)],
        "ui/main.argui",
        |_| Err("no assets".into()),
    )
    .unwrap();
    assert!(state_only.rust.contains("template_state_"));
    assert!(!state_only.rust.contains("StateTransitionPolicy::InOut"));
}
