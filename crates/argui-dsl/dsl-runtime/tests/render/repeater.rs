//! Repeater key collisions are diagnosed before mounting ambiguous rows.

use argui_dsl_compiler::{Compiler, SourceModule};
use argui_dsl_runtime::{LivePackage, LiveRuntime, RuntimeError};
use std::collections::HashMap;

/// Duplicate integer and string keys fail explicitly on ordinary and virtual rows.
#[test]
fn duplicate_repeater_keys_are_rejected() {
    for values in ["[1, 1]", "[\"same\", \"same\"]"] {
        for wrapper in [
            "Column {",
            "VirtualWindow { row_height: 20.0 viewport_height: 100.0 offset <=> offset",
        ] {
            let source = format!(
                "import {{ Column, VirtualWindow, Text }} from \"@argui/native\"
                 export component Main {{ private property offset: float = 0.0 {wrapper}
                    for row in {values} key row {{ Text {{ content: str(row) }} }}
                 }} }}"
            );
            let compiled = Compiler::compile(
                [SourceModule::new("main.argui", source)],
                "main.argui",
                |_| Err("no assets".into()),
            )
            .unwrap();
            let component = compiled.roots[0];
            let package =
                LivePackage::prepare(1, compiled.public_api_hash, compiled.ir, HashMap::new())
                    .unwrap();
            let mut runtime = LiveRuntime::new(package).unwrap();
            runtime.mount(component, []).unwrap();
            assert!(
                matches!(runtime.render(), Err(RuntimeError::InvalidBytecode(message)) if message.contains("duplicate repeater key"))
            );
        }
    }
}
